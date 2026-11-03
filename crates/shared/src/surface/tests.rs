use super::*;
use crate::rng::Seed;

fn surface() -> Surface {
    Surface::new(&WorldParams::from_seed(Seed::new(12345)))
}

// Golden values: if these change, every generated world changes.
#[test]
fn golden_samples() {
    let surface = surface();
    let samples: Vec<_> = [(0, 0), (1, 0), (-1, 5), (1234, -5678), (1_000_000, 42)]
        .iter()
        .map(|&(x, y)| surface.sample(x, y))
        .collect();
    assert_eq!(format!("{samples:?}"), GOLDEN_SAMPLES);
}

const GOLDEN_SAMPLES: &str = "[SurfaceSample { height: 66.09790093125775, jitter: -6.349335193634033 }, SurfaceSample { height: 66.56663356721401, jitter: -5.502878665924072 }, SurfaceSample { height: 69.92800272256136, jitter: -0.8336078524589539 }, SurfaceSample { height: 47.342495096847415, jitter: -0.28205934166908264 }, SurfaceSample { height: 46.198867592960596, jitter: -7.5050225257873535 }]";

#[test]
fn wraps_east_west() {
    let surface = surface();
    let width = surface.width();
    assert_eq!(surface.sample(3, 7), surface.sample(3 + width, 7));
    assert_eq!(surface.sample(-1, 7), surface.sample(width - 1, 7));
}
