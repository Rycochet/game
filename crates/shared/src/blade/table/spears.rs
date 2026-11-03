//! Spears: rows of the blade table.

use super::{
    BLANK, BladeWidth, COMMON, DOUBLE, Family, GENERIC, Guard, KNOWN, LEAF, ONE, Row, STRAIGHT,
    Section, THRUST, TWO, Tips,
};

/// Spears, in the order they are rolled from.
pub(super) const SPEARS: &[Row] = &[
    Row {
        name: "Spear",
        family: Family::Spear,
        blade: (100, 300),
        handle: (1500, 2200),
        ..GENERIC
    },
    Row {
        name: "Lance",
        family: Family::Spear,
        weight: COMMON,
        blade: (80, 150),
        handle: (2500, 3900),
        edges: DOUBLE,
        hands: ONE,
        guard: &[Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Javelin",
        family: Family::Spear,
        weight: COMMON,
        blade: (50, 150),
        handle: (1700, 2200),
        hands: ONE,
        width: &[BladeWidth::Narrow],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Harpoon",
        family: Family::Spear,
        weight: COMMON,
        blade: (100, 200),
        handle: (1200, 2000),
        tips: &[Tips::Barbed],
        ..BLANK
    },
    Row {
        name: "Trident",
        family: Family::Spear,
        weight: COMMON,
        blade: (150, 250),
        handle: (1200, 2000),
        edges: THRUST,
        tips: &[Tips::ThreeTines],
        ..BLANK
    },
    Row {
        name: "Pike",
        family: Family::Spear,
        weight: COMMON,
        blade: (150, 250),
        handle: (2800, 6800),
        edges: DOUBLE,
        hands: TWO,
        shape: LEAF,
        tips: &[Tips::Single],
        langets: &[true],
        ..BLANK
    },
    Row {
        name: "Assegai",
        family: Family::Spear,
        weight: KNOWN,
        blade: (200, 300),
        handle: (600, 700),
        hands: ONE,
        edges: DOUBLE,
        shape: LEAF,
        ..BLANK
    },
    Row {
        name: "Pilum",
        family: Family::Spear,
        weight: KNOWN,
        blade: (250, 350),
        handle: (1000, 1400),
        edges: THRUST,
        width: &[BladeWidth::Narrow],
        section: &[Section::Diamond, Section::Triangular],
        hands: ONE,
        tips: &[Tips::Single, Tips::Barbed],
        ..BLANK
    },
    Row {
        name: "Sarissa",
        family: Family::Spear,
        weight: KNOWN,
        blade: (200, 350),
        handle: (4500, 6800),
        edges: DOUBLE,
        hands: TWO,
        shape: LEAF,
        langets: &[false],
        ..BLANK
    },
    Row {
        name: "Yari",
        family: Family::Spear,
        blade: (150, 600),
        handle: (1000, 6000),
        edges: DOUBLE,
        curve: STRAIGHT,
        hands: TWO,
        section: &[Section::Triangular, Section::Diamond],
        langets: &[false],
        ..BLANK
    },
];
