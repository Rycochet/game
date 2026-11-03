//! Long swords: rows of the blade table.

use super::{
    BLANK, BladeWidth, COMMON, Curve, DOUBLE, EITHER, Family, GENERIC, Guard, Pommel, RARE,
    Ricasso, Row, SINGLE, SLIGHT, STRAIGHT, TWO,
};

/// Long swords, in the order they are rolled from.
pub(super) const LONG_SWORDS: &[Row] = &[
    Row {
        name: "Longsword",
        family: Family::LongSword,
        blade: (800, 1100),
        handle: (200, 300),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cross],
        ..GENERIC
    },
    Row {
        name: "Greatsword",
        family: Family::LongSword,
        blade: (1000, 1500),
        handle: (300, 400),
        edges: DOUBLE,
        curve: STRAIGHT,
        hands: TWO,
        guard: &[Guard::Cross],
        ..GENERIC
    },
    Row {
        name: "Claymore",
        family: Family::LongSword,
        weight: COMMON,
        blade: (1000, 1150),
        handle: (300, 360),
        edges: DOUBLE,
        curve: STRAIGHT,
        hands: TWO,
        guard: &[Guard::Cross],
        pommel: &[Pommel::Wheel, Pommel::Disc],
        width: &[BladeWidth::Normal],
        ricasso: &[Ricasso::None],
        ..BLANK
    },
    Row {
        name: "Bastard Sword",
        family: Family::LongSword,
        weight: COMMON,
        blade: (750, 950),
        handle: (200, 280),
        edges: DOUBLE,
        curve: STRAIGHT,
        hands: EITHER,
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Odachi",
        family: Family::LongSword,
        weight: RARE,
        blade: (900, 1200),
        handle: (300, 400),
        edges: SINGLE,
        curve: SLIGHT,
        hands: TWO,
        guard: &[Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Zweihänder",
        family: Family::LongSword,
        weight: RARE,
        blade: (1000, 1500),
        handle: (300, 400),
        edges: DOUBLE,
        curve: &[Curve::Straight, Curve::Wavy],
        hands: TWO,
        ricasso: &[Ricasso::WithHooks],
        guard: &[Guard::Cross],
        ..BLANK
    },
];
