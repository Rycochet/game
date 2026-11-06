//! What a box's places are between: characters, and in a box that takes items, an
//! item written in (`${item:<uuid>}`, `shared::item_text`) as one whole.

use shared::id::Id;
use shared::item_text::item_at;

/// One place's worth of writing: a character, or an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::widgets) struct Unit {
    /// Where it starts and ends, in bytes.
    pub start: usize,
    pub end: usize,
    /// The item it is, if it is one.
    pub item: Option<Id>,
    pub c: char,
}

impl Unit {
    /// What sort of thing it is, for moving by words: a word runs over units of
    /// one sort. An item is a word of its own sort.
    pub fn sort(self) -> u8 {
        if self.item.is_some() {
            3
        } else if self.c.is_whitespace() {
            0
        } else if self.c.is_alphanumeric() || self.c == '_' {
            1
        } else {
            2
        }
    }
}

/// The text's units, items as one each only if `items`.
pub(in crate::widgets) fn units(text: &str, items: bool) -> Vec<Unit> {
    let mut units = Vec::new();
    let mut indices = text.char_indices().peekable();
    while let Some((start, c)) = indices.next() {
        if items && let Some((id, length)) = item_at(&text[start..]) {
            let end = start + length;
            while indices.next_if(|&(at, _)| at < end).is_some() {}
            units.push(Unit {
                start,
                end,
                item: Some(id),
                c,
            });
            continue;
        }
        let end = start + c.len_utf8();
        units.push(Unit {
            start,
            end,
            item: None,
            c,
        });
    }
    units
}

/// The place at this byte: the number of units starting before it, so a byte
/// inside an item is after it.
pub(in crate::widgets) fn place_of_byte(units: &[Unit], byte: usize) -> usize {
    units.iter().take_while(|unit| unit.start < byte).count()
}

/// Where the unit numbered `place` starts, in bytes.
pub(in crate::widgets) fn byte_of_place(units: &[Unit], place: usize, len: usize) -> usize {
    units.get(place).map_or(len, |unit| unit.start)
}
