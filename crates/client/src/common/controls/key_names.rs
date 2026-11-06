//! The keys that can be bound, by their saved names (Bevy's `KeyCode`s, which
//! are places on the keyboard, not letters) and as Settings shows them.

use bevy::prelude::*;

/// Super's name: what the key is called on this kind of computer.
#[cfg(target_os = "macos")]
const SUPER: &str = "Cmd";
#[cfg(target_os = "windows")]
const SUPER: &str = "Win";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const SUPER: &str = "Super";
#[cfg(target_os = "macos")]
const ALT: &str = "Option";
#[cfg(not(target_os = "macos"))]
const ALT: &str = "Alt";

/// Every key that can be bound, with its label. Modifiers are their left keys
/// (the right count as the same). Esc and Backspace aren't here: in a key box
/// they cancel and clear.
const KEYS: &[(KeyCode, &str)] = &[
    (KeyCode::KeyA, "A"),
    (KeyCode::KeyB, "B"),
    (KeyCode::KeyC, "C"),
    (KeyCode::KeyD, "D"),
    (KeyCode::KeyE, "E"),
    (KeyCode::KeyF, "F"),
    (KeyCode::KeyG, "G"),
    (KeyCode::KeyH, "H"),
    (KeyCode::KeyI, "I"),
    (KeyCode::KeyJ, "J"),
    (KeyCode::KeyK, "K"),
    (KeyCode::KeyL, "L"),
    (KeyCode::KeyM, "M"),
    (KeyCode::KeyN, "N"),
    (KeyCode::KeyO, "O"),
    (KeyCode::KeyP, "P"),
    (KeyCode::KeyQ, "Q"),
    (KeyCode::KeyR, "R"),
    (KeyCode::KeyS, "S"),
    (KeyCode::KeyT, "T"),
    (KeyCode::KeyU, "U"),
    (KeyCode::KeyV, "V"),
    (KeyCode::KeyW, "W"),
    (KeyCode::KeyX, "X"),
    (KeyCode::KeyY, "Y"),
    (KeyCode::KeyZ, "Z"),
    (KeyCode::Digit0, "0"),
    (KeyCode::Digit1, "1"),
    (KeyCode::Digit2, "2"),
    (KeyCode::Digit3, "3"),
    (KeyCode::Digit4, "4"),
    (KeyCode::Digit5, "5"),
    (KeyCode::Digit6, "6"),
    (KeyCode::Digit7, "7"),
    (KeyCode::Digit8, "8"),
    (KeyCode::Digit9, "9"),
    (KeyCode::F1, "F1"),
    (KeyCode::F2, "F2"),
    (KeyCode::F3, "F3"),
    (KeyCode::F4, "F4"),
    (KeyCode::F5, "F5"),
    (KeyCode::F6, "F6"),
    (KeyCode::F7, "F7"),
    (KeyCode::F8, "F8"),
    (KeyCode::F9, "F9"),
    (KeyCode::F10, "F10"),
    (KeyCode::F11, "F11"),
    (KeyCode::F12, "F12"),
    (KeyCode::ArrowUp, "Up Arrow"),
    (KeyCode::ArrowDown, "Down Arrow"),
    (KeyCode::ArrowLeft, "Left Arrow"),
    (KeyCode::ArrowRight, "Right Arrow"),
    (KeyCode::Home, "Home"),
    (KeyCode::End, "End"),
    (KeyCode::PageUp, "Page Up"),
    (KeyCode::PageDown, "Page Down"),
    (KeyCode::Insert, "Insert"),
    (KeyCode::Delete, "Delete"),
    (KeyCode::Enter, "Enter"),
    (KeyCode::Tab, "Tab"),
    (KeyCode::Space, "Space"),
    (KeyCode::Backquote, "`"),
    (KeyCode::Minus, "-"),
    (KeyCode::Equal, "="),
    (KeyCode::BracketLeft, "["),
    (KeyCode::BracketRight, "]"),
    (KeyCode::Backslash, "\\"),
    (KeyCode::IntlBackslash, "Intl \\"),
    (KeyCode::Semicolon, ";"),
    (KeyCode::Quote, "'"),
    (KeyCode::Comma, ","),
    (KeyCode::Period, "."),
    (KeyCode::Slash, "/"),
    (KeyCode::Numpad0, "Num 0"),
    (KeyCode::Numpad1, "Num 1"),
    (KeyCode::Numpad2, "Num 2"),
    (KeyCode::Numpad3, "Num 3"),
    (KeyCode::Numpad4, "Num 4"),
    (KeyCode::Numpad5, "Num 5"),
    (KeyCode::Numpad6, "Num 6"),
    (KeyCode::Numpad7, "Num 7"),
    (KeyCode::Numpad8, "Num 8"),
    (KeyCode::Numpad9, "Num 9"),
    (KeyCode::NumpadAdd, "Num +"),
    (KeyCode::NumpadSubtract, "Num -"),
    (KeyCode::NumpadMultiply, "Num *"),
    (KeyCode::NumpadDivide, "Num /"),
    (KeyCode::NumpadDecimal, "Num ."),
    (KeyCode::NumpadEnter, "Num Enter"),
    (KeyCode::ControlLeft, "Ctrl"),
    (KeyCode::AltLeft, ALT),
    (KeyCode::ShiftLeft, "Shift"),
    (KeyCode::SuperLeft, SUPER),
    (KeyCode::CapsLock, "Caps Lock"),
    (KeyCode::ContextMenu, "Menu"),
    (KeyCode::PrintScreen, "Print Screen"),
    (KeyCode::ScrollLock, "Scroll Lock"),
    (KeyCode::Pause, "Pause"),
];

/// Saved names of the modifiers, which are written without their side.
const MODIFIER_CODES: [(KeyCode, &str); 4] = [
    (KeyCode::ControlLeft, "Ctrl"),
    (KeyCode::AltLeft, "Alt"),
    (KeyCode::ShiftLeft, "Shift"),
    (KeyCode::SuperLeft, "Super"),
];

/// Whether a key can be bound (after a right modifier is made its left).
pub(super) fn bindable(key: KeyCode) -> bool {
    KEYS.iter().any(|(each, _)| *each == key)
}

/// Its saved name: `KeyG`, `ArrowUp`, `Shift`.
pub(super) fn code_name(key: KeyCode) -> String {
    MODIFIER_CODES
        .iter()
        .find(|(each, _)| *each == key)
        .map_or_else(|| format!("{key:?}"), |(_, name)| (*name).to_string())
}

/// The key a saved name is, if it can be bound.
pub(super) fn key_from_code(name: &str) -> Option<KeyCode> {
    MODIFIER_CODES
        .iter()
        .find(|(_, each)| *each == name)
        .map(|(key, _)| *key)
        .or_else(|| {
            KEYS.iter()
                .map(|(key, _)| *key)
                .find(|key| format!("{key:?}") == name)
        })
}

/// What Settings calls it.
pub(super) fn key_label(key: KeyCode) -> &'static str {
    match key {
        KeyCode::Escape => "Esc",
        _ => KEYS
            .iter()
            .find(|(each, _)| *each == key)
            .map_or("?", |(_, label)| label),
    }
}
