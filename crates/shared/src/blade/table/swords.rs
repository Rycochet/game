//! Swords: rows of the blade table.

use super::{
    BLANK, BladeShape, BladeWidth, COMMON, Curve, DOUBLE, Family, GENERIC, Guard, KNOWN, ONE,
    Point, Pommel, RARE, Row, SINGLE, SLIGHT, STRAIGHT, STRONG, THRUST, TWO,
};

/// Swords, in the order they are rolled from.
pub(super) const SWORDS: &[Row] = &[
    Row {
        name: "Sword",
        family: Family::Sword,
        blade: (600, 1000),
        handle: (100, 200),
        ..GENERIC
    },
    Row {
        name: "Sabre",
        family: Family::Sword,
        weight: COMMON,
        blade: (700, 900),
        handle: (120, 150),
        edges: SINGLE,
        curve: &[Curve::Slight, Curve::Strong],
        guard: &[Guard::KnuckleBow],
        ..BLANK
    },
    Row {
        name: "Rapier",
        family: Family::Sword,
        weight: COMMON,
        blade: (1000, 1300),
        handle: (120, 150),
        width: &[BladeWidth::Narrow],
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cup, Guard::KnuckleBow],
        ..BLANK
    },
    Row {
        name: "Scimitar",
        family: Family::Sword,
        weight: COMMON,
        blade: (750, 900),
        handle: (120, 150),
        edges: SINGLE,
        curve: STRONG,
        guard: &[Guard::Cross],
        width: &[BladeWidth::Normal, BladeWidth::Broad],
        ..BLANK
    },
    Row {
        name: "Broadsword",
        family: Family::Sword,
        weight: COMMON,
        blade: (750, 900),
        handle: (120, 150),
        edges: DOUBLE,
        curve: STRAIGHT,
        width: &[BladeWidth::Broad],
        guard: &[Guard::Basket],
        ..BLANK
    },
    Row {
        name: "Katana",
        family: Family::Sword,
        weight: COMMON,
        blade: (600, 800),
        handle: (250, 300),
        edges: SINGLE,
        curve: SLIGHT,
        hands: TWO,
        guard: &[Guard::Disc],
        pommel: &[Pommel::Plain],
        ..BLANK
    },
    Row {
        name: "Falchion",
        family: Family::Sword,
        weight: KNOWN,
        blade: (700, 800),
        handle: (120, 150),
        edges: SINGLE,
        curve: SLIGHT,
        shape: &[BladeShape::Widening],
        width: &[BladeWidth::Broad],
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Small Sword",
        family: Family::Sword,
        weight: KNOWN,
        blade: (600, 850),
        handle: (100, 130),
        edges: THRUST,
        curve: STRAIGHT,
        width: &[BladeWidth::Narrow],
        guard: &[Guard::Cup],
        ..BLANK
    },
    Row {
        name: "Tulwar",
        family: Family::Sword,
        weight: KNOWN,
        blade: (700, 850),
        handle: (100, 130),
        edges: SINGLE,
        curve: STRONG,
        pommel: &[Pommel::Disc],
        guard: &[Guard::Cross, Guard::KnuckleBow],
        ..BLANK
    },
    Row {
        name: "Spatha",
        family: Family::Sword,
        blade: (600, 750),
        handle: (180, 200),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: &[BladeShape::Parallel],
        guard: &[Guard::Cross, Guard::None],
        ..BLANK
    },
    Row {
        name: "Backsword",
        family: Family::Sword,
        weight: RARE,
        blade: (750, 900),
        handle: (120, 150),
        edges: SINGLE,
        curve: STRAIGHT,
        guard: &[Guard::KnuckleBow, Guard::Basket],
        ..BLANK
    },
    Row {
        name: "Arming Sword",
        family: Family::Sword,
        weight: RARE,
        blade: (690, 810),
        handle: (100, 200),
        edges: DOUBLE,
        curve: STRAIGHT,
        guard: &[Guard::Cross],
        hands: ONE,
        pommel: &[Pommel::Wheel, Pommel::Disc],
        ..BLANK
    },
    Row {
        name: "Shamshir",
        family: Family::Sword,
        weight: RARE,
        blade: (750, 900),
        handle: (100, 130),
        edges: SINGLE,
        curve: STRONG,
        width: &[BladeWidth::Narrow],
        guard: &[Guard::Cross],
        ..BLANK
    },
    Row {
        name: "Jian",
        family: Family::Sword,
        blade: (450, 800),
        handle: (150, 250),
        edges: DOUBLE,
        curve: STRAIGHT,
        shape: &[BladeShape::Parallel, BladeShape::Tapered],
        guard: &[Guard::Disc, Guard::Cross],
        pommel: &[Pommel::Plain, Pommel::Ring],
        ..BLANK
    },
    Row {
        name: "Dao",
        family: Family::Sword,
        blade: (650, 900),
        handle: (150, 200),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::Disc],
        point: &[Point::Clip, Point::Flared],
        ..BLANK
    },
];
