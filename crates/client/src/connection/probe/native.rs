//! Native: one port asked for `/_version` over HTTPS, on a thread of its own,
//! with the same certificate checks as connecting (any certificate, unless a
//! key is kept for the server).

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rustls::pki_types::ServerName;
use shared::protocol::{ServerAddress, VERSION_PATH};
use tungstenite::Connector;

use super::super::socket::Refusal;
use super::super::trust::{KeyFrom, connector};
use super::{PortAnswer, read_answer};

/// How long connecting, and then answering, may each take.
const WAIT: Duration = Duration::from_secs(4);
/// More than any version answer needs.
const MOST_READ: u64 = 16 * 1024;

/// Asks `address` on a thread, handing its answer to `done`.
pub(super) fn ask(
    address: ServerAddress,
    key: Option<String>,
    done: impl FnOnce(PortAnswer) + Send + 'static,
) {
    let shown = address.with_port();
    let spawned = thread::Builder::new()
        .name("probe".into())
        .spawn(move || done(answer_of(&address, key)));
    if let Err(error) = spawned {
        bevy::log::warn!("Can't ask {shown} what it is: {error}");
    }
}

fn answer_of(address: &ServerAddress, key: Option<String>) -> PortAnswer {
    let refusal = Refusal::default();
    let key = key.map(|key| (key, KeyFrom::Pasted));
    let Ok(Connector::Rustls(config)) =
        connector(&address.host, address.port, key, refusal.clone())
    else {
        return PortAnswer::Silent;
    };
    let answer = ask_over_tls(address, config);
    match (refusal.take(), answer) {
        (Some(reason), _) => PortAnswer::Refused(reason),
        (None, Some((status, body))) => read_answer(status, &body),
        (None, None) => PortAnswer::Silent,
    }
}

/// The status and body of `/_version`, if the server answered at all.
fn ask_over_tls(
    address: &ServerAddress,
    config: Arc<rustls::ClientConfig>,
) -> Option<(u16, String)> {
    let at = (address.host.as_str(), address.port)
        .to_socket_addrs()
        .ok()?
        .next()?;
    let stream = TcpStream::connect_timeout(&at, WAIT).ok()?;
    stream.set_read_timeout(Some(WAIT)).ok()?;
    stream.set_write_timeout(Some(WAIT)).ok()?;
    let name = ServerName::try_from(address.host.clone()).ok()?;
    let tls = rustls::ClientConnection::new(config, name).ok()?;
    let mut stream = rustls::StreamOwned::new(tls, stream);
    let host = address.with_port();
    write!(
        stream,
        "GET {VERSION_PATH} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )
    .ok()?;
    let mut bytes = Vec::new();
    // A server closing without TLS's goodbye still said what it had to.
    let _ = stream.take(MOST_READ).read_to_end(&mut bytes);
    let answer = String::from_utf8_lossy(&bytes);
    let (head, body) = answer.split_once("\r\n\r\n")?;
    let status = head.split(' ').nth(1)?.parse().ok()?;
    Some((status, body.to_owned()))
}
