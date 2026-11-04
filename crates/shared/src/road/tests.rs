use super::*;
use crate::poi::PoiKind;
use crate::rng::Seed;
use crate::surface::Surface;

// The client's fixed world, 26634 points wide.
fn world() -> WorldParams {
    WorldParams::from_seed(Seed::new(12345))
}

fn map() -> PoiMap {
    PoiMap::new(&world(), world().seed.derive("pois"))
}

/// An area of about 12 by 12 chunks, a little more than a player is sent.
const AREA: ((i64, i64), (i64, i64)) = ((0, -400), (800, 400));

#[test]
fn the_smaller_town_decides_the_kind() {
    use PoiKind::*;
    assert_eq!(RoadKind::between(Hamlet, City), Some(RoadKind::Path));
    assert_eq!(RoadKind::between(City, Village), Some(RoadKind::Track));
    assert_eq!(RoadKind::between(Town, City), Some(RoadKind::Road));
    assert_eq!(RoadKind::between(City, City), Some(RoadKind::Highway));
    assert_eq!(RoadKind::between(City, Castle), None);
    assert!(RoadKind::Highway > RoadKind::Path);
}

#[test]
fn an_area_has_some_roads_and_each_joins_two_towns_in_range() {
    let (pois, roads) = (map(), Roads::new(&world()));
    let links = roads.links_in_area(&pois, AREA.0, AREA.1);
    assert!(links.len() >= 3, "{links:?}");
    for (link, kind) in &links {
        let (a, b) = (
            pois.in_chunk(link.a).unwrap(),
            pois.in_chunk(link.b).unwrap(),
        );
        assert!(link.a < link.b);
        assert_eq!(RoadKind::between(a.kind, b.kind), Some(*kind));
        let east = (b.x - a.x).rem_euclid(pois.width());
        let east = east.min(pois.width() - east);
        assert!(east.pow(2) + (b.y - a.y).pow(2) <= RANGE * RANGE);
    }
}

#[test]
fn links_dont_depend_on_the_area_asked_about() {
    let (pois, roads) = (map(), Roads::new(&world()));
    let whole = roads.links_in_area(&pois, AREA.0, AREA.1);
    // Each half finds only links the whole does, and together all of them.
    let west = roads.links_in_area(&pois, AREA.0, (400, AREA.1.1));
    let east = roads.links_in_area(&pois, (401, AREA.0.1), AREA.1);
    for link in west.iter().chain(&east) {
        assert!(whole.contains(link), "{link:?}");
    }
    for link in &whole {
        assert!(west.contains(link) || east.contains(link), "{link:?}");
    }
    // The same links a planet's width away.
    let width = pois.width();
    let shifted = roads.links_in_area(
        &pois,
        (AREA.0.0 + width, AREA.0.1),
        (AREA.1.0 + width, AREA.1.1),
    );
    assert_eq!(shifted, whole);
}

#[test]
fn roads_are_steps_over_dry_land_from_town_to_town() {
    let world = world();
    let (pois, roads, surface) = (map(), Roads::new(&world), Surface::new(&world));
    let mut made = 0;
    for (link, kind) in roads.links_in_area(&pois, AREA.0, AREA.1) {
        let Some(road) = roads.road(&pois, link) else {
            continue;
        };
        made += 1;
        assert_eq!(road.kind, kind);
        let (a, b) = (
            pois.in_chunk(link.a).unwrap(),
            pois.in_chunk(link.b).unwrap(),
        );
        assert_eq!(road.points[0], (a.x, a.y));
        let last = *road.points.last().unwrap();
        assert_eq!((last.0.rem_euclid(pois.width()), last.1), (b.x, b.y));
        for pair in road.points.windows(2) {
            let (dx, dy) = (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1);
            assert!(dx.abs() <= 1 && dy.abs() <= 1 && (dx, dy) != (0, 0));
        }
        for &(x, y) in &road.points {
            assert!(surface.sample(x, y).height > world.high_water(), "{x}, {y}");
        }
    }
    assert!(made >= 2, "{made}");
}

// Golden values: if these change, every world's roads move, and
// `ROAD_VERSION` must go up.
#[test]
fn golden_roads() {
    let (pois, roads) = (map(), Roads::new(&world()));
    let found: Vec<_> = roads
        .links_in_area(&pois, AREA.0, AREA.1)
        .into_iter()
        .take(3)
        .map(|(link, kind)| {
            let points = roads.road(&pois, link).map(|road| road.points);
            let summary = points.map(|points| (points.len(), points[points.len() / 2]));
            (link.a.x, link.a.y, link.b.x, link.b.y, kind, summary)
        })
        .collect();
    assert_eq!(format!("{found:?}"), GOLDEN_ROADS);
}

const GOLDEN_ROADS: &str = "[(1, -4, 1, -2, Path, Some((165, (88, -165)))), (1, -4, 2, -4, Path, Some((96, (121, -250)))), (1, -2, 414, 0, Track, Some((217, (15, -22))))]";

#[test]
fn a_step_counts_along_a_road_either_way_and_round_the_planet_but_not_across_it() {
    let link = RoadLink::new(ChunkPos { x: 0, y: 0 }, ChunkPos { x: 1, y: 0 });
    let road = |kind, points: Vec<(i64, i64)>| Road { link, kind, points };
    let mut steps = RoadSteps::new(100);
    steps.add(&road(
        RoadKind::Path,
        vec![(98, 5), (99, 5), (100, 5), (101, 6)],
    ));
    assert_eq!(steps.kind_of((98, 5), (99, 5)), Some(RoadKind::Path));
    assert_eq!(steps.kind_of((99, 5), (98, 5)), Some(RoadKind::Path));
    // Past the planet's edge, on any copy of it.
    assert_eq!(steps.kind_of((99, 5), (0, 5)), Some(RoadKind::Path));
    assert_eq!(steps.kind_of((-1, 5), (0, 5)), Some(RoadKind::Path));
    // Two of its points that aren't one after the other: crossing, not along.
    assert_eq!(steps.kind_of((99, 5), (1, 6)), None);
    assert_eq!(steps.kind_of((98, 5), (98, 6)), None);
    // Where a larger road shares a step, it counts.
    steps.add(&road(RoadKind::Highway, vec![(99, 5), (100, 5)]));
    assert_eq!(steps.kind_of((99, 5), (0, 5)), Some(RoadKind::Highway));
    steps.add(&road(RoadKind::Track, vec![(99, 5), (100, 5)]));
    assert_eq!(steps.kind_of((99, 5), (0, 5)), Some(RoadKind::Highway));
}

#[test]
fn larger_roads_are_faster_up_to_twice_open_land() {
    let speeds = RoadKind::ALL.map(RoadKind::speed_percent);
    assert_eq!(speeds, [125, 150, 175, FASTEST_PERCENT]);
    assert_eq!(FASTEST_PERCENT, 200);
}
