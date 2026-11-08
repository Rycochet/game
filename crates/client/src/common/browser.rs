//! Opening a page in the player's own browser: natively, the system's default
//! browser; on the web, a new tab.

/// Whether an address from a server may be opened: `https://`, or plain
/// `http://` on this machine (a provider tried out locally), with nothing in it
/// a command line could take another way.
pub fn safe_to_open(url: &str) -> bool {
    let local = ["http://localhost", "http://127.0.0.1", "http://[::1]"]
        .iter()
        .any(|start| {
            url.strip_prefix(start)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with([':', '/']))
        });
    let plain = url
        .chars()
        .all(|c| c.is_ascii_graphic() && !matches!(c, '"' | '\'' | '`' | '\\' | '^'));
    (url.starts_with("https://") || local) && plain
}

/// Opens `url` in the player's browser: an error says why it can't.
pub fn open_in_browser(url: &str) -> Result<(), String> {
    if !safe_to_open(url) {
        return Err(format!("won't open {url:?}: it isn't a secure web address"));
    }
    open(url)
}

/// The system's own way to open an address, each word given to it apart, so no
/// shell reads it.
#[cfg(not(target_arch = "wasm32"))]
fn open(url: &str) -> Result<(), String> {
    use std::process::{Command, Stdio};
    #[cfg(windows)]
    let mut command = {
        let mut command = Command::new("rundll32.exe");
        command.args(["url.dll,FileProtocolHandler", url]);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(url);
        command
    };
    #[cfg(not(any(windows, target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("can't open the browser: {error}"))
}

/// A new tab. A browser may block it, unless it follows a press closely enough.
#[cfg(target_arch = "wasm32")]
fn open(url: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    match window.open_with_url_and_target(url, "_blank") {
        Ok(Some(_)) => Ok(()),
        _ => Err("the browser didn't open the page".into()),
    }
}

#[cfg(test)]
mod tests;
