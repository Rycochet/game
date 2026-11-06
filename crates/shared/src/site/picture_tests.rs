use super::site::Plan;
use super::*;
use crate::dungeon::Cell;
use crate::poi::PoiKind;
use crate::rng::Seed;

/// Prints levels as text: `cargo test -p shared print_sites -- --ignored --nocapture`
/// (`HOW_DEBUG_SITE_KINDS`, `HOW_DEBUG_SITE_SEED`, `HOW_DEBUG_SITE_RUIN` choose what).
#[test]
#[ignore]
fn print_sites() {
    let kinds = std::env::var("HOW_DEBUG_SITE_KINDS").unwrap_or("20,4,29".into());
    let seed: u64 = std::env::var("HOW_DEBUG_SITE_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let ruin: u8 = std::env::var("HOW_DEBUG_SITE_RUIN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    for kind in kinds
        .split(',')
        .map(|k| PoiKind::ALL[k.parse::<usize>().unwrap()])
    {
        let site = Site::with_generator(
            kind,
            Seed::new(seed),
            Generator::for_poi(kind).version(),
            Generator::for_poi(kind).with_ruin(ruin),
        )
        .expect("the current version runs");
        for index in 0..site.levels() {
            let site_level = site.level(index);
            let level = &site_level.level;
            println!("{kind:?} level {index} of {}", site.levels());
            let (min_x, min_y, width, height) = level.bounds();
            for y in min_y..min_y + height {
                let row: String = (min_x..min_x + width)
                    .map(|x| {
                        if let Some(s) = site_level.stairs.iter().find(|s| (s.x, s.y) == (x, y)) {
                            return if s.direction == StairsDirection::Up {
                                '<'
                            } else {
                                '>'
                            };
                        }
                        match level.cell(x, y) {
                            Cell::Floor => match level.room(x, y) {
                                Some(r) => (b'a' + (r.0 % 26) as u8) as char,
                                None => '.',
                            },
                            Cell::Wall => '#',
                            Cell::Door => '+',
                            Cell::SecretDoor => 'S',
                            Cell::Ground => ',',
                            Cell::CoveredGround => '_',
                            Cell::Water => '~',
                            Cell::Lava => '^',
                            Cell::Hole => ' ',
                            Cell::Window => 'W',
                            Cell::BlindWindow => 'w',
                            Cell::BuildingDoor => '+',
                            Cell::Garden => '"',
                            Cell::Field => ':',
                            Cell::Fence => '|',
                            Cell::FieldGate => '/',
                            Cell::HerbBed => ';',
                            cell if cell.is_road() => '=',
                            cell => panic!("no letter for {cell:?}"),
                        }
                    })
                    .collect();
                println!("{row}");
            }
        }
    }
}

#[test]
fn about_a_third_of_sturdy_buildings_are_ruined_at_every_level() {
    let mut counts = [0; 6];
    for seed in 0..3000 {
        let look = PoiLook::of(PoiKind::Castle, Seed::new(seed), false);
        let PoiLook::Building { ruin, .. } = look else {
            panic!("a castle is a building");
        };
        counts[ruin as usize] += 1;
    }
    assert!((1900..2300).contains(&counts[0]), "{counts:?}");
    for count in &counts[1..] {
        assert!((100..260).contains(count), "{counts:?}");
    }
}

#[test]
fn tents_are_never_ruined() {
    assert!(
        (0..300).all(|seed| !PoiLook::of(PoiKind::RedTent, Seed::new(seed), false).is_ruined())
    );
}

fn floors_of(kind: PoiKind) -> impl Iterator<Item = Site> {
    (0..60).map(move |n| Site::new(kind, Seed::new(n * 31 + 7)))
}

#[test]
fn castles_and_forts_have_at_least_three_floors() {
    for kind in [PoiKind::Castle, PoiKind::Fort] {
        assert!(floors_of(kind).all(|site| site.levels() >= 3), "{kind:?}");
    }
}

/// Each floor is 3 or 4 cells smaller per side, and the top leaves no room
/// for another: its walls and a 3 by 3 room at most one step from too small.
#[test]
fn pyramids_step_in_until_no_room_fits() {
    for kind in [PoiKind::StonePyramid, PoiKind::SandPyramid] {
        for site in floors_of(kind) {
            assert!(site.levels() >= 3, "{}", site.levels());
            let plan = building_plan(&site);
            let sizes = plan.floor_widths();
            for pair in sizes.windows(2) {
                let by = (pair[0] - pair[1]) / 2;
                assert!((3..=4).contains(&by), "{sizes:?}");
            }
            let top = sizes[sizes.len() - 1];
            assert!((5..11).contains(&top), "{sizes:?}");
        }
    }
}

/// At least 5 floors, all alike, except that a tower with more may have a
/// top about half the size.
#[test]
fn towers_are_tall_and_the_same_all_the_way_up() {
    for kind in [PoiKind::StoneTower, PoiKind::WoodenTower] {
        let mut small_tops = 0;
        for site in floors_of(kind) {
            let sizes = building_plan(&site).floor_widths();
            assert!(sizes.len() >= 5, "{sizes:?}");
            let (top, rest) = sizes.split_last().unwrap();
            assert!(rest.iter().all(|&size| size == sizes[0]), "{sizes:?}");
            if *top != sizes[0] {
                small_tops += 1;
                assert!(sizes.len() > 5, "{sizes:?}");
                assert!((sizes[0] / 3..=sizes[0] * 2 / 3).contains(top), "{sizes:?}");
            }
        }
        assert!(small_tops > 0);
    }
}

fn building_plan(site: &Site) -> &building::Plan {
    match &site.plan {
        Plan::Building(plan) => plan,
        _ => panic!("a building"),
    }
}

#[test]
fn only_big_buildings_may_have_extra_stairs() {
    let most_stairs_up = |kind| {
        floors_of(kind)
            .flat_map(|site| {
                (0..site.levels()).map(move |index| {
                    let level = site.level(index);
                    let up = |s: &&Stairs| s.direction == StairsDirection::Up;
                    level.stairs.iter().filter(up).count()
                })
            })
            .max()
            .unwrap()
    };
    assert_eq!(most_stairs_up(PoiKind::StoneCottage), 1);
    assert!(most_stairs_up(PoiKind::Castle) > 1);
}
