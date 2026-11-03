//! The system clipboard: what Ctrl+V pastes into a text box, and where a Copy
//! button puts its text. Native only; in a browser the page's hidden input
//! takes the browser's own paste.

/// What the clipboard holds as text, if anything.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn paste() -> Option<String> {
    match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) {
        Ok(text) => Some(text),
        Err(error) => {
            bevy::log::debug!("Nothing to paste: {error}");
            None
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn paste() -> Option<String> {
    None
}
