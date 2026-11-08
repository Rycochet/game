use super::*;
use crate::movement::{DEEPEST_SEA, MOVEMENT_SKILLS};
use crate::skill::Skills;
use crate::world::{WorldParams, WorldSeed};

fn swimmer(percent: u32) -> Gait {
    let mut skills = Skills::default();
    skills.practise(MOVEMENT_SKILLS.swimming, percent.pow(3));
    Gait::of(&skills)
}

#[test]
fn good_swimmers_swim_water_on_a_level_instead_of_jumping_it() {
    let mut level = Level::new(0, 0, 3, 1);
    level.set(0, 0, Cell::Floor);
    level.set(1, 0, Cell::Water);
    level.set(2, 0, Cell::Floor);
    let east = Step::new(1, 0).unwrap();
    assert_eq!(
        step_on_level_by(&level, (0, 0), east, swimmer(40)),
        Some((2, 0))
    );
    assert_eq!(
        step_on_level_by(&level, (0, 0), east, swimmer(50)),
        Some((1, 0))
    );
    // Lava is never swum.
    level.set(1, 0, Cell::Lava);
    assert_eq!(
        step_on_level_by(&level, (0, 0), east, swimmer(100)),
        Some((2, 0))
    );
}

#[test]
fn only_the_coast_is_swum_on_the_world_map() {
    let world = WorldParams::from_seed(WorldSeed::new(12345).terrain());
    let ground = WorldGround::new(&world);
    let conditions = WorldConditions {
        water: world.high_water(),
        ice: true,
    };
    assert!(swimmer(40).deepest_sea() < 0.0);
    assert_eq!(swimmer(100).deepest_sea(), DEEPEST_SEA);
    // A coast: dry ground with the sea beside it, shallow then deep.
    let (mut shallow, mut deep) = (None, None);
    'search: for y in (-1500..1500).step_by(4) {
        for x in -1500..1500 {
            let dry = ground.elevation(x, y, conditions).is_some();
            let depth = ground.sea_depth(x + 1, y, conditions);
            if dry && depth.is_some_and(|d| d < DEEPEST_SEA / 2.0) && shallow.is_none() {
                shallow = Some((x, y));
            }
            if ground
                .sea_depth(x, y, conditions)
                .is_some_and(|d| d > DEEPEST_SEA * 4.0)
            {
                deep = Some((x, y));
            }
            if shallow.is_some() && deep.is_some() {
                break 'search;
            }
        }
    }
    let shallow = shallow.expect("a shallow coast");
    let east = Step::new(1, 0).unwrap();
    let into = |gait| step_on_world_map_by(&ground, conditions, shallow, east, gait);
    assert_eq!(into(swimmer(40)), None, "too weak a swimmer");
    assert_eq!(into(swimmer(100)), Some((shallow.0 + 1, shallow.1)));
    let deep = deep.expect("a deep sea");
    assert!(!ground.swimmable(deep.0, deep.1, conditions, swimmer(100)));
}
