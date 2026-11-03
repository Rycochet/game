//! Knives: rows of the blade table.

use super::{
    BLANK, BladeShape, BladeWidth, COMMON, DOUBLE, Family, GENERIC, Guard, INWARD, ONE, Point,
    Pommel, RARE, Row, SINGLE, SLIGHT, STRAIGHT,
};

/// Knives, in the order they are rolled from.
pub(super) const KNIVES: &[Row] = &[
    Row {
        name: "Knife",
        family: Family::Knife,
        blade: (80, 300),
        handle: (80, 150),
        edges: SINGLE,
        ..GENERIC
    },
    Row {
        name: "Machete",
        family: Family::Knife,
        weight: COMMON,
        blade: (300, 660),
        handle: (100, 150),
        edges: SINGLE,
        curve: STRAIGHT,
        width: &[BladeWidth::Broad],
        guard: &[Guard::None],
        point: &[Point::Rounded, Point::Sharp],
        ..BLANK
    },
    Row {
        name: "Hunting Knife",
        family: Family::Knife,
        weight: COMMON,
        blade: (100, 200),
        handle: (100, 130),
        edges: SINGLE,
        curve: SLIGHT,
        point: &[Point::Sharp],
        guard: &[Guard::None, Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Bowie Knife",
        family: Family::Knife,
        weight: COMMON,
        blade: (240, 301),
        handle: (100, 150),
        edges: SINGLE,
        curve: SLIGHT,
        point: &[Point::Clip],
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Parang",
        family: Family::Knife,
        blade: (250, 910),
        handle: (100, 150),
        edges: SINGLE,
        curve: SLIGHT,
        width: &[BladeWidth::Broad],
        shape: &[BladeShape::Widening],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Throwing Knife",
        family: Family::Knife,
        blade: (150, 300),
        handle: (80, 120),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::None],
        pommel: &[Pommel::None],
        hands: ONE,
        shape: &[BladeShape::Tapered],
        ..BLANK
    },
    Row {
        name: "Kukri",
        family: Family::Knife,
        blade: (260, 380),
        handle: (100, 150),
        edges: SINGLE,
        curve: INWARD,
        shape: &[BladeShape::Widening],
        point: &[Point::Sharp, Point::Notched],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Combat Knife",
        family: Family::Knife,
        blade: (150, 200),
        handle: (100, 130),
        edges: SINGLE,
        curve: STRAIGHT,
        guard: &[Guard::Cross],
        point: &[Point::Sharp, Point::Clip],
        ..BLANK
    },
    Row {
        name: "Seax",
        family: Family::Knife,
        weight: RARE,
        blade: (80, 750),
        handle: (100, 200),
        edges: SINGLE,
        curve: STRAIGHT,
        guard: &[Guard::None],
        pommel: &[Pommel::None, Pommel::Plain],
        point: &[Point::Clip],
        ..BLANK
    },
    Row {
        name: "Fighting Knife",
        family: Family::Knife,
        weight: RARE,
        blade: (150, 250),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: &[BladeShape::Tapered],
        guard: &[Guard::Cross],
        ..BLANK
    },
];
