//! Shoulder armour's cells and weight.

use super::{Adornment, CapeCut, Cut, Finish, Length, ShouldersForm, ShouldersPiece, Span, Wings};
use crate::armour::Construction;
use crate::creature::Size;
use crate::item::Shape;
use crate::material::Material;

/// The material its main weight is worked out against, in tenths of a gram per cm³.
pub(super) const STEEL_DENSITY: u32 = 78;
pub(super) const LEATHER_DENSITY: u32 = 9;
pub(super) const LINEN_DENSITY: u32 = 15;
pub(super) const WOOD_DENSITY: u32 = 7;

impl ShouldersForm {
    /// The cells it takes up at an ordinary size, wide and long.
    pub fn shape(&self) -> Shape {
        let (wide, long) = match self.piece {
            ShouldersPiece::Pauldrons { .. } => (3, 2),
            ShouldersPiece::Spaulders { .. } => (2, 2),
            ShouldersPiece::Guards {
                size: Span::Small, ..
            } => (2, 1),
            ShouldersPiece::Guards { .. } => (2, 2),
            ShouldersPiece::Cape {
                cut: CapeCut::Yoke, ..
            } => (2, 1),
            ShouldersPiece::Cape { .. } => (2, 2),
            ShouldersPiece::Discs { .. } | ShouldersPiece::Ailettes { .. } => (2, 1),
            ShouldersPiece::Wing => (2, 3),
        };
        Shape::rectangle(wide, long)
    }

    /// What the pair weighs at an ordinary size, in grams: its main material's share
    /// by that material's density, then backing, straps and extras. Worked out from
    /// area × thickness × density (see the research).
    pub fn grams(&self) -> u32 {
        use Construction as C;
        // Main weight at its reference density, the reference, and everything else.
        let (main, reference, rest) = match (self.piece, self.construction) {
            (
                ShouldersPiece::Pauldrons {
                    wings,
                    lames,
                    upper_lames,
                    haute_piece,
                    gardbrace,
                    besagews,
                    cut,
                },
                construction,
            ) => {
                let wings = if wings == Wings::Large { 800 } else { 0 };
                let great = if cut == Cut::GreatLeft { 500 } else { 0 };
                let extras = upper_lames as u32 * 100
                    + haute_piece.height() as u32 * 3
                    + if gardbrace { 300 } else { 0 }
                    + if besagews { 250 } else { 0 };
                match construction {
                    C::Plate => (
                        2300 + 75 * lames as u32 + wings + great + extras,
                        STEEL_DENSITY,
                        150,
                    ),
                    C::Leather => (900 + 30 * lames as u32 + wings / 3, LEATHER_DENSITY, 80),
                    _ => (1800, STEEL_DENSITY, 200),
                }
            }
            (
                ShouldersPiece::Spaulders {
                    lames,
                    besagews,
                    pteruges,
                },
                construction,
            ) => {
                let besagews = if besagews { 250 } else { 0 };
                let pteruges = pteruges as u32 * 120;
                match construction {
                    _ if self.dendra() => (2400, STEEL_DENSITY, 120),
                    C::Plate => (1400 + 75 * lames as u32 + besagews, STEEL_DENSITY, 120),
                    C::Laminar => (500 + 25 * lames as u32, STEEL_DENSITY, 100 + pteruges),
                    _ => (
                        600 + 25 * lames as u32 + besagews / 3,
                        LEATHER_DENSITY,
                        pteruges,
                    ),
                }
            }
            (ShouldersPiece::Guards { size, .. }, construction) => {
                let at_steel = match size {
                    Span::Small => 900,
                    Span::Medium => 1800,
                    Span::Large => 2800,
                };
                match construction {
                    C::Scale => (at_steel * 9 / 10, STEEL_DENSITY, 150),
                    C::PlatesInCloth => (at_steel * 85 / 100, STEEL_DENSITY, 200),
                    C::Mail => (at_steel * 7 / 10, STEEL_DENSITY, 80),
                    C::Leather if self.finish == Finish::RingMail => {
                        (at_steel * 4 / 10, STEEL_DENSITY, at_steel / 4)
                    }
                    C::Leather => (at_steel * 4 / 10, LEATHER_DENSITY, 0),
                    // Lamellar and laminar: the plates, then the lacing.
                    _ => (at_steel * 9 / 10, STEEL_DENSITY, at_steel / 10),
                }
            }
            (ShouldersPiece::Cape { cut, length }, construction) => {
                let long = length == Length::Long;
                match (cut, construction) {
                    (CapeCut::Yoke, C::Padded) => (if long { 650 } else { 500 }, LINEN_DENSITY, 0),
                    (CapeCut::Yoke, _) => (if long { 750 } else { 600 }, LEATHER_DENSITY, 0),
                    (_, C::Leather) => (if long { 700 } else { 450 }, STEEL_DENSITY, 600),
                    (_, C::Scale) => (if long { 2300 } else { 1400 }, STEEL_DENSITY, 200),
                    _ => (if long { 2100 } else { 1300 }, STEEL_DENSITY, 0),
                }
            }
            (ShouldersPiece::Discs { diameter, boss }, construction) => {
                // Two discs 2 mm thick, by their area.
                let discs = 2 * diameter as u32 * diameter as u32 * 123 / 10_000;
                let boss = if boss { 80 } else { 0 };
                let mail = if construction == C::MailAndPlate {
                    600
                } else {
                    0
                };
                (discs + boss + mail, STEEL_DENSITY, 80)
            }
            (ShouldersPiece::Ailettes { size, .. }, construction) => {
                let (wide, high) = size.ailette();
                // Each cm² of 6 mm leather weighs about half a gram.
                let area = 2 * wide as u32 * high as u32 / 100;
                match construction {
                    C::Leather => (area * 54 / 100, LEATHER_DENSITY, 20),
                    _ if self.material == Material::Wood => (area * 56 / 100, WOOD_DENSITY, 20),
                    _ => (area * 118 / 100, STEEL_DENSITY, 20),
                }
            }
            (ShouldersPiece::Wing, _) => (2800, WOOD_DENSITY, 500),
        };
        let extras = match self.adornment {
            Adornment::None => 0,
            Adornment::Spikes(count) => 40 * count as u32,
            Adornment::Horns => 300,
            Adornment::BeastHead => 400,
            Adornment::Skull => 250,
        } + if self.finish == Finish::Studded {
            120
        } else {
            0
        };
        main * self.material.density() / reference + rest + extras
    }

    /// What the pair weighs made at this size: bigger across, as thick (the owner's
    /// decision), so by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
