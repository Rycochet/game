use super::*;
use crate::rng::Seed;

fn surface() -> Surface {
    let desert = SurfaceGenerator::Desert;
    Surface::new(&WorldParams::with_generator(Seed::new(12345), desert))
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

const GOLDEN_SAMPLES: &str = "[SurfaceSample { height: 51.50281874090433, jitter: -7.710816383361816 }, SurfaceSample { height: 51.91214060783386, jitter: -7.027172088623047 }, SurfaceSample { height: 55.553331926465034, jitter: -3.9334805011749268 }, SurfaceSample { height: 89.50275233387947, jitter: -2.190032720565796 }, SurfaceSample { height: 54.027571484446526, jitter: 1.7110778093338013 }]";

#[test]
fn wraps_east_west() {
    let surface = surface();
    let width = surface.width();
    assert_eq!(surface.sample(3, 7), surface.sample(3 + width, 7));
    assert_eq!(surface.sample(-1, 7), surface.sample(width - 1, 7));
}
