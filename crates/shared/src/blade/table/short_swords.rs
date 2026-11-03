//! Short swords: rows of the blade table.

use super::{
    BLANK, COMMON, DOUBLE, Family, GENERIC, Guard, INWARD, KNOWN, LEAF, Pommel, RARE, Row, SINGLE,
    SLIGHT, STRAIGHT, STRONG, Section,
};

/// Short swords, in the order they are rolled from.
pub(super) const SHORT_SWORDS: &[Row] = &[
    Row {
        name: "Short Sword",
        family: Family::ShortSword,
        blade: (350, 600),
        handle: (100, 150),
        ..GENERIC
    },
    Row {
        name: "Cutlass",
        family: Family::ShortSword,
        weight: COMMON,
        blade: (610, 760),
        handle: (100, 150),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::Cup],
        ..BLANK
    },
    Row {
        name: "Gladius",
        family: Family::ShortSword,
        weight: COMMON,
        blade: (450, 680),
        handle: (130, 170),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: LEAF,
        guard: &[Guard::None, Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Falcata",
        family: Family::ShortSword,
        weight: KNOWN,
        blade: (350, 550),
        handle: (100, 130),
        edges: SINGLE,
        curve: INWARD,
        pommel: &[Pommel::Hooked],
        ..BLANK
    },
    Row {
        name: "Wakizashi",
        family: Family::ShortSword,
        blade: (300, 600),
        handle: (150, 200),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Yataghan",
        family: Family::ShortSword,
        blade: (550, 650),
        handle: (130, 170),
        edges: SINGLE,
        curve: INWARD,
        guard: &[Guard::None],
        pommel: &[Pommel::Disc, Pommel::Hooked],
        ..BLANK
    },
    Row {
        name: "Kopis",
        family: Family::ShortSword,
        weight: RARE,
        blade: (480, 650),
        handle: (100, 130),
        edges: SINGLE,
        curve: INWARD,
        guard: &[Guard::None, Guard::Cross],
        pommel: &[Pommel::Plain, Pommel::None],
        ..BLANK
    },
    Row {
        name: "Xiphos",
        family: Family::ShortSword,
        weight: RARE,
        blade: (450, 600),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: LEAF,
        section: &[Section::Midrib, Section::Diamond, Section::Lenticular],
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Khopesh",
        family: Family::ShortSword,
        weight: RARE,
        blade: (350, 450),
        handle: (120, 150),
        edges: SINGLE,
        curve: STRONG,
        guard: &[Guard::None],
        ..BLANK
    },
];
