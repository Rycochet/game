use std::collections::BTreeMap;

use super::*;

fn town(target: i64, seed: u64) -> SiteLevel {
    Town::new(Seed::new(seed), target).generate()
}

/// Towns of every size, from a few seeds.
fn towns() -> impl Iterator<Item = (i64, SiteLevel)> {
    [3, 5, 8, 12, 20, 30, 50, 80, 120, 150]
        .into_iter()
        .flat_map(|target| (0..12).map(move |seed| (target, town(target, seed * 31 + 7))))
}

/// The cells of a building.
fn cells_of(look: &BuildingLook) -> Vec<(i64, i64)> {
    look.rects
        .iter()
        .flat_map(|r| {
            (r.y..r.y + r.height).flat_map(move |y| (r.x..r.x + r.width).map(move |x| (x, y)))
        })
        .collect()
}

#[test]
fn the_same_seed_makes_the_same_town() {
    assert_eq!(town(40, 5), town(40, 5));
    assert_ne!(town(40, 5), town(40, 6));
}

#[test]
fn towns_have_about_the_buildings_asked_for() {
    for (target, site) in towns() {
        let count = site.buildings.len() as i64;
        assert!(count >= 3, "{target}: only {count}");
        assert!(
            count <= target && count * 10 >= target * 6,
            "{target}: {count}"
        );
    }
}

/// No building has a single cell sticking out, and each holds a 3×3.
#[test]
fn buildings_are_solid_shapes() {
    for (_, site) in towns() {
        for building in &site.buildings {
            let cells = cells_of(&building.look);
            let has = |x, y| cells.contains(&(x, y));
            let square = |x, y, side: i64| (0..side).all(|d| (0..side).all(|a| has(x + a, y + d)));
            for &(x, y) in &cells {
                assert!(
                    [(0, 0), (-1, 0), (0, -1), (-1, -1)]
                        .iter()
                        .any(|(dx, dy)| square(x + dx, y + dy, 2)),
                    "a single cell sticks out at ({x}, {y})"
                );
                assert_eq!(site.level.room(x, y), Some(RoomId(building.look.id)));
            }
            assert!(cells.iter().any(|&(x, y)| square(x, y, 3)));
        }
    }
}

#[test]
fn gardens_hold_a_three_by_three_and_nothing_sticks_out() {
    let mut found = 0;
    for (_, site) in towns() {
        let level = &site.level;
        let (x0, y0, w, h) = level.bounds();
        let is = |x, y| level.cell(x, y) == Cell::Garden;
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                if is(x, y) {
                    found += 1;
                    assert!([(0, 0), (-1, 0), (0, -1), (-1, -1)].iter().any(|(dx, dy)| {
                        is(x + dx, y + dy)
                            && is(x + dx + 1, y + dy)
                            && is(x + dx, y + dy + 1)
                            && is(x + dx + 1, y + dy + 1)
                    }));
                }
            }
        }
    }
    assert!(found > 0);
}

/// Doors stand in straight walls facing something walkable, a road unless
/// the building is a house.
#[test]
fn doors_face_the_right_ground() {
    for (_, site) in towns() {
        let level = &site.level;
        for building in &site.buildings {
            let (x, y) = building.look.door;
            let (dx, dy) = building.look.faces;
            assert_eq!(level.cell(x, y), Cell::BuildingDoor);
            let front = level.cell(x + dx, y + dy);
            assert!(front.is_walkable());
            if building.look.kind != BuildingKind::House {
                assert!(front.is_road(), "{:?}", building.look.kind);
            }
            let same = |nx, ny| level.room(nx, ny) == Some(RoomId(building.look.id));
            assert!(same(x + dy, y + dx) && same(x - dy, y - dx));
            assert!(same(x - dx, y - dy));
        }
    }
}

/// All walkable cells can be walked to, by steps along the grid, so
/// anything surrounded by buildings has a gap, and the way in is on the
/// edge of the level.
#[test]
fn everything_can_be_reached() {
    for (target, site) in towns() {
        let entrance = site.entrance.unwrap();
        assert!(super::super::leads_out(&site.level, entrance.0, entrance.1));
        let mut reach = Reach::new(&site.level);
        reach.spread(&site.level, &[entrance]);
        assert_eq!(reach.first_unreached(&site.level), None, "{target}");
    }
}

#[test]
fn numbers_stay_in_range_and_grow_with_the_town() {
    let top = |target| {
        town(target, 3)
            .buildings
            .iter()
            .map(|b| b.trades)
            .max()
            .unwrap()
    };
    assert!(top(150) > top(5));
    for (_, site) in towns() {
        for building in &site.buildings {
            assert!(building.quality <= 1000);
            assert!((10..=1000).contains(&building.trades));
        }
    }
}

#[test]
fn building_kinds_follow_their_rules() {
    for (_, site) in towns() {
        let count = site.buildings.len() as i64;
        let mut copies: BTreeMap<BuildingKind, i64> = BTreeMap::new();
        for building in &site.buildings {
            *copies.entry(building.look.kind).or_default() += 1;
            assert!(building.look.kind.odds().0 <= count);
        }
        // Houses are at least about their share.
        let houses = copies.get(&BuildingKind::House).copied().unwrap_or(0);
        assert!(houses * 100 >= count * HOUSE_PERCENT - 50);
        // Nothing else takes over: halving spreads them out.
        for (&kind, &had) in &copies {
            if kind != BuildingKind::House {
                assert!(had <= (count / 8).max(4), "{kind:?}: {had} of {count}");
            }
        }
        // Every town has a shrine or an inn, if it has more than houses.
        if copies.len() > 1 {
            assert!(
                copies.contains_key(&BuildingKind::Shrine)
                    || copies.contains_key(&BuildingKind::Inn)
            );
        }
    }
}

#[test]
fn roads_are_better_in_bigger_towns() {
    let good = |target| {
        (0..40)
            .filter(|&seed| {
                let site = town(target, seed);
                let (x, y) = site.entrance.unwrap();
                matches!(
                    site.level.cell(x, y),
                    Cell::Cobblestone | Cell::Flagstones | Cell::DressedStone
                )
            })
            .count()
    };
    assert!(good(150) > good(4));
}

#[test]
fn big_towns_always_have_a_square_and_small_ones_never() {
    let road_in_a_block = |site: &SiteLevel| {
        let level = &site.level;
        let (x0, y0, w, h) = level.bounds();
        (y0..y0 + h - 5).any(|y| {
            (x0..x0 + w - 5)
                .any(|x| (0..6).all(|d| (0..6).all(|a| level.cell(x + a, y + d).is_road())))
        })
    };
    for seed in 0..10 {
        assert!(road_in_a_block(&town(60, seed)));
        assert!(!road_in_a_block(&town(5, seed)));
    }
}

/// Nobody has too far to walk round: a line of building only passes the
/// limit by the one building that was last to join it.
#[test]
fn long_lines_of_building_have_gaps() {
    const MOST: i64 = LONGEST_WITHOUT_GAP - 1 + 9;
    for (target, site) in towns() {
        let level = &site.level;
        let (x0, y0, w, h) = level.bounds();
        let longest = |cells: Vec<Vec<(i64, i64)>>| {
            cells
                .iter()
                .map(|line| {
                    line.split(|&(x, y)| !level.cell(x, y).is_wall())
                        .map(|run| run.len() as i64)
                        .max()
                        .unwrap_or(0)
                })
                .max()
                .unwrap_or(0)
        };
        let rows = (y0..y0 + h)
            .map(|y| (x0..x0 + w).map(|x| (x, y)).collect())
            .collect();
        let columns = (x0..x0 + w)
            .map(|x| (y0..y0 + h).map(|y| (x, y)).collect())
            .collect();
        assert!(longest(rows) <= MOST, "{target}");
        assert!(longest(columns) <= MOST, "{target}");
    }
}

/// Roads don't run over each other, or side by side with no room for a
/// building, unless they are all the main roads and square.
#[test]
fn roads_keep_apart() {
    for target in [12, 30, 60, 100, 150] {
        for seed in 0..12 {
            let mut town = Town::new(Seed::new(seed), target);
            town.lay_out();
            let roads = &town.roads;
            for (i, a) in roads.iter().enumerate() {
                for b in &roads[i + 1..] {
                    if a.fixed && b.fixed {
                        continue;
                    }
                    let (m, n) = (a.area(a.along.0, a.along.1), b.area(b.along.0, b.along.1));
                    let overlap = m.x0 < n.x1 && n.x0 < m.x1 && m.y0 < n.y1 && n.y0 < m.y1;
                    assert!(!overlap, "{a:?} over {b:?}");
                    if a.east_west == b.east_west
                        && a.along.0 <= b.along.1
                        && b.along.0 <= a.along.1
                    {
                        assert!(
                            a.across.0 >= b.across.1 + BETWEEN_ROADS
                                || b.across.0 >= a.across.1 + BETWEEN_ROADS,
                            "{a:?} beside {b:?}"
                        );
                    }
                }
            }
        }
    }
}

/// A road running up to the border carries on to the level's edge, unless
/// it meets another road on the way.
#[test]
fn roads_at_the_border_reach_the_edge() {
    for target in [8, 20, 50, 80, 150] {
        for seed in 0..12 {
            let mut town = Town::new(Seed::new(seed), target);
            let (arms, middle, main) = town.lay_out();
            let roads = town.roads.clone();
            let site = town.finish(arms, middle, main);
            let level = &site.level;
            let (x0, y0, width, height) = level.bounds();
            let on_a_road = |x: i64, y: i64| {
                roads.iter().any(|r| {
                    let area = r.area(r.along.0, r.along.1);
                    (area.x0..area.x1).contains(&x) && (area.y0..area.y1).contains(&y)
                })
            };
            for road in &roads {
                let (first, last) = if road.east_west {
                    (x0, x0 + width)
                } else {
                    (y0, y0 + height)
                };
                // Out from its end: every section road, up to one that
                // another road crosses or the edge.
                let stops_short = |way: Vec<i64>| {
                    way.into_iter().any(|a| {
                        let area = road.area(a, a + 1);
                        let cells: Vec<(i64, i64)> = (area.y0..area.y1)
                            .flat_map(|y| (area.x0..area.x1).map(move |x| (x, y)))
                            .collect();
                        !cells.iter().any(|&(x, y)| on_a_road(x, y))
                            && cells.iter().any(|&(x, y)| !level.cell(x, y).is_road())
                    })
                };
                let crossed = |way: &[i64]| {
                    way.iter().position(|&a| {
                        let area = road.area(a, a + 1);
                        (area.y0..area.y1).any(|y| (area.x0..area.x1).any(|x| on_a_road(x, y)))
                    })
                };
                let before = |way: Vec<i64>| {
                    let stop = crossed(&way).unwrap_or(way.len());
                    way[..stop].to_vec()
                };
                if road.along.0 <= first + BORDER {
                    let way = before((first..road.along.0).rev().collect());
                    assert!(!stops_short(way), "{target}, {seed}: {road:?}");
                }
                if road.along.1 >= last - BORDER {
                    let way = before((road.along.1..last).collect());
                    assert!(!stops_short(way), "{target}, {seed}: {road:?}");
                }
            }
        }
    }
}

#[test]
fn neighbouring_buildings_do_not_join() {
    let mut level = Level::new(0, 0, 5, 3);
    for x in 0..4 {
        level.set(x, 1, Cell::Wall);
        level.set_room(x, 1, Some(RoomId(u16::from(x >= 2))));
    }
    assert!(level.wall_shape(0, 1).unwrap().east);
    assert!(!level.wall_shape(1, 1).unwrap().east);
    assert!(!level.wall_shape(2, 1).unwrap().west);
}
