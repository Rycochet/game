//! The language files built into the program, per language. Shared code can't
//! read files (the web build has none), so they are compiled in.

use super::language::Language;

/// One built-in file: its name (for errors) and what is in it.
pub(super) struct File {
    pub name: &'static str,
    pub text: &'static str,
}

const ENGLISH: &[File] = &[
    File {
        name: "en/common.ftl",
        text: include_str!("en/common.ftl"),
    },
    File {
        name: "en/story.ftl",
        text: include_str!("en/story.ftl"),
    },
    File {
        name: "en/story-plots.ftl",
        text: include_str!("en/story-plots.ftl"),
    },
    File {
        name: "en/talk.ftl",
        text: include_str!("en/talk.ftl"),
    },
    File {
        name: "en/quest-log.ftl",
        text: include_str!("en/quest-log.ftl"),
    },
    File {
        name: "en/item.ftl",
        text: include_str!("en/item.ftl"),
    },
];

pub(super) fn files(language: Language) -> &'static [File] {
    match language {
        Language::English => ENGLISH,
    }
}
