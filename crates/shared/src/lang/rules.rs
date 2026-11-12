//! What a language needs said in Rust because its files can't: changes to a
//! word that depend on how the word itself is spelt.

/// One language's rules. Each is also a function its files can call (see
/// `functions.rs`): `A`, `POSSESSIVE` and `CAP`; lists are joined in Rust, as a
/// Fluent value can't hold several.
pub trait Rules: Sync {
    /// The phrase with its indefinite article: "a sword", "an axe".
    fn indefinite(&self, phrase: &str) -> String;

    /// Whose: "Mara's", "Charles'".
    fn possessive(&self, name: &str) -> String;

    /// Several things as one phrase: "a, b and c". Empty for none.
    fn list(&self, items: &[&str]) -> String;

    /// The text with its first letter a capital, for a phrase starting a
    /// sentence.
    fn capitalise(&self, text: &str) -> String;
}
