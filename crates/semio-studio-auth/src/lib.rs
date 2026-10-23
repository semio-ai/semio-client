#![doc = include_str!("../README.md")]

use base64::Engine as _;
use futures_util::lock::Mutex;
use rand::Rng as _;
use reqwest::{Client, Url};
use serde::Deserialize;
use std::fmt;
use std::io::Read as _;
use std::time::{Duration, Instant};

/// An ID token this close to its expiry is renewed before it is handed out, so
/// a request does not reach the backend with a token that expires on the way.
const RENEW_BEFORE_EXPIRY: Duration = Duration::from_secs(5 * 60);

/// Firebase Auth's Secure Token endpoint for a Studio project.
#[derive(Clone, Debug)]
pub struct AuthConfig {
  /// The project's web API key. It names the project to Firebase Auth and is
  /// public: Studio's web app ships it to every browser.
  pub api_key: String,
  /// Secure Token origin, without `/v1`.
  pub token_url: String,
}

impl AuthConfig {
  /// Reads `FIREBASE_API_KEY` and `FIREBASE_AUTH_EMULATOR_HOST` through a
  /// caller-owned lookup. Empty values count as unset. With an emulator the
  /// key may be absent: the emulator accepts any key.
  pub fn from_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Result<Self, AuthError> {
    let mut value = |name| lookup(name).filter(|s: &String| !s.is_empty());
    let emulator = value("FIREBASE_AUTH_EMULATOR_HOST");
    let api_key = match (value("FIREBASE_API_KEY"), &emulator) {
      (Some(key), _) => key,
      (None, Some(_)) => "emulator".into(),
      (None, None) => return Err(AuthError::Config("missing FIREBASE_API_KEY".into())),
    };
    let token_url = match emulator {
      Some(host) => format!("http://{host}/securetoken.googleapis.com"),
      None => "https://securetoken.googleapis.com".into(),
    };
    Ok(Self { api_key, token_url })
  }
}

/// Why signing in or renewing a session failed.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AuthError {
  /// The refresh token was revoked or expired, or its account was deleted or
  /// disabled: the user has to sign in again.
  #[error("the session has ended; sign in again ({0})")]
  SessionEnded(String),
  /// The token belongs to another Firebase project than the one configured.
  #[error("the session belongs to Firebase project {actual}, not {expected}")]
  WrongProject { expected: String, actual: String },
  /// The tool-login page was not approved in time.
  #[error("no approval arrived from the sign-in page within {0:?}")]
  TimedOut(Duration),
  /// Too many attempts, a timeout, or a transport failure.
  #[error("network: {0}")]
  Network(String),
  /// Anything else Firebase Auth answered, by its error code.
  #[error("Firebase Auth refused: {0}")]
  Refused(String),
  /// A response or a callback that does not have the documented shape.
  #[error("malformed: {0}")]
  Malformed(String),
  /// The configuration is incomplete.
  #[error("configuration: {0}")]
  Config(String),
}

/// What an ID token says about its account. Read without verifying the
/// token's signature: it informs messages and early refusals, while the
/// backend, which verifies it, decides.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Claims {
  /// The project the token was issued for.
  #[serde(default)]
  pub aud: String,
  /// The account's user id.
  #[serde(default)]
  pub sub: String,
  pub email: Option<String>,
  #[serde(default)]
  pub email_verified: bool,
  #[serde(default)]
  pub firebase: FirebaseClaims,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FirebaseClaims {
  pub sign_in_provider: Option<String>,
}

impl Claims {
  /// The claims of an ID token, a JWT.
  pub fn of(id_token: &str) -> Result<Self, AuthError> {
    let payload = id_token
      .split('.')
      .nth(1)
      .ok_or_else(|| AuthError::Malformed("an ID token is a JWT".into()))?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
      .decode(payload.trim_end_matches('='))
      .map_err(|_| AuthError::Malformed("an ID token's payload is base64url".into()))?;
    serde_json::from_slice(&bytes).map_err(|e| AuthError::Malformed(e.to_string()))
  }

  /// Whether Studio's rules count the account as verified, which publishing
  /// and claiming names require: a verified email, or a Google sign-in.
  pub fn verified(&self) -> bool {
    let provider = self.firebase.sign_in_provider.as_deref();
    provider != Some("anonymous") && (self.email_verified || provider == Some("google.com"))
  }
}

/// A signed-in account: who it is, a short-lived ID token, and the refresh
/// token that renews it. `Debug` leaves both tokens out.
#[derive(Clone)]
pub struct Session {
  /// The account's Firebase user id.
  pub uid: String,
  refresh_token: String,
  id_token: String,
  expires_at: Instant,
}

impl Session {
  /// The long-lived secret that renews the session. Keep it in a secret
  /// store; never print it.
  pub fn refresh_token(&self) -> &str {
    &self.refresh_token
  }

  /// The current ID token, valid for about an hour after it was issued.
  pub fn id_token(&self) -> &str {
    &self.id_token
  }

  /// What the ID token says about the account.
  pub fn claims(&self) -> Result<Claims, AuthError> {
    Claims::of(&self.id_token)
  }

  fn fresh(&self) -> bool {
    Instant::now() + RENEW_BEFORE_EXPIRY < self.expires_at
  }
}

impl fmt::Debug for Session {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Session")
      .field("uid", &self.uid)
      .finish_non_exhaustive()
  }
}

/// Exchanges a refresh token for a session of `project_id`. The refresh token
/// in the returned session may differ from the one given: the caller saves it.
/// A token of another project is refused, so a sign-in through one Studio
/// deployment never stands for another.
pub async fn refresh(
  config: &AuthConfig,
  project_id: &str,
  refresh_token: &str,
) -> Result<Session, AuthError> {
  #[derive(Deserialize)]
  struct Response {
    user_id: String,
    id_token: String,
    refresh_token: String,
    expires_in: String,
  }
  let mut url = Url::parse(&format!("{}/v1/token", config.token_url))
    .map_err(|e| AuthError::Malformed(e.to_string()))?;
  if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
    return Err(AuthError::Malformed(
      "the token endpoint must be an HTTP(S) URL".into(),
    ));
  }
  url.query_pairs_mut().append_pair("key", &config.api_key);
  let request = client()?.post(url).form(&[
    ("grant_type", "refresh_token"),
    ("refresh_token", refresh_token),
  ]);
  let response: Response = post(request).await?;
  // The token's audience is the project it was issued for. (The response's
  // own `project_id` is the project's number in production, not its id.)
  let audience = Claims::of(&response.id_token)?.aud;
  if audience != project_id {
    return Err(AuthError::WrongProject {
      expected: project_id.into(),
      actual: audience,
    });
  }
  let seconds: u64 = response
    .expires_in
    .parse()
    .map_err(|_| AuthError::Malformed("expires_in must be a number of seconds".into()))?;
  Ok(Session {
    uid: response.user_id,
    expires_at: Instant::now() + Duration::from_secs(seconds),
    refresh_token: response.refresh_token,
    id_token: response.id_token,
  })
}

fn client() -> Result<Client, AuthError> {
  Client::builder()
    .connect_timeout(Duration::from_secs(30))
    .timeout(Duration::from_secs(60))
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .map_err(|e| AuthError::Malformed(e.to_string()))
}

/// Sends a request and reads its JSON answer. A refusal is reported by
/// Firebase Auth's error code alone: the request body, which carries the
/// refresh token, is never part of an error.
async fn post<T: for<'de> Deserialize<'de>>(
  request: reqwest::RequestBuilder,
) -> Result<T, AuthError> {
  let response = request
    .send()
    .await
    .map_err(|e| AuthError::Network(e.without_url().to_string()))?;
  let status = response.status();
  let bytes = response
    .bytes()
    .await
    .map_err(|e| AuthError::Network(e.without_url().to_string()))?;
  if status.is_success() {
    return serde_json::from_slice(&bytes).map_err(|e| AuthError::Malformed(e.to_string()));
  }
  let code = serde_json::from_slice::<serde_json::Value>(&bytes)
    .ok()
    .and_then(|v| v["error"]["message"].as_str().map(str::to_owned))
    .unwrap_or_else(|| format!("HTTP {status}"));
  Err(classify(status.as_u16(), code))
}

/// Maps Secure Token error codes onto what the user can do about them. A code
/// may carry a detail after a space or a colon.
fn classify(status: u16, code: String) -> AuthError {
  let name = code.split([' ', ':']).next().unwrap_or_default().to_owned();
  match name.as_str() {
    "TOKEN_EXPIRED"
    | "INVALID_REFRESH_TOKEN"
    | "USER_NOT_FOUND"
    | "USER_DISABLED"
    | "MISSING_REFRESH_TOKEN" => AuthError::SessionEnded(name),
    "TOO_MANY_ATTEMPTS_TRY_LATER" => AuthError::Network(name),
    _ if status == 408 || status == 429 || status >= 500 => AuthError::Network(code),
    _ => AuthError::Refused(code),
  }
}

/// Called with each refresh token Firebase Auth issues in place of the one
/// held, so the caller's store keeps the current one.
pub type OnRefreshToken = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// A signed-in Studio account: each request gets an ID token, renewed from the
/// refresh token when it is close to expiry.
pub struct StudioUser {
  config: AuthConfig,
  project_id: String,
  held: Mutex<Held>,
  on_refresh_token: OnRefreshToken,
}

enum Held {
  /// Only the refresh token, as read from the caller's store.
  Stored(String),
  Session(Session),
}

impl StudioUser {
  /// An account of `project_id` known by a refresh token from the caller's
  /// store. No request is made until a token is asked for.
  pub fn new(
    config: AuthConfig,
    project_id: String,
    refresh_token: String,
    on_refresh_token: OnRefreshToken,
  ) -> Self {
    Self {
      config,
      project_id,
      held: Mutex::new(Held::Stored(refresh_token)),
      on_refresh_token,
    }
  }

  /// The current session, renewed first if its ID token is close to expiry.
  /// A renewed refresh token is handed to the caller's callback before the
  /// session is used, so the store never falls behind the session.
  pub async fn session(&self) -> Result<Session, AuthError> {
    let mut held = self.held.lock().await;
    let stale = match &*held {
      Held::Session(session) if session.fresh() => return Ok(session.clone()),
      Held::Session(session) => session.refresh_token.clone(),
      Held::Stored(token) => token.clone(),
    };
    let session = refresh(&self.config, &self.project_id, &stale).await?;
    if session.refresh_token != stale {
      (self.on_refresh_token)(&session.refresh_token)
        .map_err(|e| AuthError::Refused(format!("saving the renewed refresh token: {e}")))?;
    }
    *held = Held::Session(session.clone());
    Ok(session)
  }

  /// The current ID token, renewed first as [`StudioUser::session`] does:
  /// what a request to Studio's backend carries.
  pub async fn id_token(&self) -> Result<String, AuthError> {
    Ok(self.session().await?.id_token)
  }
}

/// How long [`ToolLogin::wait`] waits for an approval by default.
pub const TOOL_LOGIN_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// The most bytes a callback body may hold. A refresh token is a few hundred
/// bytes; anything near this is not the page's post.
const CALLBACK_LIMIT: u64 = 16 * 1024;

/// Characters a request id is drawn from: uppercase letters and digits without
/// the look-alikes `0`, `O`, `1` and `I`, so a person can compare it at a
/// glance.
const REQUEST_ID_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Ten characters of a 32-character alphabet: 50 bits, which no caller can
/// guess within the listener's lifetime.
const REQUEST_ID_LEN: usize = 10;

/// What Studio's `/tool-login` page posted once the user approved: the
/// account's refresh token, and the uid the page says it belongs to.
pub struct Approval {
  pub uid: String,
  refresh_token: String,
}

impl Approval {
  /// The refresh token the page shared. Keep it in a secret store; never
  /// print it.
  pub fn refresh_token(&self) -> &str {
    &self.refresh_token
  }
}

impl fmt::Debug for Approval {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Approval")
      .field("uid", &self.uid)
      .finish_non_exhaustive()
  }
}

/// The terminal side of Studio's `/tool-login` page.
///
/// The wire contract is the page's: it is opened at
/// `{studio}/tool-login?port={port}&request={id}`, and on approval it sends
/// `POST http://127.0.0.1:{port}/callback` with the JSON body
/// `{request, refreshToken, uid}`. Because the body is JSON, a browser sends a
/// CORS preflight first, and the listener admits only Studio's origin there:
/// no other web page can deliver a callback. The request id the body echoes
/// must be the one printed, which ties the approval to this terminal.
pub struct ToolLogin {
  server: tiny_http::Server,
  origin: String,
  url: String,
  request_id: String,
}

impl ToolLogin {
  /// Listens on a loopback port chosen by the system, for an approval on the
  /// Studio deployment at `studio_url` (its origin, such as
  /// `https://studio.example.com`).
  pub fn start(studio_url: &str) -> Result<Self, AuthError> {
    let studio = Url::parse(studio_url)
      .map_err(|e| AuthError::Malformed(format!("the Studio URL {studio_url:?}: {e}")))?;
    if !matches!(studio.scheme(), "http" | "https") || studio.host_str().is_none() {
      return Err(AuthError::Malformed(format!(
        "the Studio URL {studio_url:?} must be an HTTP(S) URL"
      )));
    }
    let origin = studio.origin().ascii_serialization();
    let server = tiny_http::Server::http("127.0.0.1:0")
      .map_err(|e| AuthError::Network(format!("listening on a loopback port: {e}")))?;
    let port = server
      .server_addr()
      .to_ip()
      .ok_or_else(|| AuthError::Network("the loopback listener has no IP address".into()))?
      .port();
    let mut rng = rand::rng();
    let request_id: String = (0..REQUEST_ID_LEN)
      .map(|_| REQUEST_ID_ALPHABET[rng.random_range(0..REQUEST_ID_ALPHABET.len())] as char)
      .collect();
    let mut url = studio
      .join("tool-login")
      .map_err(|e| AuthError::Malformed(e.to_string()))?;
    url
      .query_pairs_mut()
      .append_pair("port", &port.to_string())
      .append_pair("request", &request_id);
    Ok(Self {
      server,
      origin,
      url: url.into(),
      request_id,
    })
  }

  /// The page to open in a browser.
  pub fn url(&self) -> &str {
    &self.url
  }

  /// The id the page shows, for the user to compare with the terminal's.
  pub fn request_id(&self) -> &str {
    &self.request_id
  }

  /// Serves the loopback listener until the page posts an approval for this
  /// request, or until `timeout`. Blocking: run it off the async runtime.
  ///
  /// A request that is not the page's callback is answered and ignored, so
  /// a stray or retried post does not end the sign-in.
  pub fn wait(self, timeout: Duration) -> Result<Approval, AuthError> {
    let deadline = Instant::now() + timeout;
    loop {
      let left = deadline.saturating_duration_since(Instant::now());
      if left.is_zero() {
        return Err(AuthError::TimedOut(timeout));
      }
      let Some(request) = self
        .server
        .recv_timeout(left)
        .map_err(|e| AuthError::Network(format!("receiving the approval: {e}")))?
      else {
        return Err(AuthError::TimedOut(timeout));
      };
      if let Some(approval) = self.answer(request) {
        return Ok(approval);
      }
    }
  }

  /// Answers one request; returns the approval it carried, if it was one.
  fn answer(&self, mut request: tiny_http::Request) -> Option<Approval> {
    let path = request
      .url()
      .split('?')
      .next()
      .unwrap_or_default()
      .to_owned();
    let origin = header_value(&request, "Origin");
    let from_studio = origin.as_deref() == Some(self.origin.as_str());
    let respond = |request: tiny_http::Request, status: u16, body: &str| {
      let mut response = tiny_http::Response::from_string(body).with_status_code(status);
      if from_studio {
        response = response
          .with_header(header("Access-Control-Allow-Origin", &self.origin))
          .with_header(header("Vary", "Origin"));
      }
      let _ = request.respond(response);
    };
    if path != "/callback" {
      respond(request, 404, "Not found");
      return None;
    }
    match request.method() {
      tiny_http::Method::Options if from_studio => {
        let response = tiny_http::Response::empty(204)
          .with_header(header("Access-Control-Allow-Origin", &self.origin))
          .with_header(header("Access-Control-Allow-Methods", "POST"))
          .with_header(header("Access-Control-Allow-Headers", "content-type"))
          .with_header(header("Vary", "Origin"));
        let _ = request.respond(response);
        None
      }
      tiny_http::Method::Post if from_studio => {
        // A JSON body is what makes a browser ask first; a simple
        // request skips the preflight that admits Studio alone.
        let json = header_value(&request, "Content-Type")
          .is_some_and(|value| value.split(';').next().map(str::trim) == Some("application/json"));
        if !json {
          respond(request, 415, "Expected application/json");
          return None;
        }
        let mut body = Vec::new();
        let read = request
          .as_reader()
          .take(CALLBACK_LIMIT + 1)
          .read_to_end(&mut body);
        if read.is_err() || body.len() as u64 > CALLBACK_LIMIT {
          respond(request, 400, "Bad request");
          return None;
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Callback {
          request: String,
          refresh_token: String,
          uid: String,
        }
        match serde_json::from_slice::<Callback>(&body) {
          Ok(callback)
            if callback.request == self.request_id
              && !callback.refresh_token.is_empty()
              && !callback.uid.is_empty() =>
          {
            respond(
              request,
              200,
              "Terminal linked: you can close this tab and return to your terminal.",
            );
            Some(Approval {
              uid: callback.uid,
              refresh_token: callback.refresh_token,
            })
          }
          _ => {
            respond(request, 400, "Bad request");
            None
          }
        }
      }
      _ => {
        respond(request, 403, "Forbidden");
        None
      }
    }
  }
}

fn header_value(request: &tiny_http::Request, name: &str) -> Option<String> {
  request
    .headers()
    .iter()
    .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
    .map(|h| h.value.as_str().to_owned())
}

fn header(name: &str, value: &str) -> tiny_http::Header {
  tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes())
    .expect("header names and values here are ASCII")
}

#[cfg(test)]
mod tests;
