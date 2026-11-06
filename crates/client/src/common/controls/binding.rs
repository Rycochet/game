//! One binding: a key, with any modifiers held with it (Shift + G), or a
//! modifier on its own (Shift). Left and right modifiers count as the same.

use bevy::prelude::*;

use super::key_names::{bindable, code_name, key_from_code, key_label};

/// Ctrl, Alt, Shift and Super (Cmd on a Mac, the Windows key) held together.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mods(u8);

impl Mods {
    pub const NONE: Self = Self(0);
    pub const CTRL: Self = Self(1);
    pub const ALT: Self = Self(2);
    pub const SHIFT: Self = Self(4);
    pub const SUPER: Self = Self(8);
    /// In the order they are written.
    const EACH: [(Self, &'static str); 4] = [
        (Self::CTRL, "Ctrl"),
        (Self::ALT, "Alt"),
        (Self::SHIFT, "Shift"),
        (Self::SUPER, "Super"),
    ];

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The modifier this key is, if it is one.
    pub fn of_key(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::ControlLeft | KeyCode::ControlRight => Some(Self::CTRL),
            KeyCode::AltLeft | KeyCode::AltRight => Some(Self::ALT),
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(Self::SHIFT),
            KeyCode::SuperLeft | KeyCode::SuperRight => Some(Self::SUPER),
            _ => None,
        }
    }

    /// Those held down now.
    pub fn held(keys: &ButtonInput<KeyCode>) -> Self {
        Self::EACH
            .into_iter()
            .filter(|(each, _)| keys.any_pressed(sides(left_key(*each))))
            .fold(Self::NONE, |held, (each, _)| held.with(each))
    }

    /// Each held, as Settings writes them ("Ctrl + Shift").
    pub fn labels(self) -> Vec<&'static str> {
        Self::EACH
            .into_iter()
            .filter(|(each, _)| self.contains(*each))
            .map(|(each, _)| key_label(left_key(each)))
            .collect()
    }
}

/// The left key of a modifier.
fn left_key(each: Mods) -> KeyCode {
    match each {
        Mods::CTRL => KeyCode::ControlLeft,
        Mods::ALT => KeyCode::AltLeft,
        Mods::SHIFT => KeyCode::ShiftLeft,
        _ => KeyCode::SuperLeft,
    }
}

/// Both keys a key stands for: a modifier's left and right, or itself twice.
pub(super) fn sides(key: KeyCode) -> [KeyCode; 2] {
    match key {
        KeyCode::ControlLeft | KeyCode::ControlRight => {
            [KeyCode::ControlLeft, KeyCode::ControlRight]
        }
        KeyCode::AltLeft | KeyCode::AltRight => [KeyCode::AltLeft, KeyCode::AltRight],
        KeyCode::ShiftLeft | KeyCode::ShiftRight => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
        KeyCode::SuperLeft | KeyCode::SuperRight => [KeyCode::SuperLeft, KeyCode::SuperRight],
        key => [key, key],
    }
}

/// A key and the modifiers held with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Binding {
    /// A modifier is always its left key.
    pub key: KeyCode,
    /// Never includes `key` itself.
    pub mods: Mods,
}

impl Binding {
    /// `key` with `mods` held, the same whichever side a modifier is.
    pub fn new(key: KeyCode, mods: Mods) -> Self {
        let [key, _] = sides(key);
        let mods = Mods::of_key(key).map_or(mods, |own| mods.without(own));
        Self { key, mods }
    }

    /// Whether this key can be bound at all (Esc and Backspace can't).
    pub fn can_bind(key: KeyCode) -> bool {
        bindable(sides(key)[0])
    }

    /// Whether it is held with exactly what it needs, or more: Shift + G is
    /// G with Shift; G alone is G whatever is held.
    pub(super) fn fits(self, held: Mods) -> bool {
        let held = Mods::of_key(self.key).map_or(held, |own| held.without(own));
        held.contains(self.mods)
    }

    /// As saved: `Ctrl+Shift+KeyG`, `Shift`.
    pub fn to_name(self) -> String {
        let mut parts: Vec<&str> = Mods::EACH
            .into_iter()
            .filter(|(each, _)| self.mods.contains(*each))
            .map(|(_, name)| name)
            .collect();
        let code = code_name(self.key);
        parts.push(&code);
        parts.join("+")
    }

    /// From its saved name, if it is one.
    pub fn from_name(name: &str) -> Option<Self> {
        let mut parts: Vec<&str> = name.split('+').collect();
        let key = key_from_code(parts.pop()?)?;
        let mut mods = Mods::NONE;
        for part in parts {
            let (each, _) = Mods::EACH.into_iter().find(|(_, name)| *name == part)?;
            mods = mods.with(each);
        }
        Some(Self::new(key, mods))
    }

    /// At most three letters, for the corner of an action bar slot: "1", "F12",
    /// "S1" (Shift), "^1" (Ctrl), "A1" (Alt), "W1" (Super), "N1" (keypad).
    pub fn short_label(self) -> String {
        let marks = [
            (Mods::SHIFT, 'S'),
            (Mods::CTRL, '^'),
            (Mods::ALT, 'A'),
            (Mods::SUPER, 'W'),
        ];
        let mut short: String = marks
            .into_iter()
            .filter(|(each, _)| self.mods.contains(*each))
            .map(|(_, letter)| letter)
            .collect();
        let key = key_label(self.key);
        if let Some(number) = key.strip_prefix("Num ") {
            short.push('N');
            short.push_str(number);
        } else if key.contains(' ') {
            // "Up Arrow" is "UA": the first letter of each word.
            short.extend(key.split(' ').filter_map(|word| word.chars().next()));
        } else {
            short.push_str(key);
        }
        short.chars().take(3).collect()
    }

    /// As Settings shows it: "Shift + G", "Up Arrow".
    pub fn label(self) -> String {
        let mut parts = self.mods.labels();
        parts.push(key_label(self.key));
        parts.join(" + ")
    }
}
