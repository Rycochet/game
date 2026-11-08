//! Asking providers things over HTTPS (checked with the operating system's
//! authorities): JSON back from a plain request or a form.

use std::time::Duration;

use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Bytes;
use hyper::{Method, Request, header};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use serde_json::Value;

/// How long a provider gets to answer.
const ANSWER_WITHIN: Duration = Duration::from_secs(10);

/// The most a provider's answer may be.
const MOST_ANSWERED: usize = 256 * 1024;

/// Makes the requests. Keeps no connections between them, so it can be used
/// from any tokio runtime.
#[derive(Clone)]
pub(super) struct Http(Client<HttpsConnector<HttpConnector>, Full<Bytes>>);

impl Http {
    /// An error if the system's authorities can't be read.
    pub(super) fn new() -> Result<Self, String> {
        let connector = HttpsConnectorBuilder::new()
            .with_provider_and_platform_verifier(rustls::crypto::ring::default_provider())
            .map_err(|error| format!("Can't check websites' certificates: {error}"))?
            // `config::secure_enough` keeps plain http to this machine.
            .https_or_http()
            .enable_http1()
            .build();
        let client = Client::builder(TokioExecutor::new())
            .pool_max_idle_per_host(0)
            .build(connector);
        Ok(Self(client))
    }

    /// GETs JSON, with an access token if given.
    pub(super) async fn get_json(&self, url: &str, token: Option<&str>) -> Result<Value, String> {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header(header::ACCEPT, "application/json");
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = request
            .body(Full::default())
            .map_err(|error| error.to_string())?;
        self.json(request).await
    }

    /// POSTs a form, for JSON back.
    pub(super) async fn post_form(
        &self,
        url: &str,
        form: &[(&str, &str)],
    ) -> Result<Value, String> {
        let body = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(form)
            .finish();
        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Full::new(Bytes::from(body)))
            .map_err(|error| error.to_string())?;
        self.json(request).await
    }

    async fn json(&self, request: Request<Full<Bytes>>) -> Result<Value, String> {
        let asked = request.uri().clone();
        let answer = async {
            let response = self
                .0
                .request(request)
                .await
                .map_err(|error| error.to_string())?;
            let status = response.status();
            let body = Limited::new(response.into_body(), MOST_ANSWERED)
                .collect()
                .await
                .map_err(|error| error.to_string())?
                .to_bytes();
            if !status.is_success() {
                return Err(format!("answered {status}"));
            }
            serde_json::from_slice(&body)
                .map_err(|error| format!("answered with no JSON ({error})"))
        };
        match tokio::time::timeout(ANSWER_WITHIN, answer).await {
            Ok(result) => result.map_err(|error| format!("{asked}: {error}")),
            Err(_) => Err(format!("{asked}: no answer")),
        }
    }
}
