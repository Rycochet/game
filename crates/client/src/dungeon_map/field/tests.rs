use super::*;
use shared::site::Rect;

fn field(livestock: bool, variety: u32) -> FieldLook {
    FieldLook {
        rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 8,
        },
        livestock,
        variety,
    }
}

#[test]
fn crops_go_through_every_stage_and_pasture_stays_short() {
    for (variety, stage) in Stage::CROPS.into_iter().enumerate() {
        assert_eq!(Stage::of(&field(false, variety as u32)), stage);
    }
    for variety in 0..20 {
        let stage = Stage::of(&field(true, variety));
        assert!(matches!(stage, Stage::Grass | Stage::Short));
    }
}

#[test]
fn crops_grow_taller() {
    let heights = [Stage::Short, Stage::Tall, Stage::Ripe].map(|s| s.crop().unwrap().1);
    assert_eq!(heights, [6.0, 12.0, 18.0]);
    assert_eq!(Stage::Grass.crop(), None);
}

/// About 20% land, 20% ploughed and 60% grass.
#[test]
fn ruined_fields_are_a_mix() {
    let mut counts = [0; 3];
    for y in 0..200 {
        for x in 0..200 {
            counts[match ruined_tile(x, y) {
                None => 0,
                Some(PLOUGHED_TILE) => 1,
                Some(_) => 2,
            }] += 1;
        }
    }
    assert!((7000..9000).contains(&counts[0]), "{counts:?}");
    assert!((7000..9000).contains(&counts[1]), "{counts:?}");
    assert!((23000..25000).contains(&counts[2]), "{counts:?}");
}
