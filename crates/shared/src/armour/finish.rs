//! Colours and finishes every armour piece shares, agreed between the location chats: a
//! dye for cloth and lacing, and a finish for metal. A set passes both to its pieces.

use serde::{Deserialize, Serialize};

use super::Style;
use crate::item::Quality;
use crate::rng::Rng;

/// What colour cloth or lacing is dyed. Dyeing things comes later; for now it is rolled.
/// Saved by name, so new ones go at the end.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dye {
    /// Undyed: the cloth's or leather's own colour.
    #[default]
    Natural,
    Red,
    Blue,
    Green,
    Black,
    Brown,
    White,
    Yellow,
    /// Rare.
    Purple,
}

impl Dye {
    pub const ALL: [Self; 9] = [
        Self::Natural,
        Self::Red,
        Self::Blue,
        Self::Green,
        Self::Black,
        Self::Brown,
        Self::White,
        Self::Yellow,
        Self::Purple,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Natural => "Natural",
            Self::Red => "Red",
            Self::Blue => "Blue",
            Self::Green => "Green",
            Self::Black => "Black",
            Self::Brown => "Brown",
            Self::White => "White",
            Self::Yellow => "Yellow",
            Self::Purple => "Purple",
        }
    }

    /// Undyed the most often, purple seldom.
    pub fn roll(rng: &mut Rng) -> Self {
        rng.pick_weighted(&[
            (Self::Natural, 30),
            (Self::Red, 16),
            (Self::Blue, 14),
            (Self::Green, 8),
            (Self::Black, 10),
            (Self::Brown, 10),
            (Self::White, 5),
            (Self::Yellow, 5),
            (Self::Purple, 2),
        ])
    }

    /// A colour for something painted or lacquered: never undyed.
    pub fn roll_paint(rng: &mut Rng) -> Self {
        match Self::roll(rng) {
            Self::Natural => Self::Black,
            dye => dye,
        }
    }
}

/// How metal (and lacquered leather) is finished. Saved by name, so new ones go at the
/// end.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Finish {
    /// Polished.
    #[default]
    Bright,
    Blackened,
    /// Browned with rust kept in check.
    Russet,
    /// Heated to a deep blue.
    Blued,
    /// Coated in tin, silvery.
    Tinned,
    Painted(Dye),
    /// Japanese, Chinese and on hardened leather.
    Lacquered(Dye),
    /// Gold over it: Rare and better only.
    Gilt,
    /// Bands of etching: Rare and better only.
    Etched,
}

impl Finish {
    /// Every finish, in order, painted and lacquered in every colour but undyed.
    pub fn all() -> Vec<Self> {
        let colours = || Dye::ALL.into_iter().filter(|&dye| dye != Dye::Natural);
        let mut all = vec![
            Self::Bright,
            Self::Blackened,
            Self::Russet,
            Self::Blued,
            Self::Tinned,
        ];
        all.extend(colours().map(Self::Painted));
        all.extend(colours().map(Self::Lacquered));
        all.extend([Self::Gilt, Self::Etched]);
        all
    }

    /// Only made on things of Rare quality and better (the owner's decision).
    pub fn rare_only(self) -> bool {
        matches!(self, Self::Gilt | Self::Etched)
    }

    /// Whether a thing of this quality may have it.
    pub fn allowed(self, quality: Quality) -> bool {
        !self.rare_only() || quality >= Quality::Rare
    }

    /// A finish for metal made in this style at this quality: lacquer in the east,
    /// etching often on Greenwich work, gilt and etching only at Rare and better.
    pub fn roll(rng: &mut Rng, style: Style, quality: Quality) -> Self {
        let rare = quality >= Quality::Rare;
        let fine = |weight: u64| if rare { weight } else { 0 };
        if matches!(style, Style::Japanese | Style::Chinese) {
            return match rng.pick_weighted(&[(0, 80), (1, 15), (2, fine(5))]) {
                0 => Self::Lacquered(Dye::roll_paint(rng)),
                1 => Self::Russet,
                _ => Self::Gilt,
            };
        }
        let etched = if style == Style::Greenwich { 40 } else { 6 };
        match rng.pick_weighted(&[
            (0, 55),
            (1, 12),
            (2, 10),
            (3, 8),
            (4, 5),
            (5, 4),
            (6, fine(6)),
            (7, fine(etched)),
        ]) {
            0 => Self::Bright,
            1 => Self::Blackened,
            2 => Self::Russet,
            3 => Self::Blued,
            4 => Self::Tinned,
            5 => Self::Painted(Dye::roll_paint(rng)),
            6 => Self::Gilt,
            _ => Self::Etched,
        }
    }
}
