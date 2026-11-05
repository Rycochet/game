//! The web client's files: the brotli copy where there is one (made by
//! `cargo xtask compress-web`) for browsers that take it, and how long
//! browsers may keep each file.

use axum::Router;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::Response;
use std::path::PathBuf;
use tower_http::services::ServeDir;

/// Files whose names carry a hash of what is in them never change, so
/// browsers (and proxies such as Cloudflare) keep them for a year.
const KEEP: &str = "public, max-age=31536000, immutable";
/// Everything else is checked with the server each time it is used, so a
/// new version is seen at once; the check is small when nothing changed.
const CHECK: &str = "no-cache";

/// Serves the files in `dir`.
pub(super) fn web_files(dir: PathBuf) -> Router {
    Router::new()
        .fallback_service(ServeDir::new(dir).precompressed_br())
        .layer(middleware::from_fn(cache_control))
}

/// Says how long the answer may be kept.
async fn cache_control(request: Request, next: Next) -> Response {
    let keep = how_long_to_keep(request.uri().path());
    let mut response = next.run(request).await;
    if response.status().is_success() || response.status() == StatusCode::NOT_MODIFIED {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static(keep));
    }
    response
}

/// `KEEP` for a file Trunk named by its hash (`hall-of-worlds-<hash>.js`,
/// `hall-of-worlds-<hash>_bg.wasm`), otherwise `CHECK`.
fn how_long_to_keep(path: &str) -> &'static str {
    let name = path.rsplit('/').next().unwrap_or(path);
    let hashed = name
        .split(['-', '_', '.'])
        .any(|part| part.len() >= 16 && part.bytes().all(|byte| byte.is_ascii_hexdigit()));
    if hashed { KEEP } else { CHECK }
}

#[cfg(test)]
mod tests;
