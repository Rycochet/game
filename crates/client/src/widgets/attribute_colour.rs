//! The colours of the three rows of attributes, as the creation screen's triangles
//! show them (Body red, Mind blue, Spirit green), and the colour of something that
//! leans on some of them: abilities, and skills.

use bevy::prelude::*;
use shared::sheet::{Attribute, Row};

/// How much of the blend an active thing keeps: dark, so a white icon stands out.
const DARKENED: f32 = 0.4;
/// How far a passive thing's blend goes towards white: pastel, so it reads apart
/// from active ones without its name.
const PASTEL: f32 = 0.55;
/// For something that leans on nothing.
const NEUTRAL: [u8; 3] = [120, 110, 100];

/// A row's colour, as bytes.
pub const fn row_rgb(row: Row) -> [u8; 3] {
    match row {
        Row::Body => [176, 76, 60],
        Row::Mind => [60, 104, 176],
        Row::Spirit => [96, 150, 84],
    }
}

/// How a blend of the rows' colours is shaded, by what it is for (owner,
/// 2026-11-07): so active abilities, passive ones and skills each read apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shade {
    /// An ability done when asked: dark.
    Active,
    /// An ability always at work: pastel.
    Passive,
    /// A skill, which just is, and serves either kind of ability: in between,
    /// the rows' own colours.
    Skill,
}

impl Shade {
    /// An ability's: `Passive` if it is, otherwise `Active`.
    pub fn of_ability(passive: bool) -> Self {
        if passive { Self::Passive } else { Self::Active }
    }
}

/// The rows' colours mixed by how many of these attributes are in each, then
/// shaded for what it is for.
pub fn attribute_colour(leans: &[Attribute], shade: Shade) -> Color {
    let [red, green, blue] = blend(leans);
    let shaded = |channel: f32| match shade {
        Shade::Active => channel * DARKENED,
        Shade::Passive => channel + (1.0 - channel) * PASTEL,
        Shade::Skill => channel,
    };
    Color::srgb(shaded(red), shaded(green), shaded(blue))
}

/// The rows' colours mixed, each channel from 0 to 1.
fn blend(leans: &[Attribute]) -> [f32; 3] {
    let colours: Vec<[u8; 3]> = if leans.is_empty() {
        vec![NEUTRAL]
    } else {
        leans.iter().map(|which| row_rgb(which.row())).collect()
    };
    let channel = |index: usize| {
        let sum: f32 = colours.iter().map(|colour| colour[index] as f32).sum();
        sum / colours.len() as f32 / 255.0
    };
    [channel(0), channel(1), channel(2)]
}

#[cfg(test)]
mod tests;
