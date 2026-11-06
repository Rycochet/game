//! Items written into text, such as a chat line or a note: `${item:<uuid>}`, the
//! item's id in the usual text form of a UUID. Copied and pasted, it stays the same
//! item; what it is called and looks like is asked of the server
//! (`ClientMessage::LookUpItems`), so the text itself gives nothing away.
//!
//! Only that exact shape counts: anything else, a misspelt one included, is plain
//! text. `@name` is kept for mentioning players later.

use uuid::Uuid;

use crate::id::Id;

/// What starts an item in text.
pub const ITEM_START: &str = "${item:";
/// What ends one.
pub const ITEM_END: char = '}';
/// How long the id between them is: the hyphenated form of a UUID.
const ID_LENGTH: usize = 36;

/// A part of some text: plain words, or an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece<'a> {
    Text(&'a str),
    Item(Id),
}

/// How an item is written into text.
pub fn item_token(id: Id) -> String {
    format!("{ITEM_START}{}{ITEM_END}", id.0.hyphenated())
}

/// The item written at the start of `text`, and how many bytes it takes.
pub fn item_at(text: &str) -> Option<(Id, usize)> {
    let rest = text.strip_prefix(ITEM_START)?;
    let id = rest.get(..ID_LENGTH)?;
    rest[ID_LENGTH..].strip_prefix(ITEM_END)?;
    // `parse_str` also takes other forms (simple, braced, urn): only hyphenated here.
    let hyphenated = id.char_indices().all(|(at, c)| match at {
        8 | 13 | 18 | 23 => c == '-',
        _ => c.is_ascii_hexdigit(),
    });
    let uuid = Uuid::parse_str(id).ok().filter(|_| hyphenated)?;
    Some((Id(uuid), ITEM_START.len() + ID_LENGTH + 1))
}

/// The text in order as plain words and items. Neighbouring words are one piece,
/// and there are no empty ones.
pub fn pieces(text: &str) -> Vec<Piece<'_>> {
    let mut pieces = Vec::new();
    let (mut start, mut at) = (0, 0);
    while let Some(found) = text[at..].find(ITEM_START) {
        at += found;
        match item_at(&text[at..]) {
            Some((id, length)) => {
                if start < at {
                    pieces.push(Piece::Text(&text[start..at]));
                }
                pieces.push(Piece::Item(id));
                at += length;
                start = at;
            }
            None => at += ITEM_START.len(),
        }
    }
    if start < text.len() {
        pieces.push(Piece::Text(&text[start..]));
    }
    pieces
}

/// Every item written in the text, each once, in the order first written.
pub fn items_in(text: &str) -> Vec<Id> {
    let mut items = Vec::new();
    for piece in pieces(text) {
        if let Piece::Item(id) = piece
            && !items.contains(&id)
        {
            items.push(id);
        }
    }
    items
}

#[cfg(test)]
mod tests;
