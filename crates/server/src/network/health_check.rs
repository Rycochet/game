//! Asking a running server whether it is well (`--health`, Docker's
//! HEALTHCHECK): one plain HTTP request to its plain port, with nothing but the
//! standard library, so an image needs no curl or wget.

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::time::Duration;

use super::HEALTH_PATH;

/// How long to wait for the server to answer.
const ANSWER_WITHIN: Duration = Duration::from_secs(5);

/// Asks the server listening on `address` (its plain port) for `HEALTH_PATH`:
/// `Ok` if it answers 200, otherwise what went wrong. An address listening
/// everywhere (`0.0.0.0`, `[::]`) is asked on this machine.
pub fn check_health(address: SocketAddr) -> Result<(), String> {
    let mut address = address;
    if address.ip().is_unspecified() {
        address.set_ip(match address.ip() {
            IpAddr::V4(_) => Ipv4Addr::LOCALHOST.into(),
            IpAddr::V6(_) => Ipv6Addr::LOCALHOST.into(),
        });
    }
    let mut stream = TcpStream::connect_timeout(&address, ANSWER_WITHIN)
        .map_err(|error| format!("Can't reach {address}: {error}"))?;
    let _ = stream.set_read_timeout(Some(ANSWER_WITHIN));
    let _ = stream.set_write_timeout(Some(ANSWER_WITHIN));
    write!(
        stream,
        "GET {HEALTH_PATH} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .map_err(|error| format!("Can't ask {address}: {error}"))?;
    let mut answer = String::new();
    // The status line is all that matters, and comes first.
    let _ = stream.take(4096).read_to_string(&mut answer);
    let status = answer.lines().next().unwrap_or_default();
    match status.split(' ').nth(1) {
        Some("200") => Ok(()),
        Some(_) => Err(format!("Unwell: {status}")),
        None => Err(format!("No answer from {address}")),
    }
}
