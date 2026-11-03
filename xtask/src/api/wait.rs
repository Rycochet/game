//! Waiting for fake input to be played, or for a file to be written.

use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use super::{FILE_WAIT, INPUT_WAIT, Request, call, number_field};

/// Asks `client.input` until nothing queued is left to play.
pub(super) fn wait_for_input(port: u16) -> bool {
    let ask = Request {
        port,
        method: "client.input".into(),
        params: None,
    }
    .body();
    let deadline = Instant::now() + INPUT_WAIT;
    while Instant::now() < deadline {
        match call(port, &ask).map(|body| number_field(&body, "pending")) {
            Ok(Some(0)) => return true,
            Ok(Some(_)) => thread::sleep(Duration::from_millis(50)),
            _ => return false,
        }
    }
    false
}

/// Waits until `file` exists and has stopped growing.
pub(super) fn wait_for_file(file: &Path) -> bool {
    let deadline = Instant::now() + FILE_WAIT;
    let mut last_size = None;
    while Instant::now() < deadline {
        let size = fs::metadata(file).ok().map(|metadata| metadata.len());
        if size.is_some_and(|size| size > 0) && size == last_size {
            return true;
        }
        last_size = size;
        thread::sleep(Duration::from_millis(200));
    }
    false
}
