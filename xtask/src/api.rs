//! `cargo xtask api`: calls a method of a running game's debug API (Bevy's
//! remote protocol: JSON-RPC over HTTP on localhost) and prints the answer.
//! Plain `std`, like the rest of xtask, so it needs nothing installed.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

// Split into files (`cargo xtask lint` keeps files small).
mod http;
mod json;
mod request;
mod temporary;
mod wait;

use http::call;
#[cfg(test)]
use http::unchunk;
use json::{number_field, pretty, string_field};
use request::Request;
#[cfg(test)]
use request::quote;
pub use temporary::temporary_methods;
use wait::{wait_for_file, wait_for_input};

/// The client's port (and a standalone server's); the native client's
/// embedded server is on the next one. Matches `server::debug_api`.
const CLIENT_PORT: u16 = 15702;

/// How long to wait for a file a method says is coming (a screenshot).
const FILE_WAIT: Duration = Duration::from_secs(15);

/// How long to wait for queued fake input to be played (`client.wait` can
/// ask for many frames).
const INPUT_WAIT: Duration = Duration::from_secs(120);

pub const USAGE: &str = "\
  api     Call a running game's debug API and print the answer:
            cargo xtask api [--server | --port N] <method> [params]
          params are key=value pairs (a value that isn't JSON is a string),
          or one JSON object. --server asks the native client's embedded
          server (the port after the client's). rpc.discover lists methods.
          The game must be built with `--features debug-api` and started
          with HOW_DEBUG_API=true. See docs/context/debug-api.md.";

pub fn run(args: &[String]) -> ExitCode {
    let request = match Request::parse(args) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("{error}\n\nUsage:\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    let body = match call(request.port, &request.body()) {
        Ok(body) => body,
        Err(error) => {
            eprintln!(
                "==> Nothing answered on port {} ({error}). Is the game running, built with \
                 `--features debug-api` and started with HOW_DEBUG_API=true?",
                request.port
            );
            return ExitCode::FAILURE;
        }
    };
    println!("{}", pretty(&body));
    if let Some(file) = string_field(&body, "wait_for_file") {
        let file = PathBuf::from(file);
        if !wait_for_file(&file) {
            eprintln!("==> {} didn't appear", file.display());
            return ExitCode::FAILURE;
        }
        println!("==> Saved {}", file.display());
    }
    if body.contains("\"wait_for_input\":true") {
        if !wait_for_input(request.port) {
            eprintln!("==> The input queued wasn't all played");
            return ExitCode::FAILURE;
        }
        println!("==> Played");
    }
    if body.contains("\"error\":") {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests;
