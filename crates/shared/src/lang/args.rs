//! The values a message is rendered with.

use fluent_bundle::{FluentArgs, FluentValue};

use crate::gender::Gender;

/// A fact's value as a language file sees it: a key for a select expression
/// (`{ $gender -> [male] he [female] she *[neutral] they }`), and the message
/// that names it (`<fact>-<key>`, such as `gender-male`). Story facts
/// (relation, trait, direction, distance, season, truth...) implement it as
/// they are added.
pub trait Selector {
    /// What kind of fact, in kebab case: `gender`, `relation`.
    fn fact(&self) -> &'static str;

    /// This value, in kebab case: `male`, `grandmother`. Never shown: the
    /// language file turns it into words.
    fn key(&self) -> &'static str;
}

impl Selector for Gender {
    fn fact(&self) -> &'static str {
        "gender"
    }

    fn key(&self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Neutral => "neutral",
        }
    }
}

/// Values by name for one message. Names in language files are kebab case
/// (`$giver-name`), so the keys here are too.
#[derive(Default)]
pub struct Args {
    pub(super) inner: FluentArgs<'static>,
}

impl Args {
    pub fn new() -> Self {
        Self::default()
    }

    /// A proper name, as it is: names are never translated.
    pub fn name(self, key: impl Into<String>, name: impl Into<String>) -> Self {
        self.text(key, name)
    }

    /// Words already in this language, such as a phrase rendered earlier or a
    /// list joined by the language's rules.
    pub fn text(mut self, key: impl Into<String>, text: impl Into<String>) -> Self {
        self.inner.set(key.into(), FluentValue::from(text.into()));
        self
    }

    /// A number, which also picks the plural form (`[one]`, `*[other]`).
    pub fn count(mut self, key: impl Into<String>, count: i64) -> Self {
        self.inner.set(key.into(), FluentValue::from(count));
        self
    }

    /// A fact's value, for select expressions.
    pub fn choice(self, key: impl Into<String>, value: &impl Selector) -> Self {
        self.text(key, value.key())
    }

    /// Whether a value of that name is given.
    pub fn has(&self, key: &str) -> bool {
        self.inner.get(key).is_some()
    }
}
