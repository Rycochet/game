//! Axes: rows of the blade table.

use super::{
    Back, BladeWidth, Build, COMMON, Family, GENERIC, HandleShape, ONE, RARE, Row, SIDE_HEAD,
    SINGLE, SLIGHT, STRONG, SidePart, TWO,
};

/// Axes, in the order they are rolled from.
pub(super) const AXES: &[Row] = &[
    Row {
        name: "Axe",
        family: Family::Axe,
        blade: (80, 200),
        handle: (300, 900),
        edges: SINGLE,
        side_one: &[SidePart::AxeBlade],
        build: Build::Side,
        ..GENERIC
    },
    Row {
        name: "Battle Axe",
        family: Family::Axe,
        blade: (150, 250),
        handle: (600, 1200),
        edges: SINGLE,
        side_one: &[SidePart::AxeBlade],
        build: Build::Side,
        ..GENERIC
    },
    Row {
        name: "Hatchet",
        family: Family::Axe,
        weight: COMMON,
        blade: (60, 120),
        handle: (250, 400),
        hands: ONE,
        back: &[Back::None],
        side_two: &[SidePart::None],
        ..SIDE_HEAD
    },
    Row {
        name: "Tomahawk",
        family: Family::Axe,
        weight: COMMON,
        blade: (60, 101),
        handle: (400, 611),
        hands: ONE,
        back: &[Back::Spike, Back::Pipe],
        ..SIDE_HEAD
    },
    Row {
        name: "Broadaxe",
        family: Family::Axe,
        blade: (150, 300),
        handle: (600, 800),
        width: &[BladeWidth::Broad],
        back: &[Back::None],
        side_two: &[SidePart::None],
        ..SIDE_HEAD
    },
    Row {
        name: "Labrys",
        family: Family::Axe,
        blade: (100, 250),
        handle: (600, 1000),
        side_two: &[SidePart::AxeBlade],
        ..SIDE_HEAD
    },
    Row {
        name: "Throwing Axe",
        family: Family::Axe,
        blade: (80, 120),
        handle: (300, 450),
        hands: ONE,
        handle_shape: &[HandleShape::Straight],
        back: &[Back::None],
        width: &[BladeWidth::Narrow],
        ..SIDE_HEAD
    },
    Row {
        name: "Francisca",
        family: Family::Axe,
        weight: RARE,
        blade: (90, 110),
        handle: (400, 451),
        hands: ONE,
        curve: SLIGHT,
        back: &[Back::None],
        ..SIDE_HEAD
    },
    Row {
        name: "Tabar",
        family: Family::Axe,
        weight: RARE,
        blade: (150, 250),
        handle: (600, 1000),
        curve: STRONG,
        ..SIDE_HEAD
    },
    Row {
        name: "Dane Axe",
        family: Family::Axe,
        weight: RARE,
        blade: (200, 301),
        handle: (900, 1701),
        hands: TWO,
        curve: SLIGHT,
        width: &[BladeWidth::Narrow],
        back: &[Back::None],
        ..SIDE_HEAD
    },
];
