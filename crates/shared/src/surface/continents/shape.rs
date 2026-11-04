//! How far a point is into a continent becomes metres: shallow shelves round
//! the coasts, a slope down to the deep ocean floor, low land near the sea
//! rising slowly inland, and mountains only where a range is. Most of the land
//! is then low, as on Earth, with a few high ranges.

/// The continental shelf: how far out it goes (on the continents' scale) and
/// how deep it is at its edge.
const SHELF_WIDTH: f64 = 0.03;
const SHELF_DEPTH: f64 = -200.0;
/// The slope down from the shelf to the ocean floor.
const SLOPE_WIDTH: f64 = 0.06;
const OCEAN_FLOOR: f64 = -4000.0;
/// Past the slope the floor keeps on deepening slowly, as far as this.
const DEEPENING: f64 = 6000.0;
const DEEPEST: f64 = -8000.0;
/// Land away from mountains: how high it gets far inland where it is raised
/// most, and how slowly it rises from the coast.
const LOWLAND: f64 = 1500.0;
const LOWLAND_SPREAD: f64 = 0.3;
/// Mountains at full strength, and how far inland from the coast they reach
/// it.
const MOUNTAINS: f64 = 9000.0;
const COAST_MARGIN: f64 = 0.05;

/// Metres from the sea at `land` into a continent (below 0: out to sea), with
/// `ridge` (`0..=1`) the strength of any mountain range there and `uplift`
/// (`0..=1`) how raised the land around is: plains low, plateaus high.
pub(super) fn metres(land: f64, ridge: f64, uplift: f64) -> f64 {
    if land >= 0.0 {
        let rise = 1.0 - libm::exp(-land / LOWLAND_SPREAD);
        let lowland = LOWLAND * rise * uplift;
        let mountains = MOUNTAINS * ridge * ridge * smoothstep(0.0, COAST_MARGIN, land);
        return lowland + mountains;
    }
    let out = -land;
    if out <= SHELF_WIDTH {
        SHELF_DEPTH * out / SHELF_WIDTH
    } else if out <= SHELF_WIDTH + SLOPE_WIDTH {
        let down = smoothstep(0.0, 1.0, (out - SHELF_WIDTH) / SLOPE_WIDTH);
        SHELF_DEPTH + (OCEAN_FLOOR - SHELF_DEPTH) * down
    } else {
        (OCEAN_FLOOR - (out - SHELF_WIDTH - SLOPE_WIDTH) * DEEPENING).max(DEEPEST)
    }
}

pub(super) fn smoothstep(from: f64, to: f64, value: f64) -> f64 {
    let t = ((value - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
