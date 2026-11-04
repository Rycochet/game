//! Asking an address whether a Hall of Worlds server is there, before
//! connecting: each port worth trying is asked for its `/_version` at once,
//! and the first port in order that answers as a Hall of Worlds server wins.
//! Natively each on a thread of its own; in a browser, by `XMLHttpRequest`.

use shared::protocol::{ServerAddress, ServerVersion};
use std::sync::{Arc, Mutex};

// Split into files (`cargo xtask lint` keeps files small).
#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
mod native;

/// What one port said.
#[derive(Clone, Debug, PartialEq, Eq)]
enum PortAnswer {
    /// A Hall of Worlds server, of this version.
    Server(ServerVersion),
    /// Something answered, but not a Hall of Worlds server.
    Other,
    /// Nothing answered (or the browser refused the certificate).
    Silent,
    /// Its certificate isn't the one kept for it (native only: a browser's
    /// refusal looks like no answer).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Refused(String),
}

/// What asking an address found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProbeResult {
    /// A Hall of Worlds server on this port (which may not speak this
    /// client's protocol: see `ServerVersion::compatible`).
    Found { port: u16, version: ServerVersion },
    /// Its certificate isn't the one kept for it, and why.
    Refused(String),
    /// Something answered, but no Hall of Worlds server.
    NotAServer,
    /// Nothing answered on any port tried.
    NoAnswer,
}

/// Asking an address, under way. Dropping it forgets the answers, which still
/// arrive but go nowhere.
pub struct ServerProbe {
    ports: Vec<u16>,
    answers: Arc<Mutex<Vec<Option<PortAnswer>>>>,
}

impl ServerProbe {
    /// Asks every port in `ports` on `address`'s host; `key` is the certificate's
    /// key fingerprint kept for it, if any (native only: a browser checks
    /// certificates itself).
    pub fn start(address: &ServerAddress, ports: Vec<u16>, key: Option<String>) -> Self {
        let answers = Arc::new(Mutex::new(vec![None; ports.len()]));
        for (index, &port) in ports.iter().enumerate() {
            let answers = answers.clone();
            let done = move |answer| answers.lock().expect("never poisoned")[index] = Some(answer);
            let at = address.at_port(port);
            #[cfg(not(target_arch = "wasm32"))]
            native::ask(at, key.clone(), done);
            #[cfg(target_arch = "wasm32")]
            {
                let _ = &key;
                browser::ask(&at, done);
            }
        }
        Self { ports, answers }
    }

    /// What it found, once it knows.
    pub fn result(&self) -> Option<ProbeResult> {
        decide(&self.ports, &self.answers.lock().expect("never poisoned"))
    }
}

/// The answer, as soon as the ports so far settle it: the first port (in the
/// order tried) that is a server, once every port before it has answered.
fn decide(ports: &[u16], answers: &[Option<PortAnswer>]) -> Option<ProbeResult> {
    for (&port, answer) in ports.iter().zip(answers) {
        match answer {
            Some(PortAnswer::Server(version)) => {
                return Some(ProbeResult::Found {
                    port,
                    version: version.clone(),
                });
            }
            None => return None,
            Some(_) => {}
        }
    }
    let answered = |wanted: fn(&PortAnswer) -> bool| answers.iter().flatten().any(wanted);
    let refusal = answers.iter().flatten().find_map(|answer| match answer {
        PortAnswer::Refused(reason) => Some(reason.clone()),
        _ => None,
    });
    Some(if let Some(reason) = refusal {
        ProbeResult::Refused(reason)
    } else if answered(|answer| *answer == PortAnswer::Other) {
        ProbeResult::NotAServer
    } else {
        ProbeResult::NoAnswer
    })
}

/// What a web server's answer to `/_version` means.
fn read_answer(status: u16, body: &str) -> PortAnswer {
    match ServerVersion::from_json(body) {
        Some(version) if status == 200 => PortAnswer::Server(version),
        _ => PortAnswer::Other,
    }
}

#[cfg(test)]
mod tests;
