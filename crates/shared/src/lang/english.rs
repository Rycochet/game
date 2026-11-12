//! English's rules: "a" or "an", possessives, lists and capitals.

use super::rules::Rules;

pub(super) struct English;

/// Starts of words spelt with a vowel but said with a consonant: "a unicorn",
/// "a one-eyed wolf".
const SAID_WITH_A_CONSONANT: [&str; 10] = [
    "uni", "use", "usu", "uti", "ute", "ubiq", "eu", "ewe", "one", "once",
];

/// Starts of words spelt with an "h" that isn't said: "an hour". "Herb" isn't
/// here: British English says its "h".
const SILENT_H: [&str; 4] = ["hour", "honest", "honour", "heir"];

impl Rules for English {
    fn indefinite(&self, phrase: &str) -> String {
        let phrase = phrase.trim_start();
        if phrase.is_empty() {
            return String::new();
        }
        let article = if takes_an(phrase) { "an" } else { "a" };
        format!("{article} {phrase}")
    }

    fn possessive(&self, name: &str) -> String {
        if name.ends_with(['s', 'S']) {
            format!("{name}'")
        } else {
            format!("{name}'s")
        }
    }

    fn list(&self, items: &[&str]) -> String {
        match items {
            [] => String::new(),
            [one] => (*one).to_string(),
            [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
        }
    }

    fn capitalise(&self, text: &str) -> String {
        match text.char_indices().find(|(_, c)| c.is_alphabetic()) {
            Some((at, first)) => {
                let after = at + first.len_utf8();
                format!("{}{}{}", &text[..at], first.to_uppercase(), &text[after..])
            }
            None => text.to_string(),
        }
    }
}

/// Whether the phrase's first word is said starting with a vowel.
fn takes_an(phrase: &str) -> bool {
    let word = phrase.to_lowercase();
    let starts = |prefixes: &[&str]| prefixes.iter().any(|p| word.starts_with(p));
    if starts(&SAID_WITH_A_CONSONANT) {
        return false;
    }
    if starts(&SILENT_H) {
        return true;
    }
    word.starts_with(['a', 'e', 'i', 'o', 'u'])
}

#[cfg(test)]
mod tests;
