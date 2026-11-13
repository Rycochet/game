//! Roads arriving at a town: a world road's last stretch, from where it comes
//! near the town's cutout, bent to the nearest way in on the cutout's edge (a
//! town road reaching the edge: through a gate, if the town is walled), so
//! the road meets it rather than stopping wherever it runs into the cutout.
//! Roads bent to the same way in meet `MEET` cells outside it.
//!
//! Only the chunk holding the town knows its cutout, so the bend is kept to
//! a zone round the cutout at least `KEEP_CLEAR` inside that chunk: no other
//! chunk's cells are near enough to be changed by it, and every chunk agrees.

mod route;
mod way_in;
mod zone;

use way_in::ways_in;
use zone::Zone;

use super::course::Segment;
use super::{Cutout, ROAD_CLEARED_TO};

/// How far round the cutout the bend may start, in cells.
const APPROACH: f64 = 16.0;

/// How far inside its chunk the zone the bend stays in is kept, in cells:
/// past the widest road (2.1 half across) and the trees thinning beside it.
pub(super) const KEEP_CLEAR: f64 = 10.0;

const _: () = assert!(KEEP_CLEAR > ROAD_CLEARED_TO + 2.1 + 1.0);

/// How far straight out from a way in roads going to it meet, in cells.
pub(super) const MEET: f64 = 8.0;

pub(super) type Point = (f64, f64);

/// A road's segments, from its town `a` to its town `b`, with the end at the
/// town whose `cutout` this is (`a` if `at_start`) bent to the nearest way in.
/// `chunk`: the town's chunk's bounds. Left as they are if the town has no
/// way in, or if this copy of the road doesn't end in the chunk.
pub(super) fn meet_town(
    segments: Vec<Segment>,
    at_start: bool,
    cutout: &Cutout,
    chunk: (i64, i64, i64, i64),
) -> Vec<Segment> {
    let ways = ways_in(cutout.level());
    let road = if at_start {
        reversed(segments.clone())
    } else {
        segments.clone()
    };
    let (Some(last), false) = (road.last().copied(), ways.is_empty()) else {
        return segments;
    };
    let (cx, cy, cw, ch) = chunk;
    let (cx, cy, cw, ch) = (cx as f64, cy as f64, cw as f64, ch as f64);
    let ends_here = last.to.0 >= cx - 8.0
        && last.to.0 <= cx + cw + 8.0
        && last.to.1 >= cy - 8.0
        && last.to.1 <= cy + ch + 8.0;
    if !ends_here {
        return segments;
    }
    let (lx, ly, lw, lh) = cutout.level().bounds();
    let rect = ((lx as f64, ly as f64), ((lx + lw) as f64, (ly + lh) as f64));
    let zone = Zone {
        min: (
            (rect.0.0 - APPROACH).max(cx + KEEP_CLEAR),
            (rect.0.1 - APPROACH).max(cy + KEEP_CLEAR),
        ),
        max: (
            (rect.1.0 + APPROACH).min(cx + cw - KEEP_CLEAR),
            (rect.1.1 + APPROACH).min(cy + ch - KEEP_CLEAR),
        ),
    };
    let half = last.half.1;
    // Kept as it is up to where it first comes into the zone.
    let mut bent = Vec::new();
    let start = match road.iter().position(|segment| zone.contains(segment.to)) {
        Some(index) => {
            let segment = road[index];
            bent.extend_from_slice(&road[..index]);
            if zone.contains(segment.from) {
                segment.from
            } else {
                let entered = zone.entered(segment.from, segment.to);
                bent.push(Segment {
                    to: entered,
                    ..segment
                });
                entered
            }
        }
        None => {
            bent.extend_from_slice(&road);
            last.to
        }
    };
    let distance = |a: Point, b: Point| (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1);
    let mut way = ways[0];
    for &other in &ways[1..] {
        if distance(start, other.meet()) < distance(start, way.meet()) {
            way = other;
        }
    }
    let mut points = route::round(start, way.meet(), rect);
    // A cell into the cutout, which covers it, so the road reaches the edge.
    points.push((way.at.0 - way.out.0, way.at.1 - way.out.1));
    for pair in points.windows(2) {
        bent.push(Segment {
            from: pair[0],
            to: pair[1],
            half: (half, half),
        });
    }
    if at_start { reversed(bent) } else { bent }
}

/// The same stretches the other way round.
fn reversed(segments: Vec<Segment>) -> Vec<Segment> {
    let back = |segment: Segment| Segment {
        from: segment.to,
        to: segment.from,
        half: (segment.half.1, segment.half.0),
    };
    segments.into_iter().rev().map(back).collect()
}

#[cfg(test)]
mod tests;
