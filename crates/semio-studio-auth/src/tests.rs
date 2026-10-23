use super::*;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use wiremock::matchers::{body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// An unsigned JWT carrying `claims`, as the Auth emulator issues them.
fn id_token(claims: serde_json::Value) -> String {
  let encode = |value: &serde_json::Value| {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value.to_string())
  };
  format!(
    "{}.{}.",
    encode(&json!({"alg": "none", "typ": "JWT"})),
    encode(&claims)
  )
}

fn config(server: &MockServer) -> AuthConfig {
  AuthConfig {
    api_key: "public-key".into(),
    token_url: server.uri(),
  }
}

fn token_response(project: &str, refresh_token: &str) -> ResponseTemplate {
  ResponseTemplate::new(200).set_body_json(json!({
      "user_id": "ada",
      "id_token": id_token(json!({"aud": project, "sub": "ada", "email": "ada@example.com",
          "email_verified": true, "firebase": {"sign_in_provider": "password"}})),
      "refresh_token": refresh_token,
      "expires_in": "3600",
      "token_type": "Bearer",
  }))
}

#[test]
fn claims_follow_studio_s_verified_account() {
  let claims = |value| Claims::of(&id_token(value)).unwrap();
  assert!(
    claims(json!({"email_verified": true, "firebase": {"sign_in_provider": "password"}}))
      .verified()
  );
  assert!(claims(json!({"firebase": {"sign_in_provider": "google.com"}})).verified());
  assert!(
    !claims(json!({"email_verified": false, "firebase": {"sign_in_provider": "password"}}))
      .verified()
  );
  assert!(
    !claims(json!({"email_verified": true, "firebase": {"sign_in_provider": "anonymous"}}))
      .verified()
  );
  let read = claims(json!({"aud": "demo-test", "sub": "ada", "email": "ada@example.com"}));
  assert_eq!((read.aud.as_str(), read.sub.as_str()), ("demo-test", "ada"));
  assert_eq!(read.email.as_deref(), Some("ada@example.com"));
  assert!(Claims::of("not a jwt").is_err());
  assert!(Claims::of("a.!!!.c").is_err());
}

#[test]
fn configuration_reads_the_key_or_the_emulator() {
  let lookup = |pairs: &'static [(&'static str, &'static str)]| {
    move |name: &str| {
      pairs
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_string())
    }
  };
  let production = AuthConfig::from_lookup(lookup(&[("FIREBASE_API_KEY", "k")])).unwrap();
  assert_eq!(production.token_url, "https://securetoken.googleapis.com");
  let emulator =
    AuthConfig::from_lookup(lookup(&[("FIREBASE_AUTH_EMULATOR_HOST", "127.0.0.1:9")])).unwrap();
  assert_eq!(
    emulator.token_url,
    "http://127.0.0.1:9/securetoken.googleapis.com"
  );
  let error = AuthConfig::from_lookup(lookup(&[("FIREBASE_API_KEY", "")])).unwrap_err();
  assert!(matches!(error, AuthError::Config(_)), "{error}");
}

#[tokio::test]
async fn refresh_exchanges_the_token_for_the_named_project_only() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .and(path("/v1/token"))
    .and(query_param("key", "public-key"))
    .and(body_string_contains("grant_type=refresh_token"))
    .and(body_string_contains("refresh_token=stored"))
    .respond_with(token_response("demo-test", "rotated"))
    .mount(&server)
    .await;
  let session = refresh(&config(&server), "demo-test", "stored")
    .await
    .unwrap();
  assert_eq!(session.uid, "ada");
  assert_eq!(session.refresh_token(), "rotated");
  assert!(session.claims().unwrap().verified());
  assert!(!format!("{session:?}").contains("rotated"));

  let error = refresh(&config(&server), "demo-other", "stored")
    .await
    .unwrap_err();
  assert!(matches!(error, AuthError::WrongProject { .. }), "{error}");
}

#[tokio::test]
async fn refusals_name_the_code_and_never_the_token() {
  for (status, body, expected) in [
    (
      400,
      json!({"error": {"message": "TOKEN_EXPIRED"}}),
      "SessionEnded",
    ),
    (
      400,
      json!({"error": {"message": "INVALID_REFRESH_TOKEN"}}),
      "SessionEnded",
    ),
    (
      400,
      json!({"error": {"message": "USER_DISABLED"}}),
      "SessionEnded",
    ),
    (
      400,
      json!({"error": {"message": "TOO_MANY_ATTEMPTS_TRY_LATER : later"}}),
      "Network",
    ),
    (503, json!({}), "Network"),
    (
      400,
      json!({"error": {"message": "API_KEY_INVALID"}}),
      "Refused",
    ),
  ] {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
      .respond_with(ResponseTemplate::new(status).set_body_json(body))
      .mount(&server)
      .await;
    let error = refresh(&config(&server), "demo-test", "secret-refresh-token")
      .await
      .unwrap_err();
    assert!(format!("{error:?}").starts_with(expected), "{error:?}");
    assert!(!error.to_string().contains("secret-refresh-token"));
  }
}

#[tokio::test]
async fn a_user_renews_once_and_saves_each_rotated_token() {
  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .respond_with(token_response("demo-test", "rotated"))
    .expect(1)
    .mount(&server)
    .await;
  let saved = Arc::new(std::sync::Mutex::new(Vec::new()));
  let kept = saved.clone();
  let user = StudioUser::new(
    config(&server),
    "demo-test".into(),
    "stored".into(),
    Box::new(move |token| {
      kept.lock().unwrap().push(token.to_owned());
      Ok(())
    }),
  );
  let first = user.id_token().await.unwrap();
  let second = user.id_token().await.unwrap();
  assert_eq!(first, second);
  assert_eq!(*saved.lock().unwrap(), ["rotated"]);

  let server = MockServer::start().await;
  Mock::given(method("POST"))
    .respond_with(token_response("demo-test", "rotated"))
    .mount(&server)
    .await;
  let failing = StudioUser::new(
    config(&server),
    "demo-test".into(),
    "stored".into(),
    Box::new(|_| Err("store locked".into())),
  );
  let error = failing.session().await.unwrap_err();
  assert!(error.to_string().contains("store locked"), "{error}");
}

/// A device client or a terminal UI shares one user between tasks, and awaits
/// its tokens on a multi-threaded runtime.
#[test]
fn a_user_is_shared_across_threads() {
  fn send_sync<T: Send + Sync>(_: &T) {}
  fn send<T: Send>(_: &T) {}
  let user = StudioUser::new(
    AuthConfig {
      api_key: "public-key".into(),
      token_url: "http://127.0.0.1:9".into(),
    },
    "demo-test".into(),
    "stored".into(),
    Box::new(|_| Ok(())),
  );
  send_sync(&user);
  // Never polled: no request is made.
  send(&user.id_token());
  send(&user.session());
}

/// Sends one request to a listener, as a browser or another local client.
fn send(
  port: u16,
  method: reqwest::Method,
  origin: Option<&str>,
  content_type: &str,
  body: String,
) -> reqwest::blocking::Response {
  let mut request = reqwest::blocking::Client::new()
    .request(method, format!("http://127.0.0.1:{port}/callback"))
    .header("Content-Type", content_type)
    .body(body);
  if let Some(origin) = origin {
    request = request.header("Origin", origin);
  }
  request.send().unwrap()
}

#[test]
fn tool_login_admits_only_studio_s_page_with_the_printed_request() {
  let login = ToolLogin::start("http://studio.test/somewhere").unwrap();
  let url = Url::parse(login.url()).unwrap();
  assert_eq!(url.origin().ascii_serialization(), "http://studio.test");
  assert_eq!(url.path(), "/tool-login");
  let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
  let request = login.request_id().to_owned();
  assert_eq!(query["request"], request);
  assert_eq!(request.len(), REQUEST_ID_LEN);
  assert!(request.bytes().all(|b| REQUEST_ID_ALPHABET.contains(&b)));
  let port: u16 = query["port"].parse().unwrap();

  let calls = Arc::new(AtomicUsize::new(0));
  let counted = calls.clone();
  let client = std::thread::spawn(move || {
    let body = |request: &str| {
      json!({"request": request, "refreshToken": "shared", "uid": "ada"}).to_string()
    };
    let mut statuses = Vec::new();
    let call = |method, origin, content_type, body| {
      counted.fetch_add(1, Ordering::SeqCst);
      send(port, method, origin, content_type, body)
    };
    let preflight = call(
      reqwest::Method::OPTIONS,
      Some("http://studio.test"),
      "text/plain",
      String::new(),
    );
    statuses.push(preflight.status().as_u16());
    assert_eq!(
      preflight.headers()["access-control-allow-origin"],
      "http://studio.test"
    );
    let foreign = call(
      reqwest::Method::OPTIONS,
      Some("http://evil.test"),
      "text/plain",
      String::new(),
    );
    statuses.push(foreign.status().as_u16());
    assert!(
      !foreign
        .headers()
        .contains_key("access-control-allow-origin")
    );
    let unnamed = json!({"request": request, "refreshToken": "shared", "uid": ""}).to_string();
    // One byte over the cap, and valid JSON: only the size refuses it.
    let base = json!({"request": request, "refreshToken": "", "uid": "ada"})
      .to_string()
      .len();
    let oversized = json!({
        "request": request,
        "refreshToken": "x".repeat(CALLBACK_LIMIT as usize + 1 - base),
        "uid": "ada",
    })
    .to_string();
    assert_eq!(oversized.len() as u64, CALLBACK_LIMIT + 1);
    for (origin, content_type, body) in [
      (Some("http://evil.test"), "application/json", body(&request)),
      (None, "application/json", body(&request)),
      (Some("http://studio.test"), "text/plain", body(&request)),
      (
        Some("http://studio.test"),
        "application/json",
        body("WRONGWRONG"),
      ),
      (Some("http://studio.test"), "application/json", "{".into()),
      (Some("http://studio.test"), "application/json", unnamed),
      (Some("http://studio.test"), "application/json", oversized),
    ] {
      statuses.push(
        call(reqwest::Method::POST, origin, content_type, body)
          .status()
          .as_u16(),
      );
    }
    let accepted = call(
      reqwest::Method::POST,
      Some("http://studio.test"),
      "application/json; charset=utf-8",
      body(&request),
    );
    statuses.push(accepted.status().as_u16());
    statuses
  });
  let approval = login.wait(Duration::from_secs(30)).unwrap();
  assert_eq!(approval.uid, "ada");
  assert_eq!(approval.refresh_token(), "shared");
  assert!(!format!("{approval:?}").contains("shared"));
  assert_eq!(
    client.join().unwrap(),
    [204, 403, 403, 403, 415, 400, 400, 400, 400, 200]
  );
  assert_eq!(calls.load(Ordering::SeqCst), 10);
}

#[test]
fn tool_login_gives_up_after_its_timeout() {
  let login = ToolLogin::start("https://studio.test").unwrap();
  let error = login.wait(Duration::from_millis(200)).unwrap_err();
  assert!(matches!(error, AuthError::TimedOut(_)), "{error}");
  assert!(ToolLogin::start("ftp://studio.test").is_err());
  assert!(ToolLogin::start("not a url").is_err());
}
