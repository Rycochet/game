use super::*;
use crate::common::CreateMobOrder;
use shared::climate::Biome;
use shared::sheet::Feeling;

#[test]
fn a_dev_area_is_kept_and_read_back() {
    let form = DevAreaForm {
        size: 64,
        layout: DevLayout::Biome(Biome::BorealForest),
        mobs: Some(Some(CreatureKind::Wolf)),
        count: 5,
        seed: "woods".into(),
        seeds_made: 3,
    };
    let kept = dev_area_prefs(&form);
    assert_eq!(kept.generator, "Boreal Forest");
    let back = dev_area_form(&kept, 3);
    assert_eq!(back.area(), form.area());
}

#[test]
fn names_no_longer_known_start_as_the_panel_does() {
    let kept = DevAreaPrefs {
        size: 7,
        generator: "Maze".into(),
        mobs: "Dragon".into(),
        count: 0,
        seed: "x".into(),
    };
    let back = dev_area_form(&kept, 0);
    assert_eq!(back.size, 32);
    assert_eq!(back.layout, DevLayout::Empty);
    assert_eq!(back.mobs, None);
    assert_eq!(back.count, 1);
}

#[test]
fn a_mob_order_is_kept_and_read_back() {
    let order = DevMob {
        kind: CreatureKind::Orc,
        attitude: Some(Feeling::Friendly),
        still: true,
        gear: false,
        seed: 42,
        ..CreateMobOrder::default().0
    };
    let back = mob_order(&mob_prefs(&order), CreateMobOrder::default().0);
    assert_eq!(back, order);
}
