//! The shape of the liquid in a quarter of a cell, and the rings it is drawn from.

use bevy::prelude::*;

use super::{CHANNEL_HALF, CURVE_PIECES, HALF_CELL};

/// A point in a quarter of a cell: distances from its centre, towards the
/// quarter's east-west side and its north-south side.
pub(super) type Point = (f32, f32);

/// The shape of the liquid in a quarter of a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outline {
    /// Floor to both sides: round, about the cell's centre.
    Round,
    /// Half a cell wide, running to the quarter's east-west side.
    AlongEast,
    /// Half a cell wide, running to the quarter's north-south side.
    AlongSouth,
    /// All of it but a round bite of floor at the corner of the cell.
    Notch,
    /// All of it.
    Full,
}

impl Outline {
    /// `beside` and `ahead` say whether the liquid carries on past the
    /// quarter's east-west and north-south sides, `across` past its corner.
    pub(super) fn of(beside: bool, ahead: bool, across: bool) -> Self {
        match (beside, ahead) {
            (false, false) => Self::Round,
            (true, false) => Self::AlongEast,
            (false, true) => Self::AlongSouth,
            (true, true) if across => Self::Full,
            (true, true) => Self::Notch,
        }
    }

    /// The floor beside the liquid in a wall's quarter, if it is the part of
    /// the quarter past a straight shore, as `(east, south)`: the floor there
    /// is drawn like any other beside a wall.
    pub fn straight_shore(self) -> Option<(f32, f32)> {
        match self {
            Self::AlongEast => Some((HALF_CELL, CHANNEL_HALF)),
            Self::AlongSouth => Some((CHANNEL_HALF, HALF_CELL)),
            _ => None,
        }
    }

    /// The part of a wall's foot in this liquid: the part between `from` and
    /// `to` along a line at distance `plane` from the centre, running to the
    /// east-west side (`along_east`) or the north-south side. The bite out of
    /// a `Notch` never reaches the wall.
    pub fn span_along(
        self,
        along_east: bool,
        plane: f32,
        (from, to): (f32, f32),
    ) -> Option<(f32, f32)> {
        let span = match (self, along_east) {
            (Self::Round, _) => None,
            (Self::Full | Self::Notch, _) => Some((from, to)),
            (Self::AlongSouth, true) | (Self::AlongEast, false) => {
                Some((from, to.min(CHANNEL_HALF)))
            }
            (Self::AlongEast, true) | (Self::AlongSouth, false) => {
                (plane <= CHANNEL_HALF).then_some((from, to))
            }
        };
        span.filter(|(from, to)| to > from)
    }

    /// The quarter of the same cell a channel would run on into, past the
    /// cell's centre line, given the quarter it is in.
    pub(super) fn behind(self, (sx, sz): (f32, f32)) -> Option<(f32, f32)> {
        match self {
            Self::AlongEast => Some((-sx, sz)),
            Self::AlongSouth => Some((sx, -sz)),
            _ => None,
        }
    }

    /// The pieces this shape is drawn from: along the shore, with the matching
    /// points the liquid and the floor stretch to from it.
    pub(super) fn rings(self) -> Rings {
        let curve = |point: &dyn Fn(f32) -> Point| -> Vec<Point> {
            (0..=CURVE_PIECES)
                .map(|i| point(i as f32 / CURVE_PIECES as f32 * std::f32::consts::FRAC_PI_2))
                .collect()
        };
        // From a point out along a line until it reaches the edge of the
        // quarter furthest along.
        let to_edge = |(east, south): Point, reach: f32| {
            let far = reach / east.abs().max(south.abs());
            (east * far, south * far)
        };
        match self {
            Self::Round => {
                let shore =
                    curve(&|angle| (CHANNEL_HALF * angle.cos(), CHANNEL_HALF * angle.sin()));
                Rings {
                    liquid: vec![(0.0, 0.0); shore.len()],
                    floor: shore.iter().map(|&p| to_edge(p, HALF_CELL)).collect(),
                    shore,
                }
            }
            Self::Notch => {
                let from_corner = |p: Point| (HALF_CELL - p.0, HALF_CELL - p.1);
                let shore = curve(&|angle| {
                    from_corner((CHANNEL_HALF * angle.cos(), CHANNEL_HALF * angle.sin()))
                });
                Rings {
                    liquid: shore
                        .iter()
                        .map(|&p| from_corner(to_edge(from_corner(p), HALF_CELL)))
                        .collect(),
                    floor: vec![(HALF_CELL, HALF_CELL); shore.len()],
                    shore,
                }
            }
            Self::AlongEast => Rings {
                shore: vec![(0.0, CHANNEL_HALF), (HALF_CELL, CHANNEL_HALF)],
                liquid: vec![(0.0, 0.0), (HALF_CELL, 0.0)],
                floor: vec![(0.0, HALF_CELL), (HALF_CELL, HALF_CELL)],
            },
            Self::AlongSouth => Rings {
                shore: vec![(CHANNEL_HALF, 0.0), (CHANNEL_HALF, HALF_CELL)],
                liquid: vec![(0.0, 0.0), (0.0, HALF_CELL)],
                floor: vec![(HALF_CELL, 0.0), (HALF_CELL, HALF_CELL)],
            },
            Self::Full => Rings {
                shore: Vec::new(),
                liquid: Vec::new(),
                floor: Vec::new(),
            },
        }
    }
}

/// Matching lists of points: the shore, where the liquid stretches to from it
/// and where the floor does.
pub(super) struct Rings {
    pub(super) shore: Vec<Point>,
    pub(super) liquid: Vec<Point>,
    pub(super) floor: Vec<Point>,
}
