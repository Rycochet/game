//! A road's way round a cutout: straight if nothing is in the way, else by
//! its corners, the shortest way round.

use super::Point;

/// How far outside the cutout's corners the way round goes, in cells.
const ROUND: f64 = 3.0;

/// The points of the shortest way from `from` to `to` that keeps out of
/// `rect` (its `(min, max)` corners), `from` and `to` included: straight
/// there, or round by the corners of `rect` grown by `ROUND`.
pub(super) fn round(from: Point, to: Point, rect: (Point, Point)) -> Vec<Point> {
    let ((x0, y0), (x1, y1)) = rect;
    // A way may run along the grown rectangle's edges, so only what comes
    // inside it by half a cell or more is in the way.
    let keep_out = (
        (x0 - ROUND + 0.5, y0 - ROUND + 0.5),
        (x1 + ROUND - 0.5, y1 + ROUND - 0.5),
    );
    let nodes = [
        from,
        to,
        (x0 - ROUND, y0 - ROUND),
        (x1 + ROUND, y0 - ROUND),
        (x1 + ROUND, y1 + ROUND),
        (x0 - ROUND, y1 + ROUND),
    ];
    let length =
        |a: Point, b: Point| ((a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)).sqrt();
    let open = |a: usize, b: usize| !crosses(nodes[a], nodes[b], keep_out);
    // Dijkstra over six points, from `from`.
    let mut best = [f64::INFINITY; 6];
    let mut came = [usize::MAX; 6];
    let mut done = [false; 6];
    best[0] = 0.0;
    loop {
        let mut next = None;
        for node in 0..6 {
            if !done[node]
                && best[node].is_finite()
                && next.is_none_or(|n: usize| best[node] < best[n])
            {
                next = Some(node);
            }
        }
        let Some(node) = next else { break };
        done[node] = true;
        if node == 1 {
            break;
        }
        for other in 0..6 {
            if done[other] || !open(node, other) {
                continue;
            }
            let through = best[node] + length(nodes[node], nodes[other]);
            if through < best[other] {
                best[other] = through;
                came[other] = node;
            }
        }
    }
    if came[1] == usize::MAX {
        // Inside the rectangle itself: nothing better than straight.
        return vec![from, to];
    }
    let mut points = vec![to];
    let mut node = 1;
    while came[node] != usize::MAX {
        node = came[node];
        points.push(nodes[node]);
    }
    points.reverse();
    points
}

/// Whether the stretch from `a` to `b` passes through the inside of a
/// rectangle (its `(min, max)` corners), not only touching its edge.
pub(super) fn crosses(a: Point, b: Point, (min, max): (Point, Point)) -> bool {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let (mut enter, mut leave) = (0.0_f64, 1.0_f64);
    for (towards, room) in [
        (-dx, a.0 - min.0),
        (dx, max.0 - a.0),
        (-dy, a.1 - min.1),
        (dy, max.1 - a.1),
    ] {
        if towards == 0.0 {
            if room <= 0.0 {
                return false;
            }
        } else {
            let at = room / towards;
            if towards < 0.0 {
                enter = enter.max(at);
            } else {
                leave = leave.min(at);
            }
        }
    }
    enter < leave
}
