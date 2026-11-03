//! Finding a way across one level: the quickest route of legal steps from one
//! point to another (A*), for the player, monsters and spells. See
//! `docs/context/shared/pathfinding.md`.
//!
//! Every move comes from `movement::step_on_level` and is priced by
//! `movement::step_cost`, so a path is one the server would accept and its
//! cost is the time it takes to walk.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::dungeon::Level;
use crate::movement::{
    DIAGONAL_COST, STRAIGHT_COST, Step, WorldConditions, WorldGround, step_cost, step_on_level,
    world_step_cost,
};

/// The order steps are tried in, so equally quick routes always come out the
/// same.
const DIRECTIONS: [(i64, i64); 8] = [
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, 0),
    (1, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
];

/// How long it takes to reach one goal from everywhere on a level, in
/// `movement::step_cost` units: for many mobs after one target, sharing the work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistanceMap {
    min: (i64, i64),
    width: i64,
    height: i64,
    costs: Vec<u32>,
}

impl DistanceMap {
    /// Flooded out from `goal` over legal steps.
    pub fn to(level: &Level, goal: (i64, i64)) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        let mut map = Self {
            min: (min_x, min_y),
            width,
            height,
            costs: vec![u32::MAX; (width * height) as usize],
        };
        let Some(start) = map.index(goal) else {
            return map;
        };
        map.costs[start] = 0;
        let mut queue = BinaryHeap::from([Reverse((0, goal))]);
        while let Some(Reverse((cost, cell))) = queue.pop() {
            if map.cost(cell).is_some_and(|known| known < cost) {
                continue;
            }
            // Whoever can step here, from one cell off or a jump away.
            for (dx, dy) in DIRECTIONS {
                for reach in [1, 2] {
                    let from = (cell.0 - dx * reach, cell.1 - dy * reach);
                    let step = Step::new(dx, dy).expect("a direction is a step");
                    if !level.is_walkable(from.0, from.1)
                        || step_on_level(level, from, step) != Some(cell)
                    {
                        continue;
                    }
                    let total = cost + step_cost(from, cell);
                    if let Some(index) = map.index(from)
                        && total < map.costs[index]
                    {
                        map.costs[index] = total;
                        queue.push(Reverse((total, from)));
                    }
                }
            }
        }
        map
    }

    fn index(&self, (x, y): (i64, i64)) -> Option<usize> {
        let (col, row) = (x - self.min.0, y - self.min.1);
        ((0..self.width).contains(&col) && (0..self.height).contains(&row))
            .then(|| (row * self.width + col) as usize)
    }

    /// How long it takes from here: `None` if the goal can't be reached.
    pub fn cost(&self, cell: (i64, i64)) -> Option<u32> {
        let cost = self.costs[self.index(cell)?];
        (cost != u32::MAX).then_some(cost)
    }

    /// The legal step onto a `free` cell that leads to where it costs the least, if
    /// any is nearer than here.
    pub fn towards(
        &self,
        level: &Level,
        from: (i64, i64),
        free: impl Fn((i64, i64)) -> bool,
    ) -> Option<Step> {
        let here = self.cost(from).unwrap_or(u32::MAX);
        self.best_step(level, from, free, |cost| (cost < here).then_some(cost))
    }

    /// The legal step onto a `free` cell that leads furthest from the goal, if any
    /// is further than here. Somewhere the goal can't be reached from is furthest.
    pub fn away(
        &self,
        level: &Level,
        from: (i64, i64),
        free: impl Fn((i64, i64)) -> bool,
    ) -> Option<Step> {
        let here = self.cost(from).unwrap_or(u32::MAX);
        self.best_step(level, from, free, |cost| {
            (cost > here).then_some(u32::MAX - cost)
        })
    }

    /// The step whose landing scores lowest. A step that scores `None` isn't taken.
    fn best_step(
        &self,
        level: &Level,
        from: (i64, i64),
        free: impl Fn((i64, i64)) -> bool,
        score: impl Fn(u32) -> Option<u32>,
    ) -> Option<Step> {
        DIRECTIONS
            .iter()
            .filter_map(|&(dx, dy)| {
                let step = Step::new(dx, dy)?;
                let to = step_on_level(level, from, step)?;
                if !free(to) {
                    return None;
                }
                Some((score(self.cost(to).unwrap_or(u32::MAX))?, step))
            })
            .min_by_key(|&(score, _)| score)
            .map(|(_, step)| step)
    }
}

/// A route across a level.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Path {
    /// The steps to take, in order.
    pub steps: Vec<Step>,
    /// Where each step lands (two cells on for a jump over liquid). The
    /// starting point isn't included.
    pub cells: Vec<(i64, i64)>,
    /// How long it takes to walk, in `movement::step_cost` units.
    pub cost: u32,
    /// Whether it ends on the goal. If not, it ends on the cell nearest the
    /// goal that could be reached (the start, with no steps, if that is it).
    pub reached: bool,
}

impl Path {
    /// The path through these cells, each a step on from the one before.
    fn through(from: (i64, i64), cells: Vec<(i64, i64)>, reached: bool) -> Self {
        let mut at = from;
        let mut steps = Vec::with_capacity(cells.len());
        let mut cost = 0;
        for &cell in &cells {
            let step = Step::new((cell.0 - at.0).signum(), (cell.1 - at.1).signum())
                .expect("cells of a path differ");
            steps.push(step);
            cost += step_cost(at, cell);
            at = cell;
        }
        Self {
            steps,
            cells,
            cost,
            reached,
        }
    }
}

/// The quickest route from `from` to `to`, using only legal steps that land on
/// cells `allowed` accepts (the client: cells the player knows; a monster: all
/// of them). `max_cost` stops a search going further than that, which also
/// keeps it cheap. If the goal can't be reached, the route goes to the nearest
/// cell that can, straight-line, so check `reached`.
///
/// Among equally quick routes the diagonals fall anywhere: see `straighten`.
pub fn find_path(
    level: &Level,
    from: (i64, i64),
    to: (i64, i64),
    max_cost: Option<u32>,
    allowed: impl Fn(i64, i64) -> bool,
) -> Path {
    let (min_x, min_y, width, height) = level.bounds();
    let index = |(x, y): (i64, i64)| ((y - min_y) * width + (x - min_x)) as usize;
    if !level.contains(from.0, from.1) {
        return Path::default();
    }

    // For each cell: the cheapest way found to it, and the cell it came from.
    let mut best = vec![u32::MAX; (width * height) as usize];
    let mut came_from = vec![from; best.len()];
    best[index(from)] = 0;

    // Cells to look at, cheapest total first. (`BinaryHeap` gives the largest
    // first, and `Reverse` turns that round.) The rest of the tuple settles
    // ties: closest to the goal, then by position.
    let mut open = BinaryHeap::new();
    open.push(Reverse((estimate(from, to), estimate(from, to), from)));
    let mut nearest = (distance_squared(from, to), from);

    while let Some(Reverse((total, left, at))) = open.pop() {
        let cost = total - left;
        // Reached again more cheaply since this was added.
        if cost > best[index(at)] {
            continue;
        }
        nearest = nearest.min((distance_squared(at, to), at));
        if at == to {
            break;
        }
        for (dx, dy) in DIRECTIONS {
            let step = Step::new(dx, dy).expect("directions are steps");
            let Some(next) = step_on_level(level, at, step) else {
                continue;
            };
            let next_cost = cost + step_cost(at, next);
            if next_cost >= best[index(next)]
                || max_cost.is_some_and(|max| next_cost > max)
                || !allowed(next.0, next.1)
            {
                continue;
            }
            best[index(next)] = next_cost;
            came_from[index(next)] = at;
            let left = estimate(next, to);
            open.push(Reverse((next_cost + left, left, next)));
        }
    }

    let end = nearest.1;
    let mut cells = Vec::new();
    let mut at = end;
    while at != from {
        cells.push(at);
        at = came_from[index(at)];
    }
    cells.reverse();
    Path::through(from, cells, end == to)
}

/// How far a route on the world map may go from where it starts, each way, in
/// points. More than the view shows, and the ground is infinite, so this is what
/// keeps the search's cost bounded.
pub const WORLD_ROUTE_REACH: i64 = 320;

/// The quickest route across the world map in these `conditions`: legal
/// steps, slopes costing extra. If the goal can't be reached within
/// `WORLD_ROUTE_REACH`, the route goes to the nearest point that can,
/// straight-line, so check `reached`.
pub fn find_world_path(
    ground: &WorldGround,
    conditions: WorldConditions,
    from: (i64, i64),
    to: (i64, i64),
) -> Path {
    /// Not looked at yet (a height the ground never has, water being `None`).
    const UNKNOWN: Option<i32> = Some(i32::MIN);

    let side = 2 * WORLD_ROUTE_REACH + 1;
    let (min_x, min_y) = (from.0 - WORLD_ROUTE_REACH, from.1 - WORLD_ROUTE_REACH);
    let inside = |(x, y): (i64, i64)| {
        (x - from.0).abs() <= WORLD_ROUTE_REACH && (y - from.1).abs() <= WORLD_ROUTE_REACH
    };
    let index = |(x, y): (i64, i64)| ((y - min_y) * side + (x - min_x)) as usize;

    // Sampling the ground is the slow part, so each point is only sampled once.
    let mut elevations = vec![UNKNOWN; (side * side) as usize];
    let mut elevation = |point: (i64, i64)| {
        let known = &mut elevations[index(point)];
        if *known == UNKNOWN {
            *known = ground.elevation(point.0, point.1, conditions);
        }
        *known
    };

    // As in `find_path`.
    let mut best = vec![u32::MAX; (side * side) as usize];
    let mut came_from = vec![from; best.len()];
    best[index(from)] = 0;
    let mut open = BinaryHeap::new();
    open.push(Reverse((estimate(from, to), estimate(from, to), from)));
    let mut nearest = (distance_squared(from, to), from);

    while let Some(Reverse((total, left, at))) = open.pop() {
        let cost = total - left;
        if cost > best[index(at)] {
            continue;
        }
        nearest = nearest.min((distance_squared(at, to), at));
        if at == to {
            break;
        }
        let here = elevation(at);
        for (dx, dy) in DIRECTIONS {
            let next = (at.0 + dx, at.1 + dy);
            if !inside(next) {
                continue;
            }
            // The rule of `step_on_world_map`, with the samples kept.
            let there = elevation(next);
            if here.is_some() && there.is_none() {
                continue;
            }
            let next_cost = cost + world_step_cost(at, next, (here, there));
            if next_cost >= best[index(next)] {
                continue;
            }
            best[index(next)] = next_cost;
            came_from[index(next)] = at;
            let left = estimate(next, to);
            open.push(Reverse((next_cost + left, left, next)));
        }
    }

    let end = nearest.1;
    let mut cells = Vec::new();
    let mut at = end;
    while at != from {
        cells.push(at);
        at = came_from[index(at)];
    }
    cells.reverse();
    let mut steps = Vec::with_capacity(cells.len());
    let mut at = from;
    for &cell in &cells {
        steps.push(Step::new(cell.0 - at.0, cell.1 - at.1).expect("cells of a path differ"));
        at = cell;
    }
    Path {
        steps,
        cells,
        cost: best[index(end)],
        reached: end == to,
    }
}

/// The same route, as quick and as legal, with its diagonals spread out along
/// each stretch instead of bunched together, which looks like walking in a
/// line. A separate step because only what is drawn cares: anything that
/// recalculates often can skip it. Jumps stay where they are, with each
/// stretch between them straightened alone.
pub fn straighten(
    level: &Level,
    from: (i64, i64),
    path: &Path,
    allowed: impl Fn(i64, i64) -> bool,
) -> Path {
    let mut points = vec![from];
    points.extend_from_slice(&path.cells);

    // The cost of the route up to each point, to compare lines against.
    let mut cost_to = vec![0];
    for pair in points.windows(2) {
        cost_to.push(cost_to[cost_to.len() - 1] + step_cost(pair[0], pair[1]));
    }
    let is_jump = |a: (i64, i64), b: (i64, i64)| (b.0 - a.0).abs() > 1 || (b.1 - a.1).abs() > 1;

    let mut cells = Vec::with_capacity(path.cells.len());
    let mut anchor = 0;
    while anchor + 1 < points.len() {
        // Where this stretch stops: the next jump, or the end.
        let stretch_end = (anchor..points.len() - 1)
            .find(|&i| is_jump(points[i], points[i + 1]))
            .unwrap_or(points.len() - 1);
        if stretch_end == anchor {
            cells.push(points[anchor + 1]);
            anchor += 1;
            continue;
        }
        // The furthest point along it that a line can replace the route to.
        let replaced = (anchor + 2..=stretch_end).rev().find_map(|end| {
            let line = line_cells(points[anchor], points[end]);
            let budget = cost_to[end] - cost_to[anchor];
            line_is_as_good(level, points[anchor], &line, budget, &allowed).then_some((end, line))
        });
        match replaced {
            Some((end, line)) => {
                cells.extend(line);
                anchor = end;
            }
            None => {
                cells.push(points[anchor + 1]);
                anchor += 1;
            }
        }
    }
    Path::through(from, cells, path.reached)
}

/// The fewest-steps line from one point to another, without the first: as many
/// diagonals as it takes, spread evenly among the straight steps.
fn line_cells(from: (i64, i64), to: (i64, i64)) -> Vec<(i64, i64)> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let long = dx.abs().max(dy.abs());
    let short = dx.abs().min(dy.abs());
    (1..=long)
        .map(|k| {
            // `k * short / long`, rounded to nearest, in whole numbers.
            let across = (2 * k * short + long) / (2 * long);
            if dx.abs() >= dy.abs() {
                (from.0 + k * dx.signum(), from.1 + across * dy.signum())
            } else {
                (from.0 + across * dx.signum(), from.1 + k * dy.signum())
            }
        })
        .collect()
}

/// Whether a line can be walked, one cell a step, and costs exactly `budget`.
fn line_is_as_good(
    level: &Level,
    from: (i64, i64),
    line: &[(i64, i64)],
    budget: u32,
    allowed: &impl Fn(i64, i64) -> bool,
) -> bool {
    let mut at = from;
    let mut cost = 0;
    for &cell in line {
        let Some(step) = Step::new(cell.0 - at.0, cell.1 - at.1) else {
            return false;
        };
        if step_on_level(level, at, step) != Some(cell) || !allowed(cell.0, cell.1) {
            return false;
        }
        cost += step_cost(at, cell);
        at = cell;
    }
    cost == budget
}

/// The least a route between two points can cost: what walking it would, with
/// nothing in the way. Never more than the real cost, which A* relies on to
/// find the quickest route. (If terrain faster than floor is ever added, this
/// must assume it.)
fn estimate(from: (i64, i64), to: (i64, i64)) -> u32 {
    let (dx, dy) = ((to.0 - from.0).abs(), (to.1 - from.1).abs());
    let (long, short) = (dx.max(dy) as u32, dx.min(dy) as u32);
    (long - short) * STRAIGHT_COST + short * DIAGONAL_COST
}

fn distance_squared(a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)
}

#[cfg(test)]
mod tests;
