use super::route::{crosses, round};
use super::way_in::WayIn;
use super::*;
use crate::dungeon::{Cell, Level};
use crate::poi::{ChunkPos, PoiMap};
use crate::rng::Seed;
use crate::road::{Road, RoadKind, RoadLink};
use crate::site::{Generator, Site};
use crate::wilderness::{BlockPos, ChunkPlan, Wilderness};
use crate::world::WorldParams;

#[test]
fn ways_in_are_the_runs_of_road_on_each_edge() {
    let mut level = Level::new(10, 20, 8, 6);
    for x in 10..18 {
        for y in 20..26 {
            level.set(x, y, Cell::Grass);
        }
    }
    // Two wide on the north edge, one on the east.
    level.set(13, 20, Cell::Cobblestone);
    level.set(14, 20, Cell::Cobblestone);
    level.set(17, 23, Cell::DirtTrack);
    let ways = ways_in(&level);
    assert_eq!(
        ways,
        vec![
            WayIn {
                at: (14.0, 20.0),
                out: (0.0, -1.0)
            },
            WayIn {
                at: (18.0, 23.5),
                out: (1.0, 0.0)
            },
        ]
    );
}

#[test]
fn the_way_round_keeps_out_of_the_cutout() {
    let rect = ((0.0, 0.0), (20.0, 10.0));
    // Clear: straight there.
    assert_eq!(round((-10.0, -10.0), (30.0, -10.0), rect).len(), 2);
    // From the west to the east side: round by two corners.
    let points = round((-10.0, 5.0), (30.0, 6.0), rect);
    assert_eq!(points.len(), 4);
    for pair in points.windows(2) {
        assert!(!crosses(pair[0], pair[1], rect), "{points:?}");
    }
    assert!(crosses((-10.0, 5.0), (30.0, 6.0), rect));
    assert!(!crosses((0.0, -1.0), (20.0, -1.0), rect));
}

/// The test world's first walled town, its chunk's plan with a straight
/// road from `points` west of it to its poi, and the road.
fn walled_town_with_road(points: i64) -> (Wilderness, ChunkPlan, Road) {
    let params = WorldParams::from_seed(Seed::new(12345));
    let wild = Wilderness::new(&params);
    let pois = PoiMap::new(&params, Seed::new(999));
    for chunk in (0..60).flat_map(|x| (-30..30).map(move |y| ChunkPos { x, y })) {
        let Some(poi) = pois.in_chunk(chunk).filter(|poi| poi.kind.is_town()) else {
            continue;
        };
        let sides = crate::road::town_sides(&pois, &poi);
        let generator = Generator::for_site(poi.kind, poi.seed, poi.desert, sides);
        let site = Site::with_generator(poi.kind, poi.seed, generator.version(), generator)
            .expect("the current version runs");
        if site.level(0).walls.is_empty() {
            continue;
        }
        let road = Road {
            link: RoadLink {
                a: ChunkPos {
                    x: chunk.x - 1,
                    ..chunk
                },
                b: chunk,
            },
            kind: RoadKind::Road,
            points: (poi.x - points..=poi.x).map(|x| (x, poi.y)).collect(),
        };
        let plan = ChunkPlan::new(&wild, chunk, Some((poi, &site)), &[], &[&road]);
        return (wild, plan, road);
    }
    panic!("a walled town");
}

#[test]
fn a_road_reaches_a_way_into_the_town() {
    let (wild, plan, _) = walled_town_with_road(12);
    let cutout = plan.cutout().expect("the town");
    let ways = ways_in(cutout.level());
    assert!(!ways.is_empty());
    let outside = |way: &WayIn| {
        let x = (way.at.0 + way.out.0 * 0.5).floor() as i64;
        let y = (way.at.1 + way.out.1 * 0.5).floor() as i64;
        let BlockPos { x: bx, y: by } = wild.block_of(x, y);
        wild.block(BlockPos { x: bx, y: by }, &plan).cell(x, y)
    };
    let reached = ways
        .iter()
        .filter(|way| outside(way) == Cell::Cobblestone)
        .count();
    assert!(reached >= 1, "no road outside any of {ways:?}");
}

#[test]
fn the_bend_stays_well_inside_the_towns_chunk() {
    for points in [3, 6, 12] {
        let (wild, plan, road) = walled_town_with_road(points);
        let cutout = plan.cutout().expect("the town");
        let bounds = wild.chunk_bounds(plan.chunk());
        let straight = wild.road_segments(&road, 0);
        let bent = meet_town(straight.clone(), false, cutout, bounds);
        assert_ne!(bent, straight);
        let (x, y, width, height) = bounds;
        let inside = |(px, py): Point| {
            px >= x as f64 + KEEP_CLEAR
                && px <= (x + width) as f64 - KEEP_CLEAR
                && py >= y as f64 + KEEP_CLEAR
                && py <= (y + height) as f64 - KEEP_CLEAR
        };
        for segment in bent.iter().filter(|segment| !straight.contains(segment)) {
            let kept_from = straight.iter().any(|s| s.from == segment.from);
            assert!(kept_from || inside(segment.from), "{segment:?}");
            assert!(inside(segment.to), "{segment:?}");
        }
        // The same the other way round.
        let reversed_road = reversed(straight.clone());
        let back = meet_town(reversed_road, true, cutout, bounds);
        assert_eq!(reversed(back), bent);
    }
}
