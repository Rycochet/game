use super::*;
use crate::material::Category;

fn made(kind: ItemKind, seed: u128, quality: Quality, size: Option<Size>) -> Item {
    let mut item = Item::new(Id::for_test(seed), kind);
    item.quality = Some(quality);
    let mut next = 1 << 64;
    give_form(&mut item, size, &mut || {
        next += 1;
        Id::for_test(next)
    });
    item
}

fn gem(seed: u128, quality: Quality) -> (GemForm, Size) {
    let item = made(ItemKind::Gem, seed, quality, None);
    match item.form {
        Some(ItemForm::Gem(gem)) => (gem, item.made.size),
        other => panic!("{other:?}"),
    }
}

fn ring_form(item: &Item) -> &RingForm {
    match &item.form {
        Some(ItemForm::Ring(ring)) => ring,
        other => panic!("{other:?}"),
    }
}

fn rings(quality: Quality) -> impl Iterator<Item = Item> {
    (0..4000).map(move |seed| made(ItemKind::Ring, seed, quality, None))
}

fn share_of(quality: Quality, wanted: impl Fn(&GemForm, Size) -> bool) -> u32 {
    let found = (0..4000).filter(|&seed| {
        let (gem, size) = gem(seed, quality);
        wanted(&gem, size)
    });
    found.count() as u32
}

fn stone(material: Material) -> Setting {
    Setting {
        angle: 0,
        size: Size::Medium,
        mount: Mount::Claws,
        gem: Some(Item {
            form: Some(ItemForm::Gem(GemForm {
                material,
                cut: Cut::Round,
                shape_seed: 0,
            })),
            ..Item::new(Id::for_test(1), ItemKind::Gem)
        }),
    }
}

fn ring_with(settings: Vec<Setting>) -> ItemForm {
    ItemForm::Ring(Box::new(RingForm {
        metal: Material::Gold,
        profile: Profile::Flat,
        width: BandWidth::Medium,
        pattern: Pattern::Plain,
        settings,
    }))
}

#[test]
fn the_same_seed_rolls_the_same_gem() {
    // Golden: changing the rolls is fine, but should be noticed.
    let (rolled, size) = gem(1, Quality::Common);
    assert_eq!(gem(1, Quality::Common), (rolled, size));
    assert_eq!(
        (rolled.material, rolled.cut, size),
        (Material::Emerald, Cut::Oval, Size::Medium)
    );
}

#[test]
fn the_same_seed_rolls_the_same_ring() {
    let one = made(ItemKind::Ring, 7, Quality::Rare, None);
    assert_eq!(one, made(ItemKind::Ring, 7, Quality::Rare, None));
}

#[test]
fn blades_take_their_weight_and_shape_from_their_form() {
    let sword = made(ItemKind::Sword, 1, Quality::Common, None);
    let Some(ItemForm::Blade(form)) = &sword.form else {
        panic!("{sword:?}");
    };
    assert_eq!(sword.total_weight(), form.grams());
    assert_eq!(sword.shape(), form.shape());
    assert_eq!(sword.name(), form.name());
    let giant = made(ItemKind::Sword, 1, Quality::Common, Some(Size::Giant));
    assert_eq!(giant.name(), "Giant Sword");
}

#[test]
fn a_commission_is_named_with_its_size() {
    let mut item = Item::new(Id::for_test(1), ItemKind::Coin);
    assert!(give_commission(
        &mut item,
        "Katana",
        Size::Tiny,
        &Limits::NONE
    ));
    assert_eq!(item.kind, ItemKind::Sword);
    assert_eq!(item.name(), "Tiny Katana");
}

#[test]
fn a_blade_within_limits_may_change_kind_or_not_be_made() {
    let limits = blade::limits_for_wielder(Size::Tiny, 5000);
    let got = (0..50).map(|seed| {
        let mut item = Item::new(Id::for_test(seed), ItemKind::Sword);
        assert!(give_blade_within(&mut item, &limits));
        item.kind
    });
    assert!(got.into_iter().any(|kind| kind != ItemKind::Sword));
    let mut item = Item::new(Id::for_test(1), ItemKind::Sword);
    let none = Limits {
        most_grams: Some(1),
        ..Limits::NONE
    };
    assert!(!give_blade_within(&mut item, &none));
}

#[test]
fn only_kinds_with_a_form_roll_one() {
    let apple = made(ItemKind::Apple, 1, Quality::Common, None);
    assert_eq!(apple.form, None);
}

#[test]
fn a_size_given_is_kept_and_the_gem_stays_the_same() {
    let told = made(ItemKind::Gem, 1, Quality::Common, Some(Size::Giant));
    assert_eq!(told.made.size, Size::Giant);
    assert_eq!(told.form, Some(ItemForm::Gem(gem(1, Quality::Common).0)));
}

#[test]
fn every_gem_rolled_is_a_gem_and_only_uncut_ones_have_a_shape_seed() {
    for seed in 0..4000 {
        let (gem, _) = gem(seed, Quality::Rare);
        assert_eq!(gem.material.category(), Category::Gem);
        assert_eq!(gem.cut == Cut::Uncut, gem.shape_seed != 0, "{gem:?}");
    }
}

#[test]
fn better_gems_are_more_often_fine() {
    let diamonds = |quality| share_of(quality, |gem, _| gem.material == Material::Diamond);
    assert!(diamonds(Quality::Legendary) > diamonds(Quality::Poor) * 4);
}

#[test]
fn stones_favour_the_cuts_they_are_known_for() {
    let emerald_cut = |wanted: Material| {
        let of_it = (0..40_000).map(|seed| gem(seed, Quality::Rare).0);
        let of_it: Vec<_> = of_it.filter(|gem| gem.material == wanted).collect();
        let cut = of_it.iter().filter(|gem| gem.cut == Cut::Emerald).count();
        cut as f64 / of_it.len() as f64
    };
    assert!(emerald_cut(Material::Emerald) > emerald_cut(Material::Ruby) * 3.0);
}

#[test]
fn the_finest_stones_never_come_giant() {
    let fine_giants = |gem: &GemForm, size| gem.material.tier() >= 3 && size == Size::Giant;
    assert_eq!(share_of(Quality::Legendary, fine_giants), 0);
    let any_giants = |_: &GemForm, size| size == Size::Giant;
    assert!(share_of(Quality::Common, any_giants) > 0);
}

#[test]
fn a_gem_is_named_by_its_cut_and_stone() {
    let form = ItemForm::Gem(GemForm {
        material: Material::Ruby,
        cut: Cut::Uncut,
        shape_seed: 3,
    });
    assert_eq!(form.name(), "Uncut Ruby");
}

#[test]
fn a_ring_is_named_by_its_metal_and_what_is_in_it() {
    assert_eq!(ring_with(vec![]).name(), "Gold Ring");
    assert_eq!(
        ring_with(vec![stone(Material::Ruby)]).name(),
        "Gold Ring with a Ruby"
    );
    assert_eq!(
        ring_with(vec![stone(Material::Emerald)]).name(),
        "Gold Ring with an Emerald"
    );
    assert_eq!(
        ring_with(vec![stone(Material::Ruby), stone(Material::Ruby)]).name(),
        "Gold Ring with Two Rubies"
    );
    let mut empty = stone(Material::Ruby);
    empty.gem = None;
    assert_eq!(
        ring_with(vec![stone(Material::Ruby), stone(Material::Topaz), empty]).name(),
        "Gold Ring with a Ruby, a Topaz and an Empty Setting"
    );
}

#[test]
fn rings_are_metal_and_their_gems_are_items_of_their_own_that_fit() {
    for ring in rings(Quality::Epic) {
        assert_eq!(ring_form(&ring).metal.category(), Category::Metal);
        let mut ids = std::collections::BTreeSet::new();
        for setting in &ring_form(&ring).settings {
            assert!(setting.size <= ring.made.size);
            if let Some(gem) = &setting.gem {
                assert_eq!(gem.kind, ItemKind::Gem);
                assert!(setting.takes(gem), "{gem:?} in {:?}", setting.size);
                assert!(ids.insert(gem.id), "the same id twice in a ring");
                assert!(gem.quality < Some(Quality::Artifact));
            }
        }
    }
}

#[test]
fn a_setting_takes_only_cut_gems_up_to_one_size_too_big() {
    let setting = stone(Material::Ruby);
    let gem = setting.gem.clone().unwrap();
    let at = |size| Item {
        made: Made::for_body(ItemKind::Gem, size, None, false),
        ..gem.clone()
    };
    assert!(setting.takes(&at(Size::Tiny)));
    assert!(setting.takes(&at(Size::Medium)));
    assert!(setting.takes(&at(Size::Large)));
    assert!(!setting.takes(&at(Size::Huge)));
    let uncut = Item {
        form: Some(ItemForm::Gem(GemForm {
            material: Material::Ruby,
            cut: Cut::Uncut,
            shape_seed: 1,
        })),
        ..gem
    };
    assert!(!setting.takes(&uncut));
    assert!(!setting.takes(&Item::new(Id::for_test(2), ItemKind::Apple)));
}

#[test]
fn several_gems_in_a_ring_mostly_match_the_first() {
    let (mut same, mut others) = (0, 0);
    for ring in rings(Quality::Legendary) {
        let stones: Vec<_> = ring.set_gems().filter_map(gem_material).collect();
        if let Some((first, rest)) = stones.split_first() {
            same += rest.iter().filter(|stone| *stone == first).count();
            others += rest.iter().filter(|stone| *stone != first).count();
        }
    }
    assert!(same > others * 8, "{same} against {others}");
}

#[test]
fn some_settings_are_left_empty_but_few() {
    let settings: Vec<bool> = rings(Quality::Legendary)
        .flat_map(|ring| {
            let settings = ring_form(&ring).settings.clone();
            settings.into_iter().map(|setting| setting.gem.is_some())
        })
        .collect();
    let empty = settings.iter().filter(|filled| !**filled).count();
    assert!(empty > 0);
    assert!(empty * 10 < settings.len(), "{empty} of {}", settings.len());
}

#[test]
fn poor_rings_mostly_have_no_settings_and_fine_ones_mostly_do() {
    let with_settings = |quality| {
        rings(quality)
            .filter(|ring| !ring_form(ring).settings.is_empty())
            .count()
    };
    assert!(with_settings(Quality::Poor) < 1500);
    assert!(with_settings(Quality::Legendary) > 2000);
}

#[test]
fn better_rings_are_more_often_of_finer_metal() {
    let gold = |quality| {
        rings(quality)
            .filter(|ring| ring_form(ring).metal == Material::Gold)
            .count()
    };
    assert!(gold(Quality::Legendary) > gold(Quality::Poor) * 3);
}

#[test]
fn settings_sit_evenly_either_side_of_the_top() {
    assert_eq!(setting_angle(0, 1), 0);
    assert_eq!(setting_angle(1, 3), 0);
    assert_eq!(setting_angle(0, 3), (256 - SETTING_SPACING as i32) as u8);
    assert_eq!(setting_angle(2, 3), SETTING_SPACING);
}

#[test]
fn a_ring_given_a_size_keeps_it() {
    let ring = made(ItemKind::Ring, 3, Quality::Rare, Some(Size::Huge));
    assert_eq!(ring.made.size, Size::Huge);
}

#[test]
fn set_gems_count_towards_a_rings_weight() {
    let found = rings(Quality::Legendary)
        .find(|ring| ring.set_gems().count() > 1)
        .unwrap();
    let gems = found.set_gems().count() as u32;
    assert_eq!(
        found.total_weight(),
        ItemKind::Ring.def().weight + gems * ItemKind::Gem.def().weight
    );
}
