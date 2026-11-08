//! World map routes and roads.

use std::time::Instant;

use super::*;
use crate::movement::{Footing, Gait, WorldConditions, WorldGround, world_step_cost};
use crate::poi::PoiMap;
use crate::river::RiverPoints;
use crate::rng::Seed;
use crate::road::{RoadSteps, Roads};
use crate::world::WorldParams;

/// The longest road through an area of the test world that a route can
/// follow (its ends within `WORLD_ROUTE_REACH`, and not across the planet's
/// edge), the roads there, and the ground.
fn longest_road() -> (crate::road::Road, RoadSteps, WorldGround, WorldConditions) {
    let world = WorldParams::from_seed(Seed::new(12345));
    let pois = PoiMap::new(&world, world.seed.derive("pois"));
    let roads = Roads::new(&world);
    let mut steps = RoadSteps::new(pois.width());
    let mut longest: Option<crate::road::Road> = None;
    for (link, _) in roads.links_in_area(&pois, (0, -400), (800, 400)) {
        let Some(road) = roads.road(&pois, link) else {
            continue;
        };
        steps.add(&road);
        let (first, last) = (road.points[0], *road.points.last().unwrap());
        let apart = (last.0 - first.0).abs().max((last.1 - first.1).abs());
        let in_reach = apart < WORLD_ROUTE_REACH && (0..pois.width()).contains(&last.0);
        if in_reach
            && longest
                .as_ref()
                .is_none_or(|l| road.points.len() > l.points.len())
        {
            longest = Some(road);
        }
    }
    let conditions = WorldConditions {
        water: world.high_water(),
        ice: true,
    };
    (
        longest.expect("roads"),
        steps,
        WorldGround::new(&world),
        conditions,
    )
}

/// What a route costs to walk, by the rules the server charges by.
fn walked(
    ground: &WorldGround,
    conditions: WorldConditions,
    roads: &RoadSteps,
    from: (i64, i64),
    path: &Path,
) -> u32 {
    let mut at = from;
    let mut cost = 0;
    for &cell in &path.cells {
        let ends = (
            ground.elevation(at.0, at.1, conditions),
            ground.elevation(cell.0, cell.1, conditions),
        );
        cost += world_step_cost(
            at,
            cell,
            ends,
            {
                let road = roads.kind_of(at, cell);
                let terrain = road.is_none().then(|| ground.land_terrain(cell.0, cell.1));
                Footing {
                    road,
                    wet: false,
                    terrain,
                }
            },
            Gait::UNSKILLED,
        );
        at = cell;
    }
    cost
}

#[test]
fn routes_take_roads_where_they_are_quicker_and_say_what_they_cost() {
    let (road, steps, ground, conditions) = longest_road();
    let (from, to) = (road.points[0], *road.points.last().unwrap());

    let start = Instant::now();
    let without = find_world_path(
        &ground,
        conditions,
        &RoadSteps::default(),
        &RiverPoints::default(),
        from,
        to,
        Gait::UNSKILLED,
    );
    let plain_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    let with = find_world_path(
        &ground,
        conditions,
        &steps,
        &RiverPoints::default(),
        from,
        to,
        Gait::UNSKILLED,
    );
    let roads_ms = start.elapsed().as_secs_f64() * 1000.0;
    eprintln!(
        "{} points of {:?}: {plain_ms:.0} ms without roads, {roads_ms:.0} ms with",
        road.points.len(),
        road.kind
    );

    assert!(with.reached && without.reached);
    assert_eq!(with.cost, walked(&ground, conditions, &steps, from, &with));
    // Quicker with the road, and mostly along it.
    assert!(with.cost < walked(&ground, conditions, &steps, from, &without) + 1);
    assert!(with.cost < without.cost);
    let mut at = from;
    let mut along = 0;
    for &cell in &with.cells {
        along += usize::from(steps.kind_of(at, cell).is_some());
        at = cell;
    }
    assert!(
        along * 2 > with.cells.len(),
        "{along} of {} steps",
        with.cells.len()
    );
}
