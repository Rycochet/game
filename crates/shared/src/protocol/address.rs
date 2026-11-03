//! A server's address as players type it: a host name or IP address and an
//! optional port, with no `wss://` or path.

use std::fmt;

/// The port a server's address means when it names none: the secure port a
/// server listens on unless told otherwise.
pub const DEFAULT_PORT: u16 = 8081;

/// Where a server is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerAddress {
    /// A host name, an IPv4 address, or an IPv6 address (without brackets).
    pub host: String,
    pub port: u16,
}

impl ServerAddress {
    /// What was typed, if it is an address: `host`, `host:port`, `1.2.3.4`,
    /// `[::1]:9000`. Spaces round it are ignored; anything else is not.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let (host, port) = match text.strip_prefix('[') {
            // An IPv6 address, which has colons of its own.
            Some(rest) => {
                let (host, after) = rest.split_once(']')?;
                let ipv6 =
                    host.contains(':') && host.chars().all(|c| c.is_ascii_hexdigit() || c == ':');
                if !ipv6 {
                    return None;
                }
                match after {
                    "" => (host, None),
                    _ => (host, Some(after.strip_prefix(':')?)),
                }
            }
            None => {
                let (host, port) = match text.split_once(':') {
                    Some((host, port)) => (host, Some(port)),
                    None => (text, None),
                };
                if !is_host_name(host) {
                    return None;
                }
                (host, port)
            }
        };
        let port = match port {
            None => DEFAULT_PORT,
            Some(port) => port.parse::<u16>().ok().filter(|&port| port != 0)?,
        };
        Some(Self {
            host: host.to_lowercase(),
            port,
        })
    }

    /// The server's WebSocket.
    pub fn url(&self) -> String {
        format!(
            "wss://{}:{}{}",
            self.bracketed_host(),
            self.port,
            super::SOCKET_PATH
        )
    }

    /// The host, with an IPv6 address in brackets as URLs need.
    fn bracketed_host(&self) -> String {
        if self.host.contains(':') {
            format!("[{}]", self.host)
        } else {
            self.host.clone()
        }
    }
}

/// Shown as typed, with the port only when it isn't the usual one.
impl fmt::Display for ServerAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let host = self.bracketed_host();
        if self.port == DEFAULT_PORT {
            write!(f, "{host}")
        } else {
            write!(f, "{host}:{}", self.port)
        }
    }
}

/// A host name (or IPv4 address): dot-separated labels of letters, digits and
/// hyphens, none starting or ending with a hyphen.
fn is_host_name(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
}

#[cfg(test)]
mod tests;
