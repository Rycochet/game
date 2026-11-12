//! A language's rules as functions its files can call: `{ A($item) }`,
//! `{ POSSESSIVE($giver-name) }`, `{ CAP(-ring) }`. A message or term can be
//! given too, and is rendered first.

use fluent_bundle::{FluentArgs, FluentValue};

use super::render::Bundle;
use super::rules::Rules;

/// Adds `A`, `POSSESSIVE` and `CAP` for the language's rules, and Fluent's own
/// `NUMBER`.
pub(super) fn add(bundle: &mut Bundle, rules: &'static dyn Rules) -> Result<(), String> {
    bundle.add_builtins().map_err(|e| e.to_string())?;
    let mut add_one = |id: &str, rule: fn(&dyn Rules, &str) -> String| {
        bundle
            .add_function(id, move |positional, named| {
                apply(rules, rule, positional, named)
            })
            .map_err(|e| e.to_string())
    };
    add_one("A", |r, s| r.indefinite(s))?;
    add_one("POSSESSIVE", |r, s| r.possessive(s))?;
    add_one("CAP", |r, s| r.capitalise(s))?;
    Ok(())
}

/// One rule on the first value, which must be text.
fn apply<'a>(
    rules: &dyn Rules,
    rule: fn(&dyn Rules, &str) -> String,
    positional: &[FluentValue<'a>],
    _named: &FluentArgs,
) -> FluentValue<'a> {
    match positional {
        [FluentValue::String(text)] => FluentValue::from(rule(rules, text)),
        _ => FluentValue::Error,
    }
}
