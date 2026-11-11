//! What Settings' rows share: a row's tip says what its setting does, and
//! why it can't be used, if it can't (the row's `title` in markup).

pub(super) fn tip_words(what: &str, why_not: Option<&str>) -> String {
    match why_not {
        Some(why) => format!("{what}\n\n{why}"),
        None => what.into(),
    }
}
