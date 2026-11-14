//! The Character panel's lines on the nine attributes, a row (Body, Mind,
//! Spirit) to a line, each with what its bonuses make of it.

use shared::sheet::{Attributes, Row};

/// A line for each row: "Body: Strength 100, Agility 95 (-5), Toughness 110
/// (+10)". `with` is what the server says they come to with every bonus
/// (`ServerMessage::Attributes`); until it has, they are shown as they are.
pub(super) fn attribute_lines(own: &Attributes, with: Option<&Attributes>) -> Vec<String> {
    Row::ALL
        .into_iter()
        .map(|row| {
            let values: Vec<_> = row
                .attributes()
                .iter()
                .map(|&which| {
                    let base = own.get(which);
                    let now = with.map_or(base, |with| with.get(which));
                    let change = i32::from(now) - i32::from(base);
                    match change {
                        0 => format!("{} {now}", which.name()),
                        _ => format!("{} {now} ({change:+})", which.name()),
                    }
                })
                .collect();
            format!("{}: {}", row.name(), values.join(", "))
        })
        .collect()
}
