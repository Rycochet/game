//! Words in the player's language. Language files (Fluent's `.ftl`, built into
//! the program) hold every word; Rust only picks which message to show and
//! gives it values: proper names, numbers, and facts as selectors (`Gender`,
//! and later a story's relation, direction and so on). Each language also has a
//! few rules in Rust for what its files can't say (English: "a" or "an",
//! "Mara's" or "Charles'", lists, capitals). See `docs/context/shared/lang.md`
//! and `docs/context/story-text.md`.
//!
//! The client renders; the server only sends values, so text never has to
//! match between them, but the same values always give the same words.

mod args;
mod english;
mod files;
mod functions;
mod language;
mod render;
mod rules;
mod variables;

pub use args::{Args, Selector};
pub use language::Language;
pub use render::{Lang, RenderError};
pub use rules::Rules;

#[cfg(test)]
mod tests;
