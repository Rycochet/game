//! Armour for the shoulders, as a pair: pauldrons, spaulders, hanging shoulder guards
//! (sode and their eastern cousins), mail and scale doublings and the linothorax yoke,
//! shoulder discs, ailettes and the Chukchi wing. What a pair is like is rolled once
//! and saved; how big a shoulder is comes from the body frame, so the same pair is drawn
//! alone and worn. See `docs/context/shared/armour.md` and the research in
//! `docs/context/research-armour-shoulders.md`.

use serde::{Deserialize, Serialize};

use super::{Construction, Style, in_style};
use crate::creature::Size;
use crate::item::Shape;
use crate::material::Material;
use crate::rng::Rng;

/// Which kind of shoulder piece, with what only that kind has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShouldersPiece {
    /// A dome over the joint (the cop), wings over the breast and back, and lames down
    /// the arm: covers the armpit.
    Pauldrons {
        cut: Cut,
        wings: Wings,
        /// Lames below the cop, and up towards the neck.
        lames: u8,
        upper_lames: u8,
        haute_piece: HautePiece,
        /// A second plate over the front of the cop.
        gardbrace: bool,
        /// Discs hung in front of the armpits.
        besagews: bool,
    },
    /// A smaller cop and lames over the top and outside of the shoulder; the armpit is
    /// open. Laminar ones are Roman, bronze plate ones Mycenaean (Dendra).
    Spaulders {
        lames: u8,
        besagews: bool,
        /// Rows of hanging strips below them.
        pteruges: u8,
    },
    /// A panel hanging from the shoulder over the upper arm, in rows.
    Guards {
        size: Span,
        shape: GuardShape,
        rows: u8,
        lacing: Lacing,
    },
    /// One piece over both shoulders round the neck.
    Cape { cut: CapeCut, length: Length },
    /// Round plates on the outside of the shoulders.
    Discs {
        /// Across, in millimetres.
        diameter: u16,
        boss: bool,
    },
    /// Flat boards standing on the shoulders, painted.
    Ailettes {
        shape: AiletteShape,
        size: Span,
        heraldry: Heraldry,
    },
    /// One huge board over the left shoulder down to the waist, used as a shield.
    Wing,
}

/// The kinds of shoulder piece, without what each has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShouldersKind {
    Pauldrons,
    Spaulders,
    Guards,
    Cape,
    Discs,
    Ailettes,
    Wing,
}

impl ShouldersKind {
    pub const ALL: [Self; 7] = [
        Self::Pauldrons,
        Self::Spaulders,
        Self::Guards,
        Self::Cape,
        Self::Discs,
        Self::Ailettes,
        Self::Wing,
    ];
}

/// How a pair of pauldrons matches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cut {
    Matched,
    /// The left larger, its wings nearly meeting in the middle (Milanese).
    GreatLeft,
    /// The right cut away under the arm for a lance rest.
    LanceCut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Wings {
    Small,
    Large,
}

/// An upright flange on a pauldron guarding the neck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HautePiece {
    None,
    Low,
    High,
}

impl HautePiece {
    /// How far it stands up from the cop, in millimetres on an ordinary body.
    pub fn height(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Low => 25,
            Self::High => 50,
        }
    }
}

/// How big a shoulder guard or an ailette is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Span {
    Small,
    Medium,
    Large,
}

impl Span {
    /// A shoulder guard's width round the arm and height, in millimetres on an
    /// ordinary body.
    pub fn guard(self) -> (i32, i32) {
        match self {
            Self::Small => (200, 200),
            Self::Medium => (260, 280),
            Self::Large => (330, 360),
        }
    }

    /// An ailette's width and height.
    pub fn ailette(self) -> (i32, i32) {
        match self {
            Self::Small => (120, 150),
            Self::Medium => (140, 175),
            Self::Large => (160, 200),
        }
    }

    /// How many rows a guard of this size has, fewest and most.
    fn rows(self) -> (u8, u8) {
        match self {
            Self::Small => (3, 5),
            Self::Medium => (5, 6),
            Self::Large => (6, 7),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardShape {
    Straight,
    /// Narrower at the bottom, wrapping the arm (tsubo-sode).
    Tapered,
    /// Wider at the bottom (hiro-sode).
    Flared,
}

/// How close the lacing of lamellae is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lacing {
    /// Covering the plates (kebiki).
    Dense,
    /// In spaced pairs (sugake).
    Spaced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapeCut {
    /// Round the neck and over both shoulders: a Roman doubling.
    Round,
    /// From the back over each shoulder, tied down on the chest: a linothorax's top.
    Yoke,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Length {
    /// To the shoulders' edge.
    Short,
    /// To the middle of the upper arm.
    Long,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiletteShape {
    Rectangle,
    Rounded,
    Shield,
    Lozenge,
}

/// The colours of heraldry: two metals and five colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tincture {
    Or,
    Argent,
    Gules,
    Azure,
    Vert,
    Sable,
    Purpure,
}

impl Tincture {
    pub const ALL: [Self; 7] = [
        Self::Or,
        Self::Argent,
        Self::Gules,
        Self::Azure,
        Self::Vert,
        Self::Sable,
        Self::Purpure,
    ];

    /// Gold and silver: heraldry puts metal on colour and colour on metal.
    pub fn metal(self) -> bool {
        matches!(self, Self::Or | Self::Argent)
    }
}

/// What is painted on the field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Charge {
    Plain,
    Cross,
    Saltire,
    Pale,
    Fess,
    Bend,
    Chevron,
}

impl Charge {
    pub const ALL: [Self; 7] = [
        Self::Plain,
        Self::Cross,
        Self::Saltire,
        Self::Pale,
        Self::Fess,
        Self::Bend,
        Self::Chevron,
    ];
}

/// A simple coat of arms. Kept apart so shields and surcoats can share it later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Heraldry {
    pub field: Tincture,
    pub charge: Charge,
    /// The charge's colour: the field's when plain.
    pub colour: Tincture,
}

impl Heraldry {
    /// Metal on colour or colour on metal, and nothing to colour when plain.
    pub fn allowed(&self) -> bool {
        if self.charge == Charge::Plain {
            self.colour == self.field
        } else {
            self.colour.metal() != self.field.metal()
        }
    }

    pub fn roll(rng: &mut Rng) -> Self {
        let field = Tincture::ALL[rng.range_u64(0, 7) as usize];
        let charge = rng.pick_weighted(&[
            (Charge::Plain, 10),
            (Charge::Cross, 20),
            (Charge::Saltire, 12),
            (Charge::Pale, 12),
            (Charge::Fess, 14),
            (Charge::Bend, 16),
            (Charge::Chevron, 16),
        ]);
        let colour = if charge == Charge::Plain {
            field
        } else {
            let others: Vec<_> = Tincture::ALL
                .into_iter()
                .filter(|other| other.metal() != field.metal())
                .collect();
            others[rng.range_u64(0, others.len() as u64) as usize]
        };
        Self {
            field,
            charge,
            colour,
        }
    }
}

/// Something fantastic added on top, whatever the style (the owner's list).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Adornment {
    None,
    /// This many spikes along the top: 1, 3 or 5.
    Spikes(u8),
    /// A pair of curved horns.
    Horns,
    /// A beast's head, the arm coming out of its mouth, as on Chinese generals' armour.
    BeastHead,
    Skull,
}

/// How leather is finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Finish {
    Plain,
    /// Pressed and cut patterns.
    Tooled,
    /// Rows of rivet heads: studded leather.
    Studded,
    /// Rings sewn flat on it: ring mail.
    RingMail,
}

/// A pair of shoulder pieces: everything about them that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShouldersForm {
    pub piece: ShouldersPiece,
    pub construction: Construction,
    pub adornment: Adornment,
    pub finish: Finish,
    /// What it is mostly made of: the plates, rings, scales, leather, padding or board.
    pub material: Material,
    /// The cloth over plates in cloth, or what lamellae are laced with.
    pub cloth: Option<Material>,
    /// Edges of another metal.
    pub trim: Option<Material>,
    pub style: Style,
}

/// The material its main weight is worked out against, in tenths of a gram per cm³.
const STEEL_DENSITY: u32 = 78;
const LEATHER_DENSITY: u32 = 9;
const LINEN_DENSITY: u32 = 15;
const WOOD_DENSITY: u32 = 7;

impl ShouldersForm {
    pub fn kind(&self) -> ShouldersKind {
        match self.piece {
            ShouldersPiece::Pauldrons { .. } => ShouldersKind::Pauldrons,
            ShouldersPiece::Spaulders { .. } => ShouldersKind::Spaulders,
            ShouldersPiece::Guards { .. } => ShouldersKind::Guards,
            ShouldersPiece::Cape { .. } => ShouldersKind::Cape,
            ShouldersPiece::Discs { .. } => ShouldersKind::Discs,
            ShouldersPiece::Ailettes { .. } => ShouldersKind::Ailettes,
            ShouldersPiece::Wing => ShouldersKind::Wing,
        }
    }

    /// Roman laminar spaulders.
    pub fn roman(&self) -> bool {
        self.kind() == ShouldersKind::Spaulders
            && self.construction == Construction::Laminar
            && self.style == Style::Classical
    }

    /// Mycenaean bronze spaulders, as at Dendra: one big plate each.
    pub fn dendra(&self) -> bool {
        self.kind() == ShouldersKind::Spaulders
            && self.construction == Construction::Plate
            && self.style == Style::Classical
    }

    /// Japanese shoulder guards, sode: with a solid top plate.
    pub fn japanese(&self) -> bool {
        self.kind() == ShouldersKind::Guards && self.style == Style::Japanese
    }

    /// Every pair with these choices: the grid a roll keeps to.
    pub fn allowed(&self) -> bool {
        let kind = self.kind();
        let construction = self.construction;
        let has = |list: &[(Construction, u64)]| list.iter().any(|&(c, _)| c == construction);
        let piece_fits = match self.piece {
            ShouldersPiece::Pauldrons {
                cut,
                wings,
                lames,
                upper_lames,
                haute_piece,
                gardbrace,
                besagews,
            } => {
                let plate = construction == Construction::Plate;
                let hanging = !matches!(construction, Construction::Plate | Construction::Leather);
                (plate || cut == Cut::Matched)
                    && (!hanging || wings == Wings::Small)
                    && if hanging {
                        lames == 0
                    } else {
                        (3..=6).contains(&lames)
                    }
                    && (plate || upper_lames == 0)
                    && upper_lames <= 2
                    && (plate || haute_piece == HautePiece::None)
                    && (plate || !gardbrace)
                    && (!besagews || (wings == Wings::Small && !hanging))
            }
            ShouldersPiece::Spaulders {
                lames,
                besagews,
                pteruges,
            } => {
                let dendra = self.dendra();
                (if dendra {
                    lames == 0
                } else {
                    (2..=5).contains(&lames)
                }) && (!besagews
                    || matches!(construction, Construction::Plate | Construction::Leather)
                        && !dendra)
                    && (pteruges == 0
                        || matches!(construction, Construction::Laminar | Construction::Leather))
                    && pteruges <= 2
            }
            ShouldersPiece::Guards {
                size, rows, lacing, ..
            } => {
                let (fewest, most) = size.rows();
                (fewest..=most).contains(&rows) && (lacing == Lacing::Dense || laced(construction))
            }
            ShouldersPiece::Cape { cut, .. } => match cut {
                CapeCut::Round => {
                    matches!(construction, Construction::Mail | Construction::Scale)
                        || construction == Construction::Leather && self.finish == Finish::RingMail
                }
                CapeCut::Yoke => {
                    matches!(construction, Construction::Padded | Construction::Leather)
                        && self.finish != Finish::RingMail
                        && self.style == Style::Classical
                }
            },
            ShouldersPiece::Discs { diameter, .. } => {
                (110..=160).contains(&diameter) && diameter % 10 == 0
            }
            ShouldersPiece::Ailettes { heraldry, .. } => heraldry.allowed(),
            ShouldersPiece::Wing => true,
        };
        has(&constructions_in(kind, self.style))
            && piece_fits
            && adornments(kind, construction)
                .iter()
                .any(|&(a, _)| a == self.adornment)
            && finishes(kind, construction, &self.piece)
                .iter()
                .any(|&(f, _)| f == self.finish)
            && materials(kind, construction, self.style)
                .iter()
                .any(|&(m, _)| m == self.material)
            && self.cloth.is_some() == needs_cloth(construction)
            && (self.trim.is_none() || may_have_trim(kind, construction, self.material))
    }

    /// What anything like it is called.
    pub fn generic_name(&self) -> &'static str {
        match self.kind() {
            ShouldersKind::Pauldrons => "Pauldrons",
            ShouldersKind::Spaulders => "Spaulders",
            ShouldersKind::Guards => "Shoulder Guards",
            ShouldersKind::Cape => "Shoulder Cape",
            ShouldersKind::Discs => "Shoulder Discs",
            ShouldersKind::Ailettes => "Ailettes",
            ShouldersKind::Wing => "Wing",
        }
    }

    /// Its own name, when it fits one: Japanese, Roman, Greek, Russian and Bronze Age
    /// pieces by theirs (the owner's choice), others by what makes them stand out.
    pub fn specific_name(&self) -> Option<&'static str> {
        use Construction as C;
        Some(match (self.piece, self.construction) {
            (ShouldersPiece::Guards { size, .. }, _) if self.japanese() => match size {
                Span::Small => "Ko-sode",
                Span::Medium => "Chū-sode",
                Span::Large => "Ō-sode",
            },
            (ShouldersPiece::Pauldrons { .. }, _) if self.adornment == Adornment::Skull => {
                "Skull Pauldrons"
            }
            (ShouldersPiece::Pauldrons { .. }, _)
                if matches!(self.adornment, Adornment::Spikes(_)) =>
            {
                "Spiked Pauldrons"
            }
            (
                ShouldersPiece::Pauldrons {
                    cut: Cut::GreatLeft,
                    ..
                },
                _,
            ) => "Great Pauldrons",
            (ShouldersPiece::Pauldrons { .. }, C::Leather) if self.finish == Finish::Studded => {
                "Studded Leather Pauldrons"
            }
            (ShouldersPiece::Pauldrons { .. }, C::Leather) => "Leather Pauldrons",
            (ShouldersPiece::Pauldrons { .. }, C::Scale) => "Scale Pauldrons",
            (ShouldersPiece::Pauldrons { .. }, C::MailAndPlate) => "Plated Mail Pauldrons",
            (ShouldersPiece::Pauldrons { .. }, C::Plate) if self.style == Style::Gothic => {
                "Gothic Pauldrons"
            }
            (ShouldersPiece::Spaulders { .. }, _) if self.dendra() => "Dendra Shoulder Guards",
            (ShouldersPiece::Spaulders { .. }, _) if self.roman() => "Lorica Shoulder Guards",
            (ShouldersPiece::Spaulders { .. }, C::Leather) => "Leather Spaulders",
            (ShouldersPiece::Guards { .. }, C::Lamellar) => "Lamellar Shoulder Guards",
            (ShouldersPiece::Guards { .. }, C::Scale) => "Scale Shoulder Guards",
            (ShouldersPiece::Guards { .. }, C::PlatesInCloth) => "Brigandine Shoulder Guards",
            (ShouldersPiece::Guards { .. }, C::Leather) if self.finish == Finish::RingMail => {
                "Ring Mail Shoulder Guards"
            }
            (ShouldersPiece::Guards { .. }, C::Leather) => "Leather Shoulder Guards",
            (
                ShouldersPiece::Cape {
                    cut: CapeCut::Yoke, ..
                },
                _,
            ) => "Linothorax Yoke",
            (ShouldersPiece::Cape { .. }, C::Mail) if self.style == Style::Classical => {
                "Mail Doubling"
            }
            (ShouldersPiece::Cape { .. }, C::Scale) if self.style == Style::Classical => {
                "Scale Doubling"
            }
            (ShouldersPiece::Cape { .. }, C::Mail) => "Mail Cape",
            (ShouldersPiece::Cape { .. }, C::Scale) => "Scale Cape",
            (ShouldersPiece::Cape { .. }, _) => "Ring Mail Cape",
            (ShouldersPiece::Discs { .. }, C::MailAndPlate) => "Naplechniki",
            (ShouldersPiece::Discs { .. }, C::Mirror) => "Mirror Shoulder Plates",
            (ShouldersPiece::Wing, _) => "Chukchi Wing",
            _ => return None,
        })
    }

    /// What it is called.
    pub fn name(&self) -> &'static str {
        self.specific_name().unwrap_or(self.generic_name())
    }

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

/// How often each kind is made.
const KINDS: [(ShouldersKind, u64); 7] = [
    (ShouldersKind::Pauldrons, 30),
    (ShouldersKind::Spaulders, 22),
    (ShouldersKind::Guards, 20),
    (ShouldersKind::Cape, 12),
    (ShouldersKind::Discs, 6),
    (ShouldersKind::Ailettes, 7),
    (ShouldersKind::Wing, 3),
];

/// Which constructions a kind is made in, and how often.
fn constructions(kind: ShouldersKind) -> &'static [(Construction, u64)] {
    use Construction as C;
    match kind {
        ShouldersKind::Pauldrons => &[
            (C::Plate, 60),
            (C::Leather, 20),
            (C::Scale, 10),
            (C::MailAndPlate, 10),
        ],
        ShouldersKind::Spaulders => &[(C::Plate, 55), (C::Leather, 25), (C::Laminar, 20)],
        ShouldersKind::Guards => &[
            (C::Lamellar, 35),
            (C::Scale, 20),
            (C::Laminar, 15),
            (C::PlatesInCloth, 15),
            (C::Leather, 10),
            (C::Mail, 5),
        ],
        ShouldersKind::Cape => &[
            (C::Mail, 45),
            (C::Scale, 25),
            (C::Padded, 20),
            (C::Leather, 10),
        ],
        ShouldersKind::Discs => &[(C::MailAndPlate, 40), (C::Plate, 30), (C::Mirror, 30)],
        ShouldersKind::Ailettes => &[(C::Leather, 70), (C::Plate, 30)],
        ShouldersKind::Wing => &[(C::Laminar, 1)],
    }
}

/// Lamellae and lames laced together.
fn laced(construction: Construction) -> bool {
    matches!(construction, Construction::Lamellar | Construction::Laminar)
}

/// The constructions a kind is made in this style. Padded and leather yokes are only
/// Greek.
fn constructions_in(kind: ShouldersKind, style: Style) -> Vec<(Construction, u64)> {
    let list: Vec<_> = constructions(kind)
        .iter()
        .copied()
        .filter(|&(construction, _)| {
            kind != ShouldersKind::Cape
                || construction != Construction::Padded
                || style == Style::Classical
        })
        .collect();
    in_style(&list, style)
}

/// Fantastic things go on rigid tops; a beast's head on eastern-looking pieces and
/// plate pauldrons; nothing on capes, ailettes and the wing.
fn adornments(kind: ShouldersKind, construction: Construction) -> &'static [(Adornment, u64)] {
    use Adornment::{BeastHead, Horns, None, Skull, Spikes};
    match kind {
        ShouldersKind::Pauldrons if construction == Construction::Plate => &[
            (None, 76),
            (Spikes(1), 3),
            (Spikes(3), 5),
            (Spikes(5), 3),
            (Horns, 4),
            (BeastHead, 3),
            (Skull, 6),
        ],
        ShouldersKind::Pauldrons | ShouldersKind::Spaulders => &[
            (None, 82),
            (Spikes(1), 3),
            (Spikes(3), 5),
            (Spikes(5), 2),
            (Horns, 3),
            (Skull, 5),
        ],
        ShouldersKind::Guards => &[(None, 90), (BeastHead, 6), (Skull, 4)],
        ShouldersKind::Discs => &[(None, 85), (Spikes(1), 10), (Skull, 5)],
        _ => &[(None, 1)],
    }
}

/// Leather is plain, tooled or studded; ring mail on guards and round capes.
fn finishes(
    kind: ShouldersKind,
    construction: Construction,
    piece: &ShouldersPiece,
) -> &'static [(Finish, u64)] {
    use Finish::{Plain, RingMail, Studded, Tooled};
    if construction != Construction::Leather || kind == ShouldersKind::Ailettes {
        return &[(Plain, 1)];
    }
    match piece {
        ShouldersPiece::Cape {
            cut: CapeCut::Round,
            ..
        } => &[(RingMail, 1)],
        ShouldersPiece::Guards { .. } => {
            &[(Plain, 30), (Tooled, 15), (Studded, 35), (RingMail, 20)]
        }
        _ => &[(Plain, 40), (Tooled, 25), (Studded, 35)],
    }
}

fn materials(
    kind: ShouldersKind,
    construction: Construction,
    style: Style,
) -> &'static [(Material, u64)] {
    use Construction as C;
    use Material as M;
    match (kind, construction) {
        // Greek plate is bronze.
        (_, C::Plate) if style == Style::Classical => &[(M::Bronze, 1)],
        (ShouldersKind::Wing, _) => &[(M::Wood, 60), (M::HardenedLeather, 40)],
        (ShouldersKind::Ailettes, C::Leather) => &[(M::HardenedLeather, 1)],
        (ShouldersKind::Ailettes, _) => &[(M::Steel, 40), (M::Iron, 30), (M::Wood, 30)],
        (_, C::Plate) => &[
            (M::Steel, 55),
            (M::Iron, 35),
            (M::Chitin, 4),
            (M::Mithril, 3),
            (M::Adamantine, 2),
        ],
        (_, C::Mail) => &[
            (M::Iron, 50),
            (M::Steel, 40),
            (M::Bronze, 5),
            (M::Mithril, 4),
            (M::Adamantine, 1),
        ],
        (_, C::Scale) => &[
            (M::Iron, 30),
            (M::Steel, 25),
            (M::Bronze, 15),
            (M::Horn, 12),
            (M::HardenedLeather, 10),
            (M::Chitin, 3),
            (M::DragonScale, 3),
            (M::Mithril, 2),
        ],
        (_, C::Lamellar) => &[
            (M::Iron, 40),
            (M::Steel, 20),
            (M::HardenedLeather, 25),
            (M::Horn, 8),
            (M::DragonScale, 4),
            (M::Chitin, 3),
        ],
        (_, C::Laminar) => &[
            (M::Iron, 50),
            (M::Steel, 30),
            (M::HardenedLeather, 15),
            (M::Mithril, 5),
        ],
        (_, C::PlatesInCloth | C::MailAndPlate) => &[(M::Steel, 55), (M::Iron, 45)],
        (_, C::Mirror) => &[(M::Steel, 70), (M::Iron, 25), (M::Mithril, 5)],
        (_, C::Leather) => &[
            (M::HardenedLeather, 55),
            (M::BuffLeather, 25),
            (M::Leather, 20),
        ],
        _ => &[(M::Linen, 75), (M::Cotton, 15), (M::Wool, 10)],
    }
}

/// Cloth over plates in cloth, and lacing through lamellae and lames.
fn needs_cloth(construction: Construction) -> bool {
    matches!(
        construction,
        Construction::PlatesInCloth | Construction::Lamellar | Construction::Laminar
    )
}

fn cloths(construction: Construction) -> &'static [(Material, u64)] {
    use Material as M;
    match construction {
        Construction::PlatesInCloth => &[(M::Linen, 40), (M::Wool, 35), (M::Silk, 25)],
        // Lacing.
        _ => &[(M::Silk, 50), (M::Leather, 50)],
    }
}

fn may_have_trim(kind: ShouldersKind, construction: Construction, material: Material) -> bool {
    let metal = matches!(
        material,
        Material::Steel | Material::Iron | Material::Mithril
    );
    match kind {
        ShouldersKind::Pauldrons | ShouldersKind::Spaulders | ShouldersKind::Discs => {
            metal && matches!(construction, Construction::Plate | Construction::Mirror)
        }
        ShouldersKind::Cape => construction == Construction::Mail,
        _ => false,
    }
}

/// How the cop and lames of plate look by style: Gothic and Kastenbrust pairs are
/// spaulders more often, Milanese pauldrons are large and often uneven.
fn pauldron_cut(style: Style) -> &'static [(Cut, u64)] {
    match style {
        Style::Milanese => &[
            (Cut::Matched, 40),
            (Cut::GreatLeft, 40),
            (Cut::LanceCut, 20),
        ],
        _ => &[(Cut::Matched, 80), (Cut::GreatLeft, 8), (Cut::LanceCut, 12)],
    }
}

fn kinds_for(style: Style) -> Vec<(ShouldersKind, u64)> {
    let fantasy = matches!(style, Style::Elven | Style::Dwarven);
    KINDS
        .iter()
        .map(|&(kind, weight)| {
            let weight = match (kind, style) {
                // Ailettes are European, the wing from the far north-east of the steppe.
                (ShouldersKind::Ailettes, _) if !style.european() && !fantasy => 0,
                (ShouldersKind::Wing, _) if style != Style::Steppe => 0,
                (ShouldersKind::Spaulders, Style::Gothic | Style::Kastenbrust) => weight * 2,
                (
                    ShouldersKind::Pauldrons,
                    Style::Milanese | Style::Maximilian | Style::Greenwich,
                ) => weight * 2,
                (ShouldersKind::Guards, _) if !style.european() && !fantasy => weight * 3,
                (ShouldersKind::Cape | ShouldersKind::Spaulders, Style::Classical) => weight * 3,
                (ShouldersKind::Discs, Style::Steppe | Style::IndoPersian) => weight * 4,
                _ => weight,
            };
            (kind, weight)
        })
        .filter(|&(_, weight)| weight > 0)
        .collect()
}

fn roll_piece(
    rng: &mut Rng,
    kind: ShouldersKind,
    construction: Construction,
    style: Style,
) -> ShouldersPiece {
    use Construction as C;
    let chance = |rng: &mut Rng, one_in: u64| rng.range_u64(0, one_in) == 0;
    match kind {
        ShouldersKind::Pauldrons => {
            let plate = construction == C::Plate;
            let hanging = !matches!(construction, C::Plate | C::Leather);
            let cut = if plate {
                rng.pick_weighted(pauldron_cut(style))
            } else {
                Cut::Matched
            };
            let wings = if hanging {
                Wings::Small
            } else if style == Style::Milanese || cut == Cut::GreatLeft {
                Wings::Large
            } else {
                rng.pick_weighted(&[(Wings::Small, 60), (Wings::Large, 40)])
            };
            let lames = if hanging {
                0
            } else {
                rng.range_u64(3, 7) as u8
            };
            let upper_lames = if plate { rng.range_u64(0, 3) as u8 } else { 0 };
            let haute_piece = if plate {
                rng.pick_weighted(&[
                    (HautePiece::None, 60),
                    (HautePiece::Low, 25),
                    (HautePiece::High, 15),
                ])
            } else {
                HautePiece::None
            };
            let gardbrace = plate && chance(rng, 5);
            let besagews = wings == Wings::Small && !hanging && chance(rng, 3);
            ShouldersPiece::Pauldrons {
                cut,
                wings,
                lames,
                upper_lames,
                haute_piece,
                gardbrace,
                besagews,
            }
        }
        ShouldersKind::Spaulders => {
            let dendra = construction == C::Plate && style == Style::Classical;
            let lames = if dendra { 0 } else { rng.range_u64(2, 6) as u8 };
            let besagews = matches!(construction, C::Plate | C::Leather)
                && !dendra
                && (style == Style::Gothic || style == Style::Kastenbrust || chance(rng, 3));
            let pteruges = if matches!(construction, C::Laminar | C::Leather) {
                rng.pick_weighted(&[(0, 50), (1, 35), (2, 15)])
            } else {
                0
            };
            ShouldersPiece::Spaulders {
                lames,
                besagews,
                pteruges,
            }
        }
        ShouldersKind::Guards => {
            let size =
                rng.pick_weighted(&[(Span::Small, 30), (Span::Medium, 40), (Span::Large, 30)]);
            let (fewest, most) = size.rows();
            let rows = rng.range_u64(fewest as u64, most as u64 + 1) as u8;
            let shape = rng.pick_weighted(&[
                (GuardShape::Straight, 60),
                (GuardShape::Tapered, 25),
                (GuardShape::Flared, 15),
            ]);
            let lacing = if laced(construction) {
                rng.pick_weighted(&[(Lacing::Dense, 70), (Lacing::Spaced, 30)])
            } else {
                Lacing::Dense
            };
            ShouldersPiece::Guards {
                size,
                shape,
                rows,
                lacing,
            }
        }
        ShouldersKind::Cape => ShouldersPiece::Cape {
            cut: match construction {
                C::Padded => CapeCut::Yoke,
                C::Leather if style == Style::Classical => {
                    rng.pick_weighted(&[(CapeCut::Round, 40), (CapeCut::Yoke, 60)])
                }
                _ => CapeCut::Round,
            },
            length: rng.pick_weighted(&[(Length::Short, 45), (Length::Long, 55)]),
        },
        ShouldersKind::Discs => ShouldersPiece::Discs {
            diameter: 110 + 10 * rng.range_u64(0, 6) as u16,
            boss: chance(rng, 2),
        },
        ShouldersKind::Ailettes => ShouldersPiece::Ailettes {
            shape: rng.pick_weighted(&[
                (AiletteShape::Rectangle, 55),
                (AiletteShape::Rounded, 15),
                (AiletteShape::Shield, 20),
                (AiletteShape::Lozenge, 10),
            ]),
            size: rng.pick_weighted(&[(Span::Small, 30), (Span::Medium, 45), (Span::Large, 25)]),
            heraldry: Heraldry::roll(rng),
        },
        ShouldersKind::Wing => ShouldersPiece::Wing,
    }
}

/// Rolls a pair of a kind, or any kind. A set passes its style; a pair on its own
/// rolls one. Each choice comes from what the ones before allow, so nothing is thrown
/// away.
pub fn roll(rng: &mut Rng, kind: Option<ShouldersKind>, style: Option<Style>) -> ShouldersForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    let kind = kind.unwrap_or_else(|| rng.pick_weighted(&kinds_for(style)));
    let construction = rng.pick_weighted(&constructions_in(kind, style));
    let material = rng.pick_weighted(materials(kind, construction, style));
    let piece = roll_piece(rng, kind, construction, style);
    let finish = rng.pick_weighted(finishes(kind, construction, &piece));
    let adornment = rng.pick_weighted(adornments(kind, construction));
    let cloth = needs_cloth(construction).then(|| rng.pick_weighted(cloths(construction)));
    let trim = (may_have_trim(kind, construction, material) && rng.range_u64(0, 5) == 0)
        .then_some(Material::Brass);
    ShouldersForm {
        piece,
        construction,
        adornment,
        finish,
        material,
        cloth,
        trim,
        style,
    }
}

#[cfg(test)]
mod tests;
