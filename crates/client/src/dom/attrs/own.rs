//! The attributes only some elements take, and what each may be. Which
//! element takes which is `Tag::own_attrs`.

use super::values::{count, length, number, spacing};

/// What an attribute's value has to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Text,
    /// A number that isn't negative.
    Number,
    /// Pixels (`12`) or a share (`50%`).
    Length,
    /// A multiple of 4.
    Spacing,
    /// A whole number from 1.
    Count,
    /// Standing alone: there or not.
    Flag,
    OneOf(&'static [&'static str]),
}

pub(super) const OWN: [(&str, Kind); 38] = [
    ("for", Kind::Text),
    ("src", Kind::Text),
    ("alt", Kind::Text),
    ("value", Kind::Text),
    ("label", Kind::Text),
    ("prefix", Kind::Text),
    ("suffix", Kind::Text),
    ("min", Kind::Number),
    ("max", Kind::Number),
    ("step", Kind::Number),
    ("colspan", Kind::Count),
    ("cellspacing", Kind::Spacing),
    ("cellpadding", Kind::Spacing),
    // An `input`'s, then an `object`'s (`check` keeps each to its own).
    (
        "type",
        Kind::OneOf(&["text", "checkbox", "range", "item", "ability"]),
    ),
    ("inputmode", Kind::OneOf(&["numeric"])),
    ("checked", Kind::Flag),
    ("readonly", Kind::Flag),
    ("selected", Kind::Flag),
    ("switch", Kind::Flag),
    ("searchable", Kind::Flag),
    ("hold", Kind::Flag),
    ("accepts-items", Kind::Flag),
    ("moveable", Kind::Flag),
    ("grip", Kind::Flag),
    ("modal", Kind::Flag),
    ("closable", Kind::Flag),
    ("auto", Kind::Flag),
    ("depth", Kind::Number),
    ("max-height", Kind::Length),
    ("scrollbars", Kind::OneOf(&["auto", "always"])),
    // A grid of things: how big, whose places they are, and what it takes.
    ("columns", Kind::Count),
    ("rows", Kind::Count),
    ("grows", Kind::Flag),
    ("places", Kind::OneOf(&["kept", "packed"])),
    ("stacks", Kind::OneOf(&["true", "false"])),
    ("compact", Kind::Flag),
    // A slot: the types it takes (`item`, `ability` or both), and its key.
    ("accepts", Kind::Text),
    ("key", Kind::Text),
];

/// The name as the table has it (so it can be kept without copying), and
/// what its value has to be.
pub(super) fn known(name: &str) -> Option<(&'static str, Kind)> {
    OWN.iter().find(|(known, _)| *known == name).copied()
}

impl Kind {
    /// Whether `value` (`None`: standing alone) will do for `name`.
    pub(super) fn check(self, name: &str, value: Option<&str>) -> Result<(), String> {
        let Some(value) = value else {
            return match self {
                Self::Flag => Ok(()),
                _ => Err(format!("`{name}` needs a value")),
            };
        };
        match self {
            Self::Text => Ok(()),
            Self::Number => number(name, value).map(|_| ()),
            Self::Length => length(name, value).map(|_| ()),
            Self::Spacing => spacing(name, value).map(|_| ()),
            Self::Count => count(name, value).map(|_| ()),
            // `checked`, `checked=""` and `checked="checked"` all mean yes.
            Self::Flag if value.is_empty() || value == name => Ok(()),
            Self::Flag => Err(format!("`{name}` stands alone, without `{value}`")),
            Self::OneOf(allowed) if allowed.contains(&value) => Ok(()),
            Self::OneOf(allowed) => {
                let allowed = allowed.join("`, `");
                Err(format!("`{name}` is `{allowed}`, not `{value}`"))
            }
        }
    }
}
