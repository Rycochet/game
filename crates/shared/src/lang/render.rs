//! A language's messages, ready to render.

use std::fmt;

use fluent_bundle::FluentResource;
use fluent_syntax::ast;
use unic_langid::LanguageIdentifier;

use super::args::{Args, Selector};
use super::files::files;
use super::functions;
use super::language::Language;
use super::rules::Rules;

/// Fluent's bundle that may be shared between threads, as Bevy resources are.
pub(super) type Bundle = fluent_bundle::concurrent::FluentBundle<FluentResource>;

/// Every message of one language, with its rules. Built once (parsing its
/// files) and kept.
pub struct Lang {
    language: Language,
    pub(super) bundle: Bundle,
    ids: Vec<String>,
}

/// A message that couldn't be rendered as written: unknown, or missing a value
/// or something it refers to. What could be rendered is in `text`, with the
/// missing parts in braces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError {
    pub id: String,
    pub text: String,
    pub problems: Vec<String>,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.id, self.problems.join("; "))
    }
}

impl Lang {
    /// The language's built-in files. They are checked by tests, so this only
    /// fails if one is broken.
    pub fn new(language: Language) -> Self {
        let sources = files(language).iter().map(|file| (file.name, file.text));
        Self::build(language, sources).unwrap_or_else(|problems| {
            panic!(
                "built-in {} files: {}",
                language.name(),
                problems.join("; ")
            )
        })
    }

    /// The language from named sources: every one must parse, no message may
    /// be written twice.
    pub(super) fn build<'a>(
        language: Language,
        sources: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, Vec<String>> {
        let id: LanguageIdentifier = language.code().parse().map_err(|e| vec![format!("{e}")])?;
        let mut bundle = Bundle::new_concurrent(vec![id]);
        // Fluent marks where values start and end for right-to-left text;
        // Lexend can't draw the marks.
        bundle.set_use_isolating(false);
        functions::add(&mut bundle, language.rules()).map_err(|e| vec![e])?;
        let mut ids = Vec::new();
        let mut problems = Vec::new();
        for (name, text) in sources {
            let resource = match FluentResource::try_new(text.to_string()) {
                Ok(resource) => resource,
                Err((_, errors)) => {
                    problems.extend(errors.iter().map(|e| format!("{name}: {e}")));
                    continue;
                }
            };
            for entry in resource.entries() {
                if let ast::Entry::Message(message) = entry {
                    ids.push(message.id.name.to_string());
                }
            }
            if let Err(errors) = bundle.add_resource(resource) {
                problems.extend(errors.iter().map(|e| format!("{name}: {e}")));
            }
        }
        if !problems.is_empty() {
            return Err(problems);
        }
        ids.sort();
        Ok(Self {
            language,
            bundle,
            ids,
        })
    }

    pub fn language(&self) -> Language {
        self.language
    }

    /// Its rules, for what Rust joins itself (lists, capitals).
    pub fn rules(&self) -> &'static dyn Rules {
        self.language.rules()
    }

    /// Every message's id, sorted.
    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    /// Whether there is a message (`id`) or attribute (`id.attribute`).
    pub fn has(&self, id: &str) -> bool {
        self.pattern(id).is_some()
    }

    /// A message's attributes' names, in the order written: a message may hold
    /// several wordings as `.one`, `.two` and so on.
    pub fn attributes(&self, id: &str) -> Vec<&str> {
        self.bundle
            .get_message(id)
            .map(|m| m.attributes().map(|a| a.id()).collect())
            .unwrap_or_default()
    }

    /// The message (`id`) or attribute (`id.attribute`) with these values.
    pub fn render(&self, id: &str, args: &Args) -> Result<String, RenderError> {
        let Some(pattern) = self.pattern(id) else {
            return Err(RenderError {
                id: id.to_string(),
                text: format!("{{{id}}}"),
                problems: vec!["no such message".to_string()],
            });
        };
        let mut errors = Vec::new();
        let text = self
            .bundle
            .format_pattern(pattern, Some(&args.inner), &mut errors);
        if errors.is_empty() {
            Ok(text.into_owned())
        } else {
            Err(RenderError {
                id: id.to_string(),
                text: text.into_owned(),
                problems: errors.iter().map(|e| e.to_string()).collect(),
            })
        }
    }

    /// A fact's value in words: the message `<fact>-<key>`, such as
    /// `gender-female`.
    pub fn phrase(&self, value: &impl Selector) -> Result<String, RenderError> {
        self.render(&format!("{}-{}", value.fact(), value.key()), &Args::new())
    }

    pub(super) fn pattern(&self, id: &str) -> Option<&ast::Pattern<&str>> {
        let (message, attribute) = match id.split_once('.') {
            Some((message, attribute)) => (message, Some(attribute)),
            None => (id, None),
        };
        let message = self.bundle.get_message(message)?;
        match attribute {
            Some(attribute) => Some(message.get_attribute(attribute)?.value()),
            None => message.value(),
        }
    }
}
