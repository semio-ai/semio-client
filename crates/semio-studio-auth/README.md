# semio-studio-auth

A Semio Studio account's identity for tools that run outside the browser: a
command line, a device client, a terminal UI.

- `ToolLogin` signs a tool in through Studio's `/tool-login` page. The tool
  listens on a loopback port, the user opens the page, compares the request id
  it shows with the one the tool printed, and approves; the page then posts the
  account's Firebase refresh token to the listener. The account signs in to
  Studio however Studio allows (Google or a verified email), and no password
  ever reaches the tool.
- `refresh` exchanges a refresh token for a `Session` through Firebase
  Auth's Secure Token REST API, and refuses a token issued for another project
  than the one named, so a sign-in through one Studio deployment never stands
  for another.
- `StudioUser` keeps a refresh token and hands out ID tokens renewed from it,
  about five minutes before they expire. Each refresh token Firebase Auth
  issues in its place is handed to the caller's callback before it is used.
- `Claims` reads what an ID token says about its account, including whether
  Studio counts it as verified, which publishing needs.

This crate stores nothing: the caller keeps the refresh token in a secret store
of its choosing. Tokens are never formatted into errors, logs or `Debug`
output.

```rust,no_run
use semio_studio_auth::{AuthConfig, AuthError, StudioUser, TOOL_LOGIN_TIMEOUT, ToolLogin};

async fn sign_in(project_id: &str) -> Result<StudioUser, AuthError> {
    let config = AuthConfig::from_lookup(|name| std::env::var(name).ok())?;
    let login = ToolLogin::start("https://studio.example.com")?;
    eprintln!("Approve request {} at {}", login.request_id(), login.url());
    // Waiting blocks, so it runs off the async runtime.
    let approval = tokio::task::spawn_blocking(move || login.wait(TOOL_LOGIN_TIMEOUT))
        .await
        .expect("the sign-in thread panicked")?;
    // The page's word for the account is checked with Firebase Auth, and the
    // token against the project it must belong to.
    let session = semio_studio_auth::refresh(&config, project_id, approval.refresh_token()).await?;
    // Save `session.refresh_token()` in a secret store here.
    Ok(StudioUser::new(
        config,
        project_id.into(),
        session.refresh_token().into(),
        Box::new(|rotated| {
            // Replace the stored refresh token with `rotated`.
            Ok(())
        }),
    ))
}

async fn authorization(user: &StudioUser) -> Result<String, AuthError> {
    Ok(format!("Bearer {}", user.id_token().await?))
}
```

## Configuration

`AuthConfig::from_lookup` reads these through a caller-owned lookup, such as
`std::env::var`. Empty values count as unset. The fields also accept explicit
values.

| Variable | Meaning |
| --- | --- |
| `FIREBASE_API_KEY` | The project's web API key. It is public: Studio's web app ships it to every browser. Optional with an emulator. |
| `FIREBASE_AUTH_EMULATOR_HOST` | Optional `host:port` of the Auth emulator, prefixed with `http://` |

The default Secure Token origin is `https://securetoken.googleapis.com`.

## The `/tool-login` contract

The page is opened at `{studio}/tool-login?port={port}&request={id}`. On
approval it sends `POST http://127.0.0.1:{port}/callback` with the JSON body
`{request, refreshToken, uid}`. Because the body is JSON, a browser sends a
CORS preflight first, and the listener admits only Studio's origin there, so no
other web page can deliver a callback. The request id the body echoes must be
the one printed, which ties the approval to this terminal. A request that is
not the page's callback is answered and ignored, so a stray or retried post
does not end the sign-in.

## Errors

| `AuthError` | What the user can do |
| --- | --- |
| `SessionEnded` | The refresh token was revoked or expired, or its account was deleted or disabled: sign in again |
| `WrongProject` | Sign in through the Studio deployment of the configured project |
| `TimedOut` | Approve the sign-in page sooner, or start again |
| `Network` | Retry: too many attempts, a timeout or a transport failure |
| `Refused` | Anything else Firebase Auth answered, by its error code, or a refused save of a rotated refresh token |
| `Malformed` | A response, callback or URL without the documented shape |
| `Config` | Complete the configuration |

## Tests

`cargo test -p semio-studio-auth` runs the Secure Token exchange against a
local HTTP fake and `ToolLogin` against real loopback sockets. No test contacts
a Google or Firebase host.
