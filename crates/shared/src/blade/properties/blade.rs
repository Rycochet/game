//! Rolling the blade itself: its width, edges, curve, point, shape, section and fuller.

use super::Holding;
use crate::blade::table::Row;
use crate::blade::{BladeShape, BladeWidth, Curve, Edges, GREAT, Point, Section, pick};
use crate::rng::Rng;

/// What is rolled for the blade itself.
pub(super) struct BladeRolls {
    pub(super) width: BladeWidth,
    pub(super) edges: Edges,
    pub(super) curve: Curve,
    pub(super) point: Point,
    pub(super) shape: BladeShape,
    pub(super) section: Section,
    pub(super) fuller: bool,
}

pub(super) fn roll_blade_itself(
    rng: &mut Rng,
    row: &Row,
    holding: Holding,
    blade: u16,
) -> BladeRolls {
    let Holding {
        is_side,
        held,
        on_a_grip,
    } = holding;
    // A blade this long needs to be slim to be held in one hand.
    let must_be_narrow = on_a_grip && blade >= GREAT.0;
    let width = if must_be_narrow {
        BladeWidth::Narrow
    } else {
        pick(
            rng,
            row.width,
            &[
                (BladeWidth::Narrow, 25),
                (BladeWidth::Normal, 60),
                (BladeWidth::Broad, 15),
            ],
        )
    };
    let edges = pick(
        rng,
        row.edges,
        &match (is_side, held) {
            (true, _) => [
                (Edges::Single, 100),
                (Edges::Double, 0),
                (Edges::PointOnly, 0),
            ],
            (false, true) => [
                (Edges::Single, 45),
                (Edges::Double, 45),
                (Edges::PointOnly, 10),
            ],
            (false, false) => [
                (Edges::Single, 30),
                (Edges::Double, 60),
                (Edges::PointOnly, 10),
            ],
        },
    );
    let curve = pick(
        rng,
        row.curve,
        &if held {
            [
                (Curve::Straight, 55),
                (Curve::Slight, 25),
                (Curve::Strong, 10),
                (Curve::Inward, 7),
                (Curve::Wavy, 3),
            ]
        } else {
            [
                (Curve::Straight, 80),
                (Curve::Slight, 20),
                (Curve::Strong, 0),
                (Curve::Inward, 0),
                (Curve::Wavy, 0),
            ]
        },
    );
    let point = pick(
        rng,
        row.point,
        &if is_side {
            [
                (Point::None, 100),
                (Point::Sharp, 0),
                (Point::Rounded, 0),
                (Point::Hooked, 0),
                (Point::Clip, 0),
                (Point::Notched, 0),
                (Point::Flared, 0),
            ]
        } else {
            [
                (Point::None, 5),
                (Point::Sharp, 75),
                (Point::Rounded, 5),
                (Point::Hooked, 5),
                (Point::Clip, 4),
                (Point::Notched, 3),
                (Point::Flared, 3),
            ]
        },
    );
    let shape = pick(
        rng,
        row.shape,
        &if is_side {
            [
                (BladeShape::Widening, 100),
                (BladeShape::Parallel, 0),
                (BladeShape::Tapered, 0),
                (BladeShape::Leaf, 0),
            ]
        } else {
            [
                (BladeShape::Widening, 5),
                (BladeShape::Parallel, 40),
                (BladeShape::Tapered, 40),
                (BladeShape::Leaf, 15),
            ]
        },
    );
    let section = pick(
        rng,
        row.section,
        &if is_side {
            [
                (Section::Flat, 100),
                (Section::Lenticular, 0),
                (Section::Diamond, 0),
                (Section::Midrib, 0),
                (Section::Triangular, 0),
            ]
        } else {
            [
                (Section::Flat, 20),
                (Section::Lenticular, 40),
                (Section::Diamond, 25),
                (Section::Midrib, 10),
                (Section::Triangular, 5),
            ]
        },
    );
    let fuller = pick(
        rng,
        row.fuller,
        &[(true, u64::from(held) * 40), (false, 60)],
    );
    BladeRolls {
        width,
        edges,
        curve,
        point,
        shape,
        section,
        fuller,
    }
}
