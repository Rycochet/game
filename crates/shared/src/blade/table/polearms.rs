//! Polearms: rows of the blade table.

use super::{
    BLANK, BROAD_EDGE, Back, BladeShape, BladeWidth, COMMON, DOUBLE, Family, GENERIC, Guard,
    INWARD, KNOWN, LEAF, RARE, Row, SIDE_HEAD, SINGLE, SLIGHT, STRONG, SidePart, TWO, Top,
};

/// Polearms, in the order they are rolled from.
pub(super) const POLEARMS: &[Row] = &[
    // Polearms: blades along the pole
    Row {
        name: "Polearm",
        family: Family::Polearm,
        weight: KNOWN,
        blade: (350, 600),
        handle: (1200, 2500),
        edges: SINGLE,
        ..GENERIC
    },
    Row {
        name: "Glaive",
        family: Family::Polearm,
        weight: COMMON,
        blade: (400, 500),
        handle: (1700, 2000),
        edges: SINGLE,
        curve: SLIGHT,
        shape: &[BladeShape::Widening],
        ..BLANK
    },
    Row {
        name: "Naginata",
        family: Family::Polearm,
        weight: KNOWN,
        blade: (300, 600),
        handle: (1200, 2400),
        edges: SINGLE,
        curve: SLIGHT,
        guard: &[Guard::Disc],
        ..BLANK
    },
    Row {
        name: "Spontoon",
        family: Family::Polearm,
        blade: (150, 300),
        handle: (1600, 2100),
        edges: DOUBLE,
        shape: LEAF,
        side_one: &[SidePart::Lug],
        side_two: &[SidePart::Lug],
        ..BLANK
    },
    Row {
        name: "Guandao",
        family: Family::Polearm,
        weight: RARE,
        blade: (300, 460),
        handle: (1000, 1800),
        edges: SINGLE,
        curve: STRONG,
        width: &[BladeWidth::Broad],
        ..BLANK
    },
    Row {
        name: "Nagamaki",
        family: Family::Polearm,
        weight: RARE,
        blade: (600, 900),
        handle: (600, 900),
        edges: SINGLE,
        curve: SLIGHT,
        hands: TWO,
        guard: &[Guard::Disc],
        ..BLANK
    },
    // Polearms: heads at the side of the pole
    Row {
        name: "Halberd",
        family: Family::Polearm,
        weight: COMMON,
        blade: (200, 300),
        handle: (1800, 2200),
        side_two: &[SidePart::Spike, SidePart::Hook],
        top: &[Top::Spike],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Poleaxe",
        family: Family::Polearm,
        blade: BROAD_EDGE,
        handle: (1500, 2000),
        side_one: &[SidePart::AxeBlade],
        side_two: &[SidePart::Spike, SidePart::Beak],
        back: &[Back::Hammer, Back::Spike],
        top: &[Top::Spike],
        hands: TWO,
        langets: &[true],
        ..SIDE_HEAD
    },
    Row {
        name: "Bill",
        family: Family::Polearm,
        blade: (200, 400),
        handle: (1200, 2700),
        curve: INWARD,
        side_one: &[SidePart::Hook],
        side_two: &[SidePart::Spike],
        top: &[Top::Spike],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Lochaber Axe",
        family: Family::Polearm,
        weight: RARE,
        blade: (400, 500),
        handle: (1500, 1800),
        top: &[Top::None],
        side_two: &[SidePart::Hook],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Voulge",
        family: Family::Polearm,
        weight: RARE,
        blade: (300, 600),
        handle: (1800, 2500),
        side_two: &[SidePart::None],
        top: &[Top::Blade],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Guisarme",
        family: Family::Polearm,
        weight: RARE,
        blade: (200, 400),
        handle: (1800, 2500),
        curve: INWARD,
        side_one: &[SidePart::Hook],
        side_two: &[SidePart::Spike, SidePart::None],
        top: &[Top::Blade],
        hands: TWO,
        ..SIDE_HEAD
    },
    Row {
        name: "Bardiche",
        family: Family::Polearm,
        weight: RARE,
        blade: (600, 700),
        handle: (1200, 1501),
        curve: SLIGHT,
        side_two: &[SidePart::None],
        top: &[Top::None],
        hands: TWO,
        ..SIDE_HEAD
    },
];
