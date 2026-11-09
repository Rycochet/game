//! Characters' names: plain letters and digits, with single spaces, `'` and `-`
//! between them, so no two names look alike and nothing in one can pass for
//! something else (such as a staff mark). What is typed is folded into that as
//! it is typed (`fold`): lookalike apostrophes, dashes and spaces become the
//! plain ones and accents come off. The server folds again and checks
//! (`checked`): it decides the name.

use unicode_normalization::char::decompose_compatible;

/// The longest a name may be, in characters.
pub const MAX_NAME: usize = 24;

/// What a character with no name of its own is called.
pub const PLAIN_NAME: &str = "Character";

/// Folds typed text towards a name: anything that can't be in one is turned
/// into what can, or dropped, and no separator starts it or follows another.
/// One may still end it, as the player may be typing the next word.
pub fn fold(typed: &str) -> String {
    let mut name = String::new();
    for c in typed.chars() {
        for plain in plain_letters(c).chars() {
            let separator = is_separator(plain);
            if separator && name.chars().last().is_none_or(is_separator) {
                continue;
            }
            name.push(plain);
        }
    }
    name
}

/// The name as it will be kept: folded, without a separator at the end. An
/// error if nothing is left, or it is too long.
pub fn checked(typed: &str) -> Result<String, String> {
    let name = fold(typed).trim_end_matches(is_separator).to_owned();
    let length = name.chars().count();
    if !(1..=MAX_NAME).contains(&length) {
        return Err(format!(
            "a name is 1 to {MAX_NAME} letters, digits, spaces, ' and -"
        ));
    }
    Ok(name)
}

/// What two names that count as the same share: names differ by more than
/// their capitals.
pub fn key(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// `base` with a number after it, fitting in `MAX_NAME` (`Brin 2`), for a name
/// that is already taken.
pub fn numbered(base: &str, number: u32) -> String {
    let suffix = format!(" {number}");
    let room = MAX_NAME - suffix.len();
    let base: String = base.chars().take(room).collect();
    format!("{}{suffix}", base.trim_end_matches(is_separator))
}

fn is_separator(c: char) -> bool {
    matches!(c, ' ' | '\'' | '-')
}

/// What a typed character becomes in a name: itself, its plain form, or nothing.
fn plain_letters(c: char) -> String {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' | '\'' | '-' => return c.into(),
        '\u{2018}' | '\u{2019}' | '\u{201B}' | '\u{02BC}' | '\u{2032}' | '\u{00B4}' | '`'
        | '\u{02B9}' | '\u{FF07}' => return "'".into(),
        '_' | '\u{2010}'..='\u{2015}' | '\u{2212}' | '\u{FE63}' | '\u{FF0D}' => {
            return "-".into();
        }
        _ if c.is_whitespace() => return " ".into(),
        _ => {}
    }
    // Letters that don't split into a letter and its accent.
    let spelled = match c {
        'ß' => "ss",
        'æ' => "ae",
        'Æ' => "AE",
        'œ' => "oe",
        'Œ' => "OE",
        'ø' => "o",
        'Ø' => "O",
        'ł' => "l",
        'Ł' => "L",
        'þ' => "th",
        'Þ' => "Th",
        'ð' | 'đ' => "d",
        'Ð' | 'Đ' => "D",
        'ı' => "i",
        'ħ' => "h",
        'Ħ' => "H",
        _ => "",
    };
    if !spelled.is_empty() {
        return spelled.into();
    }
    // Accented letters (and full-width ones) split into plain letters and
    // marks: the marks are dropped.
    let mut plain = String::new();
    decompose_compatible(c, |part| {
        if part.is_ascii_alphanumeric() {
            plain.push(part);
        }
    });
    plain
}

#[cfg(test)]
mod tests;
