//! Which values a message uses, read from its syntax tree, so tests can check
//! that whatever renders it gives them all.

use std::collections::BTreeSet;

use fluent_syntax::ast::{Expression, InlineExpression, Pattern, PatternElement};

use super::args::Args;
use super::render::Lang;

impl Lang {
    /// The names of every value a message (`id`) or attribute (`id.attribute`)
    /// uses, including those of messages it refers to (which see the same
    /// values). Terms are left out: theirs are given where they are used.
    pub fn variables(&self, id: &str) -> BTreeSet<String> {
        let mut found = BTreeSet::new();
        let mut seen = BTreeSet::new();
        self.collect(id, &mut found, &mut seen);
        found
    }

    /// The values a message uses that `args` doesn't give.
    pub fn missing(&self, id: &str, args: &Args) -> Vec<String> {
        self.variables(id)
            .into_iter()
            .filter(|name| !args.has(name))
            .collect()
    }

    fn collect(&self, id: &str, found: &mut BTreeSet<String>, seen: &mut BTreeSet<String>) {
        if !seen.insert(id.to_string()) {
            return;
        }
        let Some(pattern) = self.pattern(id) else {
            return;
        };
        let mut references = Vec::new();
        in_pattern(pattern, found, &mut references);
        for reference in references {
            self.collect(&reference, found, seen);
        }
    }
}

/// Values used in a pattern, and the messages it refers to.
fn in_pattern(pattern: &Pattern<&str>, found: &mut BTreeSet<String>, refs: &mut Vec<String>) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            in_expression(expression, found, refs);
        }
    }
}

fn in_expression(
    expression: &Expression<&str>,
    found: &mut BTreeSet<String>,
    refs: &mut Vec<String>,
) {
    match expression {
        Expression::Inline(inline) => in_inline(inline, found, refs),
        Expression::Select { selector, variants } => {
            in_inline(selector, found, refs);
            for variant in variants {
                in_pattern(&variant.value, found, refs);
            }
        }
    }
}

fn in_inline(
    inline: &InlineExpression<&str>,
    found: &mut BTreeSet<String>,
    refs: &mut Vec<String>,
) {
    match inline {
        InlineExpression::VariableReference { id } => {
            found.insert(id.name.to_string());
        }
        InlineExpression::FunctionReference { arguments, .. } => {
            for argument in &arguments.positional {
                in_inline(argument, found, refs);
            }
        }
        InlineExpression::MessageReference { id, attribute } => refs.push(match attribute {
            Some(attribute) => format!("{}.{}", id.name, attribute.name),
            None => id.name.to_string(),
        }),
        InlineExpression::Placeable { expression } => in_expression(expression, found, refs),
        InlineExpression::StringLiteral { .. }
        | InlineExpression::NumberLiteral { .. }
        | InlineExpression::TermReference { .. } => {}
    }
}
