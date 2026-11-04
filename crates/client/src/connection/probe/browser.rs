//! Web: one port asked for `/_version` with the browser's `XMLHttpRequest`.
//! The browser checks the certificate, and a refused one looks the same as no
//! answer. Servers allow any page to ask (`Access-Control-Allow-Origin: *`).

use shared::protocol::ServerAddress;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use super::{PortAnswer, read_answer};

/// How long the browser waits, in milliseconds.
const WAIT_MS: u32 = 4000;

/// Asks `address`, handing its answer to `done` when the browser has one.
pub(super) fn ask(address: &ServerAddress, done: impl FnOnce(PortAnswer) + 'static) {
    let Ok(request) = web_sys::XmlHttpRequest::new() else {
        done(PortAnswer::Silent);
        return;
    };
    if request.open("GET", &address.version_url()).is_err() {
        done(PortAnswer::Silent);
        return;
    }
    request.set_timeout(WAIT_MS);
    let finished = request.clone();
    // Called once, for loading, failing or giving up alike (`loadend`).
    let on_end = Closure::once_into_js(move || {
        let status = finished.status().unwrap_or(0);
        let answer = if status == 0 {
            PortAnswer::Silent
        } else {
            let body = finished.response_text().ok().flatten().unwrap_or_default();
            read_answer(status, &body)
        };
        done(answer);
    });
    let _ = request.add_event_listener_with_callback("loadend", on_end.unchecked_ref());
    if request.send().is_err() {
        bevy::log::info!("Can't ask {} what it is", address.with_port());
    }
}
