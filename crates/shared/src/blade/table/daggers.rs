//! Daggers: rows of the blade table.

use super::{
    BLANK, BladeShape, BladeWidth, COMMON, Curve, DOUBLE, Family, GENERIC, Guard, HandleShape,
    KNOWN, LEAF, Pommel, RARE, Row, SINGLE, SLIGHT, STRAIGHT, Section, THRUST,
};

/// Daggers, in the order they are rolled from.
pub(super) const DAGGERS: &[Row] = &[
    Row {
        name: "Dagger",
        family: Family::Dagger,
        blade: (180, 300),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        ..GENERIC
    },
    Row {
        name: "Tanto",
        family: Family::Dagger,
        weight: COMMON,
        blade: (150, 300),
        handle: (100, 150),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::None, Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Stiletto",
        family: Family::Dagger,
        weight: COMMON,
        blade: (200, 300),
        handle: (90, 110),
        edges: THRUST,
        curve: STRAIGHT,
        width: &[BladeWidth::Narrow],
        section: &[Section::Diamond],
        ..BLANK
    },
    Row {
        name: "Dirk",
        family: Family::Dagger,
        weight: COMMON,
        blade: (200, 350),
        handle: (100, 130),
        edges: SINGLE,
        curve: STRAIGHT,
        shape: &[BladeShape::Tapered],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Kris",
        family: Family::Dagger,
        weight: COMMON,
        blade: (300, 600),
        handle: (80, 120),
        edges: DOUBLE,
        curve: &[Curve::Wavy],
        handle_shape: &[HandleShape::Angled, HandleShape::Curved],
        guard: &[Guard::None],
        ..BLANK
    },
    Row {
        name: "Poignard",
        family: Family::Dagger,
        weight: KNOWN,
        blade: (150, 300),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        width: &[BladeWidth::Narrow],
        guard: &[Guard::Cross],
        section: &[Section::Diamond, Section::Lenticular],
        ..BLANK
    },
    Row {
        name: "Athame",
        family: Family::Dagger,
        weight: KNOWN,
        blade: (100, 200),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cross],
        pommel: &[Pommel::Plain],
        fuller: &[false],
        shape: &[BladeShape::Leaf, BladeShape::Tapered],
        ..BLANK
    },
    Row {
        name: "Kunai",
        family: Family::Dagger,
        blade: (200, 300),
        handle: (100, 150),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: LEAF,
        guard: &[Guard::None],
        pommel: &[Pommel::Ring],
        ..BLANK
    },
    Row {
        name: "Rondel Dagger",
        family: Family::Dagger,
        blade: (300, 350),
        handle: (100, 200),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Disc],
        pommel: &[Pommel::Disc],
        ..BLANK
    },
    Row {
        name: "Pugio",
        family: Family::Dagger,
        blade: (180, 280),
        handle: (100, 131),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: LEAF,
        width: &[BladeWidth::Broad],
        guard: &[Guard::Cross, Guard::None],
        ..BLANK
    },
    Row {
        name: "Parrying Dagger",
        family: Family::Dagger,
        blade: (350, 400),
        handle: (100, 130),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cup, Guard::KnuckleBow],
        ..BLANK
    },
    Row {
        name: "Misericorde",
        family: Family::Dagger,
        weight: RARE,
        blade: (200, 300),
        handle: (90, 110),
        edges: THRUST,
        curve: STRAIGHT,
        width: &[BladeWidth::Narrow],
        section: &[Section::Triangular],
        guard: &[Guard::None, Guard::Cross],
        ..BLANK
    },
];
