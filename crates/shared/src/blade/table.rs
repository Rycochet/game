//! The named weapons. Sizes and popularity came from research recorded in
//! `docs/context/research/bladed-weapons.md`. A generic row is only there to roll from:
//! such a weapon takes the generic name of its cell. A specific row's name goes on
//! anything that fits it, however it was rolled.

mod axes;
mod daggers;
mod knives;
mod long_swords;
mod polearms;
mod short_swords;
mod spears;
mod swords;

use axes::AXES;
use daggers::DAGGERS;
use knives::KNIVES;
use long_swords::LONG_SWORDS;
use polearms::POLEARMS;
use short_swords::SHORT_SWORDS;
use spears::SPEARS;
use swords::SWORDS;

use super::{
    BROAD_EDGE, Back, BladeForm, BladeShape, BladeWidth, Build, CELLS, Curve, Edges, Family, Guard,
    HandleShape, Hands, Point, Pommel, Ricasso, Section, SidePart, Span, Tips, Top, overlap,
    within,
};

/// What a name asks for. An empty list means anything will do.
#[derive(Clone, Copy, Debug)]
pub(super) struct Row {
    pub name: &'static str,
    pub family: Family,
    pub generic: bool,
    /// How often this is rolled in its family, from how well known the name is.
    pub weight: u64,
    pub build: Build,
    pub blade: Span,
    pub handle: Span,
    pub width: &'static [BladeWidth],
    pub edges: &'static [Edges],
    pub curve: &'static [Curve],
    pub point: &'static [Point],
    pub shape: &'static [BladeShape],
    pub section: &'static [Section],
    pub fuller: &'static [bool],
    pub hands: &'static [Hands],
    pub guard: &'static [Guard],
    pub pommel: &'static [Pommel],
    pub ricasso: &'static [Ricasso],
    pub langets: &'static [bool],
    pub handle_shape: &'static [HandleShape],
    pub side_one: &'static [SidePart],
    pub side_two: &'static [SidePart],
    pub top: &'static [Top],
    pub back: &'static [Back],
    pub tips: &'static [Tips],
}

impl Row {
    /// Every part of a size grid this row's lengths reach.
    pub fn places(&self) -> impl Iterator<Item = (Span, Span)> + '_ {
        let cells = CELLS.iter().filter(|cell| cell.build == self.build);
        cells.filter_map(|cell| {
            let blade = overlap(self.blade, cell.blade)?;
            let handle = overlap(self.handle, cell.handle)?;
            Some((blade, handle))
        })
    }

    pub fn fits(&self, form: &BladeForm) -> bool {
        fn open<T: PartialEq>(allowed: &[T], value: T) -> bool {
            allowed.is_empty() || allowed.contains(&value)
        }
        self.build == form.build
            && within(self.blade, form.blade)
            && within(self.handle, form.handle)
            && open(self.width, form.width)
            && open(self.edges, form.edges)
            && open(self.curve, form.curve)
            && open(self.point, form.point)
            && open(self.shape, form.shape)
            && open(self.section, form.section)
            && open(self.fuller, form.fuller)
            && open(self.hands, form.hands)
            && open(self.guard, form.guard)
            && open(self.pommel, form.pommel)
            && open(self.ricasso, form.ricasso)
            && open(self.langets, form.langets)
            && open(self.handle_shape, form.handle_shape)
            && open(self.side_one, form.sides[0])
            && open(self.side_two, form.sides[1])
            && open(self.top, form.top)
            && open(self.back, form.back)
            && open(self.tips, form.tips)
    }

    /// How many things this asks for, so the name asking for the most wins.
    pub fn specifics(&self) -> usize {
        [
            self.width.len(),
            self.edges.len(),
            self.curve.len(),
            self.point.len(),
            self.shape.len(),
            self.section.len(),
            self.fuller.len(),
            self.hands.len(),
            self.guard.len(),
            self.pommel.len(),
            self.ricasso.len(),
            self.langets.len(),
            self.handle_shape.len(),
            self.side_one.len(),
            self.side_two.len(),
            self.top.len(),
            self.back.len(),
            self.tips.len(),
        ]
        .iter()
        .filter(|allowed| **allowed > 0)
        .count()
    }
}

/// How often a row is rolled, by its popularity in books.
const COMMON: u64 = 8;
const KNOWN: u64 = 4;
const RARE: u64 = 2;
/// A family's generic row: most of what is made has no special name.
const USUAL: u64 = 16;

const BLANK: Row = Row {
    name: "",
    family: Family::Knife,
    generic: false,
    weight: KNOWN,
    build: Build::Along,
    blade: (0, 0),
    handle: (0, 0),
    width: &[],
    edges: &[],
    curve: &[],
    point: &[],
    shape: &[],
    section: &[],
    fuller: &[],
    hands: &[],
    guard: &[],
    pommel: &[],
    ricasso: &[],
    langets: &[],
    handle_shape: &[],
    side_one: &[],
    side_two: &[],
    top: &[],
    back: &[],
    tips: &[],
};

const GENERIC: Row = Row {
    generic: true,
    weight: USUAL,
    ..BLANK
};

/// An axe's or side-headed polearm's defaults.
const SIDE_HEAD: Row = Row {
    build: Build::Side,
    edges: &[Edges::Single],
    side_one: &[SidePart::AxeBlade],
    ..BLANK
};

const ONE: &[Hands] = &[Hands::One];
const TWO: &[Hands] = &[Hands::Two];
const EITHER: &[Hands] = &[Hands::OneOrTwo];
const SINGLE: &[Edges] = &[Edges::Single];
const DOUBLE: &[Edges] = &[Edges::Double];
const THRUST: &[Edges] = &[Edges::PointOnly];
const STRAIGHT: &[Curve] = &[Curve::Straight];
const SLIGHT: &[Curve] = &[Curve::Slight];
const STRONG: &[Curve] = &[Curve::Strong];
const INWARD: &[Curve] = &[Curve::Inward];
const LEAF: &[BladeShape] = &[BladeShape::Leaf];

/// Every row, family by family, in the order they are rolled from: best known
/// first within each family.
pub(super) fn rows() -> impl Iterator<Item = &'static Row> {
    [
        KNIVES,
        DAGGERS,
        SHORT_SWORDS,
        SWORDS,
        LONG_SWORDS,
        AXES,
        SPEARS,
        POLEARMS,
    ]
    .into_iter()
    .flatten()
}
