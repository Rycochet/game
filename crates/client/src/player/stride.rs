//! Walking freely on a level: the stretch walked in a straight line since the
//! last move was sent, cut into straight moves the server can check by the
//! one rule (`shared::clearance::check_segment`), sliding along what is in the
//! way.

use bevy::math::DVec2;
use shared::clearance::{Other, check_segment, crowd_per_mille, slide};
use shared::creature::Size;
use shared::dungeon::Level;
use shared::spot::{CELL_SPAN, Spot};

/// How finely a walk is cut, in units: small enough that a piece never skips
/// a cell or a corner.
const PIECE: f64 = 40.0;
/// The longest straight move sent, in units: a move also ends on entering
/// another cell, so each crosses at most one cell's edge.
const LONGEST: f64 = CELL_SPAN as f64;
/// How close two ways must be to count as the same (the cosine between them):
/// a camera turning a hair doesn't end a move.
const SAME_WAY: f64 = 0.9999;

/// Who walks: their size, and who else is about (enemies block; anyone else
/// but another player is squeezed past, `shared::clearance::crowd`).
pub(super) struct Walker<'a> {
    pub level: &'a Level,
    pub size: Size,
    pub others: &'a [Other],
    /// Walking freely, nobody squeezes any more than where they started (as
    /// `slide`), so pushing against a wall doesn't press into it; a
    /// journey's route squeezes through where it was found to.
    pub squeezes: bool,
}

impl Walker<'_> {
    /// Whether the straight walk from `a` to `b` is allowed: by the one rule,
    /// without jumping, and not into an enemy.
    pub(super) fn walks(&self, a: Spot, b: Spot) -> bool {
        let Some(passage) = check_segment(self.level, self.size, a, b) else {
            return false;
        };
        let squeeze =
            |spot| check_segment(self.level, self.size, spot, spot).map(|p| p.squeeze_per_mille);
        let squeezed = !self.squeezes
            && passage.squeeze_per_mille > 1000
            && squeeze(a).is_none_or(|at_a| passage.squeeze_per_mille > at_a);
        let free = crowd_per_mille(self.size, a, b, self.others).is_some();
        passage.jumped.is_none() && !squeezed && free
    }
}

/// The straight stretch walked since the last move was sent.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Stride {
    /// Where the stretch started: where the server will have the player once
    /// every move sent so far is taken.
    pub start: Spot,
    /// Where the player is now.
    pub at: Spot,
    /// The way it goes, as a unit vector in units (zero: not yet going).
    way: DVec2,
    /// How far along it the player is, in units.
    gone: f64,
}

/// How a walk went.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Walked {
    /// How far it went, in units.
    pub distance: f64,
    /// It stopped short: something is in the way.
    pub stuck: bool,
}

impl Stride {
    /// Standing at `at`.
    pub(super) fn at(at: Spot) -> Self {
        Self {
            start: at,
            at,
            way: DVec2::ZERO,
            gone: 0.0,
        }
    }

    /// Ends the stretch here: the move from its start, if it went anywhere.
    pub(super) fn finish(&mut self) -> Option<(Spot, Spot)> {
        let walked = (self.start != self.at).then_some((self.start, self.at));
        *self = Self::at(self.at);
        walked
    }

    /// The spot `gone` units along the stretch.
    fn along(&self, gone: f64) -> Spot {
        let offset = self.way * gone;
        self.start
            .offset(offset.x.round() as i64, offset.y.round() as i64)
    }

    /// Starts a new stretch from here going `way`, sending the old one.
    fn turn(&mut self, way: DVec2, moves: &mut Vec<(Spot, Spot)>) {
        moves.extend(self.finish());
        self.way = way;
    }

    /// Walks up to `distance` units `want` (a unit vector), sliding along
    /// whatever is in the way, adding each straight move finished to `moves`.
    /// `exactly`: stop on this spot, and don't slide (a journey's next
    /// point).
    pub(super) fn walk(
        &mut self,
        walker: &Walker,
        want: DVec2,
        distance: f64,
        exactly: Option<Spot>,
        moves: &mut Vec<(Spot, Spot)>,
    ) -> Walked {
        let mut left = distance;
        while left > 0.0 {
            if let Some(goal) = exactly
                && self.at == goal
            {
                break;
            }
            let piece = left.min(PIECE);
            // Back to the way wanted as soon as it is open again.
            if self.way.dot(want) < SAME_WAY {
                let tried = offset(self.at, want * piece);
                if self.way == DVec2::ZERO || walker.walks(self.at, tried) {
                    self.turn(want, moves);
                }
            }
            let near_goal = exactly.filter(|goal| unit_distance(self.at, *goal) <= piece);
            // Counted exactly, so a slow walk's fractions add up.
            let (next, gone, step) = match near_goal {
                Some(goal) => (
                    goal,
                    unit_distance(self.start, goal),
                    unit_distance(self.at, goal),
                ),
                None => (self.along(self.gone + piece), self.gone + piece, piece),
            };
            if walker.walks(self.start, next) {
                left -= step;
                self.gone = gone;
                self.at = next;
                if self.at.cell() != self.start.cell() || self.gone >= LONGEST {
                    moves.extend(self.finish());
                    self.way = want;
                }
                continue;
            }
            // In the way: send what was walked, and slide from here.
            moves.extend(self.finish());
            if exactly.is_some() {
                return Walked {
                    distance: distance - left,
                    stuck: true,
                };
            }
            let Some(slid) = slid(walker, self.at, want * piece) else {
                return Walked {
                    distance: distance - left,
                    stuck: true,
                };
            };
            let gone = unit_distance(self.at, slid);
            self.way = to_vector(self.at, slid) / gone;
            self.gone = gone;
            self.at = slid;
            left -= gone;
            if self.at.cell() != self.start.cell() {
                moves.extend(self.finish());
            }
        }
        Walked {
            distance: distance - left,
            stuck: false,
        }
    }
}

/// Where sliding from `at` on a walk of `by` (one piece) gets to, if
/// anywhere: by `slide`, or else closer by halves, or along whichever of
/// east-west or north-south goes further (as much as the walk goes that
/// way), as nothing may squeeze more than where it starts.
fn slid(walker: &Walker, at: Spot, by: DVec2) -> Option<Spot> {
    let slid = slide(walker.level, walker.size, at, offset(at, by));
    let (along_x, along_y) = (DVec2::new(by.x, 0.0), DVec2::new(0.0, by.y));
    let axes = match along_x.length() >= along_y.length() {
        true => [along_x, along_y],
        false => [along_y, along_x],
    };
    let others = [by * 0.5, by * 0.25].into_iter().chain(axes);
    std::iter::once(slid)
        .chain(others.map(|by| offset(at, by)))
        .find(|&to| unit_distance(at, to) >= 1.0 && walker.walks(at, to))
}

/// `from` moved by `by` units, rounded.
fn offset(from: Spot, by: DVec2) -> Spot {
    from.offset(by.x.round() as i64, by.y.round() as i64)
}

/// The vector from `a` to `b`, in units.
pub(super) fn to_vector(a: Spot, b: Spot) -> DVec2 {
    DVec2::new((b.x - a.x) as f64, (b.y - a.y) as f64)
}

/// How far apart two spots are, in units.
pub(super) fn unit_distance(a: Spot, b: Spot) -> f64 {
    to_vector(a, b).length()
}

#[cfg(test)]
mod tests;
