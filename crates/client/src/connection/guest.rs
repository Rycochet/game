//! The token this client logs in with as a guest (`ClientMessage::Login`), kept
//! so it gets the same character back. There is one per server, so a server
//! can't use it on another.
//!
//! - Native: in `guest_tokens.ron` beside the known servers, by `host:port`.
//!   `HOW_DEBUG_GUEST` names a slot, so that clients on one computer can be
//!   different guests.
//! - Web: in the browser's local storage, which is already per server.

use shared::protocol::GuestToken;

use super::socket::Target;

/// This client's token for `target`, made now if it has none. An error if a
/// safe one can't be made.
#[cfg(target_arch = "wasm32")]
pub(super) fn token(_target: &Target) -> Result<GuestToken, String> {
    const KEY: &str = "hall-of-worlds-guest-token";
    let window = web_sys::window().ok_or("no window")?;
    let storage = window.local_storage().ok().flatten();
    let saved = storage
        .as_ref()
        .and_then(|storage| storage.get_item(KEY).ok().flatten())
        .and_then(|text| GuestToken::from_hex(&text));
    if let Some(token) = saved {
        return Ok(token);
    }
    let mut bytes = [0; 32];
    window
        .crypto()
        .and_then(|crypto| crypto.get_random_values_with_u8_array(&mut bytes))
        .map_err(|_| "This browser can't provide secure random numbers".to_owned())?;
    let token = GuestToken(bytes);
    if let Some(storage) = storage {
        let _ = storage.set_item(KEY, &token.to_hex());
    }
    Ok(token)
}

/// This client's token for `target`, made now if it has none. An error if a
/// safe one can't be made.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn token(target: &Target) -> Result<GuestToken, String> {
    let Target::Url(url) = target else {
        // An embedded server has no logins, so this doesn't get used.
        return server::new_guest_token();
    };
    let slot = std::env::var("HOW_DEBUG_GUEST").ok();
    native::token(
        &native::key(url, slot.as_deref()),
        native::file().as_deref(),
    )
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::{fs, io};

    use bevy::log::warn;
    use shared::protocol::GuestToken;

    use crate::connection::trust::{KnownServers, write_through_new};

    /// Where the tokens are kept: secrets, so not in the known servers' file,
    /// which people are told to edit.
    pub fn file() -> Option<PathBuf> {
        Some(KnownServers::path()?.with_file_name("guest_tokens.ron"))
    }

    /// What a server's token is kept under.
    pub fn key(url: &str, slot: Option<&str>) -> String {
        let uri = url.parse::<tungstenite::http::Uri>().ok();
        let uri = uri.as_ref();
        let host = uri.and_then(|uri| uri.host()).unwrap_or(url);
        let secure = uri.is_some_and(|uri| uri.scheme_str() == Some("wss"));
        let port = uri.and_then(|uri| uri.port_u16());
        let port = port.unwrap_or(if secure { 443 } else { 80 });
        match slot {
            Some(slot) => format!("{host}:{port}#{slot}"),
            None => format!("{host}:{port}"),
        }
    }

    /// The token kept under `key` in `file`, or a new one, kept if it can be. A
    /// file that can't be read is left alone rather than lose the tokens in it:
    /// this run plays a new guest.
    pub fn token(key: &str, file: Option<&Path>) -> Result<GuestToken, String> {
        let Some(file) = file else {
            return server::new_guest_token();
        };
        let mut tokens = match read(file) {
            Ok(tokens) => tokens,
            Err(error) => {
                warn!("Can't read {}: {error}", file.display());
                return server::new_guest_token();
            }
        };
        if let Some(token) = tokens.get(key).and_then(|text| GuestToken::from_hex(text)) {
            return Ok(token);
        }
        let token = server::new_guest_token()?;
        tokens.insert(key.to_owned(), token.to_hex());
        if let Err(error) = write(file, &tokens) {
            warn!("Can't keep the guest token in {}: {error}", file.display());
        }
        Ok(token)
    }

    fn read(file: &Path) -> io::Result<BTreeMap<String, String>> {
        match fs::read_to_string(file) {
            Ok(text) => ron::from_str(&text).map_err(io::Error::other),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(error) => Err(error),
        }
    }

    fn write(file: &Path, tokens: &BTreeMap<String, String>) -> io::Result<()> {
        let pretty = ron::ser::PrettyConfig::default();
        let text = ron::ser::to_string_pretty(tokens, pretty).map_err(io::Error::other)?;
        write_through_new(file, &text)?;
        // Only this user's, where a system can say so.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(file, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn each_server_and_slot_has_its_own_key() {
            assert_eq!(key("wss://example.org/ws", None), "example.org:443");
            assert_eq!(key("ws://example.org:8080/ws", None), "example.org:8080");
            assert_eq!(key("wss://a:1/ws", Some("two")), "a:1#two");
        }

        #[test]
        fn a_token_is_kept_for_its_server() {
            let folder = std::env::temp_dir()
                .join(format!("hall-of-worlds-guest-test-{}", std::process::id()));
            let file = folder.join("guest_tokens.ron");
            let first = token("a:1", Some(&file)).unwrap();
            let second = token("b:1", Some(&file)).unwrap();
            assert_ne!(first, second);
            assert_eq!(token("a:1", Some(&file)).unwrap(), first);

            // A file that can't be read isn't overwritten.
            fs::write(&file, "not tokens").unwrap();
            assert!(token("a:1", Some(&file)).is_ok());
            assert_eq!(fs::read_to_string(&file).unwrap(), "not tokens");
            let _ = fs::remove_dir_all(&folder);
        }
    }
}
