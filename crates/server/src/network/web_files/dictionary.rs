//! Compression dictionaries for the wasm: browsers are told to keep it as one
//! (`Use-As-Dictionary`), and a browser that names a build it kept
//! (`Available-Dictionary`) and takes `dcz` is sent a delta from that build
//! instead of the whole file, when there is one.

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::sync::Arc;
use web_delta::Hash;

use super::WebFiles;

const USE_AS_DICTIONARY: &str = "use-as-dictionary";
const AVAILABLE_DICTIONARY: &str = "available-dictionary";
/// The answer differs by both, which caches (and Cloudflare) must know.
const VARY: &str = "accept-encoding, available-dictionary";

pub(super) async fn dictionaries(
    State(web): State<Arc<WebFiles>>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().trim_start_matches('/');
    if web.wasm() != Some(path) {
        return next.run(request).await;
    }
    let delta = takes_dcz(request.headers())
        .then(|| available(request.headers()))
        .flatten()
        .and_then(|hash| web.delta_from(&hash));
    let mut response = match delta {
        Some(delta) => Response::builder()
            .header(header::CONTENT_TYPE, "application/wasm")
            .header(header::CONTENT_ENCODING, "dcz")
            .body(Body::from(delta))
            .expect("a plain answer"),
        None => next.run(request).await,
    };
    if response.status().is_success() {
        let headers = response.headers_mut();
        headers.insert(header::VARY, HeaderValue::from_static(VARY));
        headers.insert(USE_AS_DICTIONARY, web.use_as_dictionary.clone());
    }
    response
}

/// Whether the browser takes `dcz` (`Accept-Encoding`).
fn takes_dcz(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::ACCEPT_ENCODING)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|coding| {
            let (name, weight) = coding.split_once(';').unwrap_or((coding, ""));
            name.trim().eq_ignore_ascii_case("dcz") && weight.trim().replace(' ', "") != "q=0"
        })
}

/// The hash of the build the browser has (`Available-Dictionary: :<base64>:`).
fn available(headers: &HeaderMap) -> Option<Hash> {
    let value = headers.get(AVAILABLE_DICTIONARY)?.to_str().ok()?.trim();
    let encoded = value.strip_prefix(':')?.strip_suffix(':')?;
    STANDARD.decode(encoded).ok()?.try_into().ok()
}

#[cfg(test)]
mod tests;
