//! The system clipboard: what Ctrl+V pastes into a text box, and where Ctrl+C,
//! Ctrl+X and a Copy button put text. Natively through `arboard`; in a browser
//! through its own clipboard, which answers a paste a frame or more later
//! (`arrived`), and may ask the player first.

/// What the clipboard holds as text, if anything, straight away. In a browser
/// this asks for it, and it comes from `arrived`.
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

/// Text asked for by `paste` that has come since: only ever in a browser.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn arrived() -> Option<String> {
    None
}

/// Puts this text on the clipboard; `false` if it couldn't.
#[cfg(not(target_arch = "wasm32"))]
pub fn copy_to_clipboard(text: &str) -> bool {
    match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text)) {
        Ok(()) => true,
        Err(error) => {
            bevy::log::warn!("Can't copy to the clipboard: {error}");
            false
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) use browser::{arrived, copy_to_clipboard, paste};

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::RefCell;

    use wasm_bindgen::JsValue;
    use wasm_bindgen::closure::Closure;

    thread_local! {
        static ARRIVED: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    fn clipboard() -> Option<web_sys::Clipboard> {
        Some(web_sys::window()?.navigator().clipboard())
    }

    pub fn paste() -> Option<String> {
        let clipboard = clipboard()?;
        let take = Closure::once(move |text: JsValue| {
            if let Some(text) = text.as_string() {
                ARRIVED.with(|arrived| *arrived.borrow_mut() = Some(text));
            }
        });
        let refused = Closure::once(move |error: JsValue| {
            bevy::log::debug!("Nothing to paste: {error:?}");
        });
        let _ = clipboard.read_text().then2(&take, &refused);
        // (They last until the browser answers.)
        take.forget();
        refused.forget();
        None
    }

    pub fn arrived() -> Option<String> {
        ARRIVED.with(|arrived| arrived.borrow_mut().take())
    }

    pub fn copy_to_clipboard(text: &str) -> bool {
        clipboard().is_some_and(|clipboard| {
            let _ = clipboard.write_text(text);
            true
        })
    }
}
