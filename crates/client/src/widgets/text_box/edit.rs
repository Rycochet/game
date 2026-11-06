//! Editing what a box says: where the caret is, what is selected, and what keys
//! and clicks do to them. Plain Rust, so it can be tested without Bevy. Places
//! are counted in units (`units`), not bytes: characters, and items written in
//! as one each in a box that takes them. The caret sits before the unit with its
//! number, and after the last at the text's length.

use super::units::{Unit, byte_of_place, place_of_byte, units};

/// Where the caret goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::widgets) enum Motion {
    Left,
    Right,
    /// To the start of this word, or the one before.
    WordLeft,
    /// Past this word and the spaces after it.
    WordRight,
    Home,
    End,
    /// A place, as a click finds it.
    To(usize),
}

/// A box's writing with its caret and selection. The selection runs between
/// `anchor` (where it began) and `caret`; with both the same, nothing is selected.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::widgets) struct Edited {
    pub text: String,
    pub caret: usize,
    pub anchor: usize,
    /// Items written in count as one place each (a box that takes items).
    pub items: bool,
}

impl Edited {
    /// The caret at the end, nothing selected.
    pub fn at_end(text: &str, items: bool) -> Self {
        let end = units(text, items).len();
        Self {
            text: text.to_owned(),
            caret: end,
            anchor: end,
            items,
        }
    }

    pub fn units(&self) -> Vec<Unit> {
        units(&self.text, self.items)
    }

    pub fn len(&self) -> usize {
        self.units().len()
    }

    /// Where the selection starts and ends, if there is one.
    pub fn selection(&self) -> Option<(usize, usize)> {
        (self.caret != self.anchor)
            .then(|| (self.caret.min(self.anchor), self.caret.max(self.anchor)))
    }

    /// What is selected; empty with nothing selected.
    pub fn selected(&self) -> String {
        self.selection().map_or_else(String::new, |(from, to)| {
            self.text[self.byte(from)..self.byte(to)].to_owned()
        })
    }

    /// Where the unit numbered `place` starts, in bytes.
    pub fn byte(&self, place: usize) -> usize {
        byte_of_place(&self.units(), place, self.text.len())
    }

    /// The place at this byte (inside an item: after it).
    pub fn place_of_byte(&self, byte: usize) -> usize {
        place_of_byte(&self.units(), byte)
    }

    /// Each unit's sort, for moving by words.
    fn chars(&self) -> Vec<u8> {
        self.units().into_iter().map(Unit::sort).collect()
    }

    /// Where a motion from the caret ends.
    fn target(&self, motion: Motion) -> usize {
        let chars = self.chars();
        let mut at = self.caret.min(chars.len());
        match motion {
            Motion::Left => at.saturating_sub(1),
            Motion::Right => (at + 1).min(chars.len()),
            Motion::Home => 0,
            Motion::End => chars.len(),
            Motion::To(place) => place.min(chars.len()),
            Motion::WordLeft => {
                while at > 0 && chars[at - 1] == 0 {
                    at -= 1;
                }
                if let Some(&word) = at.checked_sub(1).and_then(|i| chars.get(i)) {
                    while at > 0 && chars[at - 1] == word {
                        at -= 1;
                    }
                }
                at
            }
            Motion::WordRight => {
                if let Some(&word) = chars.get(at) {
                    while at < chars.len() && chars[at] == word && word != 0 {
                        at += 1;
                    }
                }
                while at < chars.len() && chars[at] == 0 {
                    at += 1;
                }
                at
            }
        }
    }

    /// Moves the caret; `select` keeps the anchor where it was, so the
    /// selection grows or shrinks. Without it, Left and Right with something
    /// selected go to the selection's start or end.
    pub fn move_caret(&mut self, motion: Motion, select: bool) {
        let to = match (self.selection(), motion, select) {
            (Some((from, _)), Motion::Left, false) => from,
            (Some((_, to)), Motion::Right, false) => to,
            _ => self.target(motion),
        };
        self.caret = to;
        if !select {
            self.anchor = to;
        }
    }

    /// Takes out what is selected, leaving the caret where it was; `false` if
    /// nothing was.
    fn remove_selection(&mut self) -> bool {
        let Some((from, to)) = self.selection() else {
            return false;
        };
        let (start, end) = (self.byte(from), self.byte(to));
        self.text.replace_range(start..end, "");
        self.caret = from;
        self.anchor = from;
        true
    }

    /// Writes this at the caret, in place of anything selected.
    pub fn insert(&mut self, typed: &str) {
        self.remove_selection();
        let at = self.byte(self.caret);
        self.text.insert_str(at, typed);
        self.caret = self.place_of_byte(at + typed.len());
        self.anchor = self.caret;
    }

    /// Takes out what is selected, or from the caret to where a motion ends.
    fn remove_to(&mut self, motion: Motion) {
        if self.remove_selection() {
            return;
        }
        let other = self.target(motion);
        self.anchor = other;
        self.remove_selection();
    }

    /// Backspace: the character before the caret, or the word with `word`.
    pub fn backspace(&mut self, word: bool) {
        self.remove_to(if word { Motion::WordLeft } else { Motion::Left });
    }

    /// Delete: the character after the caret, or the word with `word`.
    pub fn delete(&mut self, word: bool) {
        self.remove_to(if word {
            Motion::WordRight
        } else {
            Motion::Right
        });
    }

    /// Takes out what is selected and gives it back.
    pub fn cut(&mut self) -> String {
        let selected = self.selected();
        self.remove_selection();
        selected
    }

    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.caret = self.len();
    }

    /// Selects the word at this place (or the spaces or marks, if that is what
    /// is there), as a double click does.
    pub fn select_word(&mut self, place: usize) {
        let chars = self.chars();
        if chars.is_empty() {
            return;
        }
        // At the end, the word before.
        let at = place.min(chars.len() - 1);
        let word = chars[at];
        let mut from = at;
        while from > 0 && chars[from - 1] == word {
            from -= 1;
        }
        let mut to = at + 1;
        while to < chars.len() && chars[to] == word {
            to += 1;
        }
        self.anchor = from;
        self.caret = to;
    }
}

#[cfg(test)]
mod tests;
