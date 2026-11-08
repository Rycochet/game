use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use axum::Router;
use axum::http::{HeaderMap, StatusCode, header};
use axum::routing::{get, post};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ring::digest::{SHA256, digest};
use serde_json::json;

use super::config::{callback_from, providers_from, secure_enough};
use super::provider::verified_email;
use super::*;
use crate::storage::tests::TestFolder;

/// A provider on this machine: only `good` and `unverified` codes work, and
/// it keeps the PKCE verifiers it was given.
pub(crate) struct FakeProvider {
    pub base: String,
    verifiers: Arc<Mutex<Vec<String>>>,
}

const FAKE_SECRET: &str = "fake secret";

impl FakeProvider {
    pub(crate) fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        let verifiers = Arc::<Mutex<Vec<String>>>::default();
        let routes = fake_routes(&base, verifiers.clone());
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async move {
                listener.set_nonblocking(true).unwrap();
                let listener = tokio::net::TcpListener::from_std(listener).unwrap();
                axum::serve(listener, routes).await.unwrap();
            });
        });
        Self { base, verifiers }
    }

    pub(crate) fn settings(&self) -> ProviderSettings {
        ProviderSettings {
            name: "fake".into(),
            label: "Fake".into(),
            issuer: self.base.clone(),
            client_id: "fake id".into(),
            client_secret: FAKE_SECRET.into(),
        }
    }

    /// Logins through this provider, hashing addresses with `secret`.
    pub(crate) fn logins(&self, secret: &str) -> Logins {
        let callback = "http://localhost:24690/login/callback".to_owned();
        Logins::start(vec![self.settings()], callback, LoginSecret::new(secret)).unwrap()
    }
}

fn fake_routes(base: &str, verifiers: Arc<Mutex<Vec<String>>>) -> Router {
    let discovery = json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/authorize"),
        "token_endpoint": format!("{base}/token"),
        "userinfo_endpoint": format!("{base}/userinfo"),
    })
    .to_string();
    Router::new()
        .route(
            "/.well-known/openid-configuration",
            get(move || async move { answer(StatusCode::OK, serde_json::from_str(&discovery).unwrap()) }),
        )
        .route(
            "/token",
            post(move |body: String| async move {
                let form: Vec<(String, String)> = url::form_urlencoded::parse(body.as_bytes())
                    .into_owned()
                    .collect();
                let field = |name: &str| {
                    form.iter()
                        .find(|(key, _)| key == name)
                        .map(|(_, value)| value.clone())
                        .unwrap_or_default()
                };
                verifiers.lock().unwrap().push(field("code_verifier"));
                if field("client_secret") != FAKE_SECRET
                    || field("grant_type") != "authorization_code"
                {
                    return answer(StatusCode::UNAUTHORIZED, json!({"error": "invalid_client"}));
                }
                let token = format!("token-{}", field("code"));
                answer(StatusCode::OK, json!({"access_token": token, "token_type": "Bearer"}))
            }),
        )
        .route(
            "/userinfo",
            get(|headers: HeaderMap| async move {
                let token = headers
                    .get(header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default();
                let verified = match token {
                    "Bearer token-good" => true,
                    "Bearer token-unverified" => false,
                    _ => return answer(StatusCode::UNAUTHORIZED, json!({})),
                };
                answer(
                    StatusCode::OK,
                    json!({"sub": "1", "email": " Someone@Example.com", "email_verified": verified}),
                )
            }),
        )
}

/// A JSON answer.
fn answer(
    status: StatusCode,
    body: serde_json::Value,
) -> (StatusCode, [(header::HeaderName, &'static str); 1], String) {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
}

/// A player coming back from the provider's page `url` with `code`, as the
/// browser would: the page's status.
pub(crate) fn come_back(logins: &Logins, url: &str, code: &str) -> StatusCode {
    let url = url::Url::parse(url).unwrap();
    let state = url.query_pairs().find(|(key, _)| key == "state").unwrap().1;
    come_back_with(logins, &format!("state={state}&code={code}"))
}

fn come_back_with(logins: &Logins, query: &str) -> StatusCode {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let query = RawQuery(Some(query.to_owned()));
    runtime
        .block_on(callback::came_back(logins.clone(), query))
        .0
}

fn variables(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    let pairs = pairs.iter();
    pairs
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

#[test]
fn google_needs_only_its_client_id_and_secret() {
    let (providers, warnings) = providers_from(&variables(&[
        ("HOW_OIDC_GOOGLE_CLIENT_ID", "id"),
        ("HOW_OIDC_GOOGLE_CLIENT_SECRET", " secret "),
        ("PATH", "x"),
    ]));
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(
        providers,
        [ProviderSettings {
            name: "google".into(),
            label: "Google".into(),
            issuer: "https://accounts.google.com".into(),
            client_id: "id".into(),
            client_secret: "secret".into(),
        }]
    );
}

#[test]
fn other_providers_need_an_issuer_and_are_named_by_their_variables() {
    let (providers, warnings) = providers_from(&variables(&[
        ("HOW_OIDC_TINYAUTH_CLIENT_ID", "id"),
        ("HOW_OIDC_TINYAUTH_CLIENT_SECRET", "secret"),
        ("HOW_OIDC_TINYAUTH_ISSUER", "https://auth.example.com"),
        ("HOW_OIDC_NOISSUER_CLIENT_ID", "id"),
        ("HOW_OIDC_NOISSUER_CLIENT_SECRET", "secret"),
        ("HOW_OIDC_NOSECRET_CLIENT_ID", "id"),
        ("HOW_OIDC_PLAIN_CLIENT_ID", "id"),
        ("HOW_OIDC_PLAIN_CLIENT_SECRET", "secret"),
        ("HOW_OIDC_PLAIN_ISSUER", "http://auth.example.com"),
        ("HOW_OIDC_LABELLED_CLIENT_ID", "id"),
        ("HOW_OIDC_LABELLED_CLIENT_SECRET", "secret"),
        ("HOW_OIDC_LABELLED_ISSUER", "http://localhost:8080/"),
        ("HOW_OIDC_LABELLED_LABEL", "Our Logins"),
    ]));
    let named: Vec<(&str, &str)> = providers
        .iter()
        .map(|p| (p.name.as_str(), p.label.as_str()))
        .collect();
    assert_eq!(
        named,
        [("labelled", "Our Logins"), ("tinyauth", "Tinyauth")]
    );
    assert_eq!(warnings.len(), 3, "{warnings:?}");
    assert!(
        warnings
            .iter()
            .all(|warning| warning.contains("logins are off"))
    );
}

#[test]
fn players_are_sent_back_to_the_login_address_or_the_lets_encrypt_domain() {
    let back = callback_from(Some("https://game.example.com/"), Some("other.example.com"));
    assert_eq!(back.unwrap(), "https://game.example.com/login/callback");
    let back = callback_from(None, Some("game.example.com"));
    assert_eq!(back.unwrap(), "https://game.example.com/login/callback");
    assert!(callback_from(None, None).is_err());
    assert!(callback_from(Some("http://game.example.com"), None).is_err());
    assert!(secure_enough("http://127.0.0.1:24690").is_ok());
    assert!(secure_enough("ftp://example.com").is_err());
    assert!(secure_enough("not an address").is_err());
}

#[test]
fn the_login_secret_is_made_once_or_taken_from_the_environment_and_kept() {
    let folder = TestFolder::new();
    let key = |secret: &LoginSecret| secret.email_key("a@example.com");
    let made = LoginSecret::load(&folder.0, None).unwrap();
    assert_eq!(
        key(&LoginSecret::load(&folder.0, None).unwrap()),
        key(&made)
    );
    // Another secret now would lose every account.
    assert!(LoginSecret::load(&folder.0, Some("another")).is_err());

    let folder = TestFolder::new();
    let given = LoginSecret::load(&folder.0, Some(" shared ")).unwrap();
    assert_eq!(key(&given), key(&LoginSecret::new("shared")));
    assert_eq!(
        key(&LoginSecret::load(&folder.0, None).unwrap()),
        key(&given)
    );
    assert!(LoginSecret::load(&folder.0, Some("shared")).is_ok());
}

#[test]
fn an_address_has_one_key_however_it_is_written_and_another_per_secret() {
    let secret = LoginSecret::new("one");
    let key = secret.email_key("someone@example.com");
    assert_eq!(secret.email_key(" Someone@Example.COM "), key);
    assert_ne!(secret.email_key("someone.else@example.com"), key);
    assert_ne!(
        LoginSecret::new("two").email_key("someone@example.com"),
        key
    );
}

#[test]
fn only_a_verified_address_is_taken() {
    let email = |info| verified_email(&info);
    let address = json!({"email": "a@example.com", "email_verified": true});
    assert_eq!(email(address).unwrap(), "a@example.com");
    let as_text = json!({"email": "a@example.com", "email_verified": "true"});
    assert_eq!(email(as_text).unwrap(), "a@example.com");
    assert!(email(json!({"email": "a@example.com", "email_verified": false})).is_err());
    assert!(email(json!({"email": "a@example.com"})).is_err());
    assert!(email(json!({"email_verified": true})).is_err());
}

#[test]
fn a_login_goes_through_the_providers_page_and_comes_back_with_the_address() {
    let provider = FakeProvider::start();
    let logins = provider.logins("secret");
    assert_eq!(
        logins.methods(),
        [LoginMethod::Provider {
            name: "fake".into(),
            label: "Fake".into()
        }]
    );
    assert!(logins.begin("google").is_err());
    let (state, page) = logins.begin("fake").unwrap();
    let url = url::Url::parse(&page).unwrap();
    assert!(page.starts_with(&format!("{}/authorize?", provider.base)));
    let asked = |name: &str| {
        let found = url.query_pairs().find(|(key, _)| key == name);
        found.map(|(_, value)| value.into_owned())
    };
    assert_eq!(asked("client_id").as_deref(), Some("fake id"));
    assert_eq!(asked("scope").as_deref(), Some("openid email"));
    assert_eq!(asked("state"), Some(state.clone()));
    assert_eq!(
        asked("redirect_uri").as_deref(),
        Some("http://localhost:24690/login/callback")
    );
    assert_eq!(asked("code_challenge_method").as_deref(), Some("S256"));

    assert_eq!(come_back(&logins, &page, "good"), StatusCode::OK);
    let finished = logins.finished();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].state, state);
    assert_eq!(finished[0].provider, "fake");
    assert_eq!(finished[0].email.as_deref(), Ok(" Someone@Example.com"));
    // The PKCE verifier sent is the one the challenge was made from.
    let verifier = provider.verifiers.lock().unwrap()[0].clone();
    let challenge = URL_SAFE_NO_PAD.encode(digest(&SHA256, verifier.as_bytes()));
    assert_eq!(asked("code_challenge"), Some(challenge));

    // A state works once.
    assert_eq!(come_back(&logins, &page, "good"), StatusCode::BAD_REQUEST);
    assert!(logins.finished().is_empty());
}

#[test]
fn a_login_that_goes_wrong_comes_back_with_why() {
    let provider = FakeProvider::start();
    let logins = provider.logins("secret");
    for (code, query) in [
        ("unverified", None),
        ("unknown", None),
        ("", Some("error=access_denied")),
    ] {
        let (state, page) = logins.begin("fake").unwrap();
        let status = match query {
            Some(query) => come_back_with(&logins, &format!("state={state}&{query}")),
            None => come_back(&logins, &page, code),
        };
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let finished = logins.finished();
        assert!(finished[0].email.is_err(), "{code}");
    }
    assert_eq!(
        come_back_with(&logins, "state=made-up&code=good"),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(come_back_with(&logins, ""), StatusCode::BAD_REQUEST);
    // Given up on, it can't be finished.
    let (state, _) = logins.begin("fake").unwrap();
    logins.forget(&state);
    let query = format!("state={state}&code=good");
    assert_eq!(come_back_with(&logins, &query), StatusCode::BAD_REQUEST);
}

#[test]
fn a_provider_that_cant_be_reached_is_left_out() {
    let closed = TcpListener::bind("127.0.0.1:0").unwrap();
    let issuer = format!("http://127.0.0.1:{}", closed.local_addr().unwrap().port());
    drop(closed);
    let settings = ProviderSettings {
        issuer,
        ..FakeProvider::start().settings()
    };
    let callback = "http://localhost/login/callback".to_owned();
    assert!(Logins::start(vec![settings], callback, LoginSecret::new("s")).is_none());
}
