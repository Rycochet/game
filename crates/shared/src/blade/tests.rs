use super::*;
use crate::rng::Seed;
use std::collections::BTreeSet;

fn rolled(family: Family) -> impl Iterator<Item = BladeForm> {
    (0..6000).map(move |seed| roll_blade(&mut Seed::new(seed).rng(), family, Quality::Common))
}

fn all_names() -> BTreeSet<&'static str> {
    Family::ALL
        .into_iter()
        .flat_map(rolled)
        .map(|form| form.name())
        .collect()
}

fn sword(blade: u16, handle: u16) -> BladeForm {
    roll_properties(
        &mut Seed::new(1).rng(),
        ROWS.iter().find(|row| row.name == "Sword").unwrap(),
        Quality::Common,
        blade,
        handle,
    )
}

#[test]
fn the_same_seed_rolls_the_same_weapon() {
    // Golden: changing the rolls is fine, but should be noticed.
    let form = roll_blade(&mut Seed::new(1).rng(), Family::Sword, Quality::Common);
    assert_eq!(
        form,
        roll_blade(&mut Seed::new(1).rng(), Family::Sword, Quality::Common)
    );
    assert_eq!((form.blade, form.handle, form.name()), (609, 298, "Katana"));
}

#[test]
fn every_row_can_be_made_somewhere() {
    for row in &ROWS {
        assert!(row.places().next().is_some(), "{} has no cell", row.name);
    }
}

#[test]
fn every_specific_name_turns_up() {
    let names = all_names();
    for row in ROWS.iter().filter(|row| !row.generic) {
        assert!(names.contains(row.name), "{} never turns up", row.name);
    }
}

#[test]
fn nothing_is_made_outside_the_grids() {
    for form in Family::ALL.into_iter().flat_map(rolled) {
        assert!(
            cell_of(form.build, form.blade, form.handle).is_some(),
            "{form:?}"
        );
        assert_ne!(form.name(), "Blade", "{form:?}");
    }
}

#[test]
fn a_short_blade_is_never_on_a_long_grip() {
    assert!(cell_of(Build::Along, 200, 300).is_none());
    assert!(cell_of(Build::Along, 500, 300).is_none());
}

#[test]
fn short_blades_are_named_by_their_handle() {
    let named = |handle| sword(200, handle).name();
    assert!(["Knife", "Dagger"].contains(&named(120)));
    assert_eq!(
        cell_of(Build::Along, 200, 700).unwrap().generic,
        Some("Stabbing Spear")
    );
    assert_eq!(
        cell_of(Build::Along, 200, 2000).unwrap().generic,
        Some("Spear")
    );
    assert_eq!(
        cell_of(Build::Along, 200, 4000).unwrap().generic,
        Some("Pike")
    );
}

#[test]
fn a_knife_with_two_edges_is_a_dagger() {
    let mut form = sword(200, 120);
    form.guard = Guard::Cup;
    form.pommel = Pommel::Wheel;
    form.edges = Edges::Single;
    form.curve = Curve::Wavy;
    assert_eq!(form.name(), "Knife");
    form.edges = Edges::Double;
    assert_eq!(form.name(), "Dagger");
}

#[test]
fn a_katana_and_a_bastard_sword_share_a_cell_but_not_a_name() {
    let cell = |name: &str| {
        let family = ROWS.iter().find(|row| row.name == name).unwrap().family;
        rolled(family)
            .find(|form| form.name() == name && form.blade < 800)
            .map(|form| cell_of(form.build, form.blade, form.handle).unwrap().blade)
            .unwrap()
    };
    assert_eq!(cell("Katana"), cell("Bastard Sword"));
    let katana = rolled(Family::Sword)
        .find(|f| f.name() == "Katana")
        .unwrap();
    let bastard = rolled(Family::LongSword)
        .find(|f| f.name() == "Bastard Sword")
        .unwrap();
    assert_eq!((katana.edges, katana.curve), (Edges::Single, Curve::Slight));
    assert_eq!(
        (bastard.edges, bastard.curve),
        (Edges::Double, Curve::Straight)
    );
}

#[test]
fn a_great_blade_in_one_hand_is_narrow() {
    for form in rolled(Family::Sword) {
        if form.blade >= GREAT.0 && form.handle < GRIP.1 {
            assert_eq!(form.width, BladeWidth::Narrow);
        }
    }
}

#[test]
fn specifics_win_over_generics() {
    let rapier = rolled(Family::Sword).find(|f| f.blade >= GREAT.0).unwrap();
    assert_eq!(rapier.name(), "Rapier");
}

#[test]
fn axes_have_a_head_at_the_side_and_nothing_along_the_handle() {
    for form in rolled(Family::Axe) {
        assert_eq!(form.build, Build::Side);
        assert_eq!(form.attachment(), Attachment::SideMounted);
        assert_ne!(form.top, Top::Blade);
        assert!(form.sides.contains(&SidePart::AxeBlade), "{form:?}");
    }
}

#[test]
fn a_labrys_has_a_blade_each_side_and_a_halberd_differs() {
    let labrys = rolled(Family::Axe).find(|f| f.name() == "Labrys").unwrap();
    assert_eq!(labrys.sides, [SidePart::AxeBlade, SidePart::AxeBlade]);
    let halberd = rolled(Family::Polearm)
        .find(|f| f.name() == "Halberd")
        .unwrap();
    assert_eq!(halberd.sides[0], SidePart::AxeBlade);
    assert_ne!(halberd.sides[1], SidePart::None);
    assert_eq!(halberd.top, Top::Spike);
}

#[test]
fn long_shafts_are_socketed_and_grips_have_tangs() {
    assert_eq!(sword(800, 150).attachment(), Attachment::Tang);
    let spear = rolled(Family::Spear).next().unwrap();
    assert_eq!(spear.attachment(), Attachment::Socket);
}

fn asked_for(seed: u64, family: Family, limits: &Limits) -> Option<Rolled> {
    roll_blade_within(&mut Seed::new(seed).rng(), family, Quality::Common, limits)
}

#[test]
fn a_plain_sword_is_about_what_the_placeholder_was() {
    let swords: Vec<_> = rolled(Family::Sword)
        .filter(|form| form.name() == "Sword")
        .collect();
    let grams: u32 = swords.iter().map(|form| form.grams()).sum();
    let average = grams / swords.len() as u32;
    assert!((800..1800).contains(&average), "{average}");
    assert!(swords.iter().all(|form| form.shape().height >= 4));
}

#[test]
fn longer_weighs_more_and_size_is_cubed() {
    let mut form = sword(800, 150);
    let light = form.grams();
    form.blade = 900;
    assert!(form.grams() > light);
    assert_eq!(form.grams_at(Size::Giant), form.grams() * 8);
    assert_eq!(form.grams_at(Size::Medium), form.grams());
}

#[test]
fn nothing_rolled_breaks_its_limits() {
    for size in Size::ALL {
        for spare in [300, 800, 2000, 10_000, 100_000] {
            let limits = limits_for_wielder(size, spare);
            for family in Family::ALL {
                for seed in 0..100 {
                    if let Some(got) = asked_for(seed, family, &limits) {
                        let shape = got.form.shape().scaled(got.size);
                        let grams = got.form.grams_at(got.size);
                        assert!(limits.allows(shape, grams), "{got:?} in {limits:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn a_tiny_creature_almost_always_gets_an_ordinary_small_weapon() {
    let limits = limits_for_wielder(Size::Tiny, 5000);
    let got: Vec<_> = (0..1000)
        .filter_map(|seed| asked_for(seed, Family::Sword, &limits))
        .collect();
    assert_eq!(got.len(), 1000);
    let ordinary: Vec<_> = got.iter().filter(|got| got.size == Size::Medium).collect();
    assert!((850..950).contains(&ordinary.len()), "{}", ordinary.len());
    for got in ordinary {
        assert!(!got.form.generic_only);
        assert!(matches!(
            got.form.family,
            Family::Knife | Family::Dagger | Family::ShortSword
        ));
    }
}

#[test]
fn a_giant_may_get_a_giant_sword_named_only_generically() {
    let limits = limits_for_wielder(Size::Giant, 500_000);
    let giants: Vec<_> = (0..500)
        .filter_map(|seed| asked_for(seed, Family::Sword, &limits))
        .filter(|got| got.size == Size::Giant)
        .collect();
    assert!(!giants.is_empty());
    for got in giants {
        assert!(got.form.generic_only);
        assert_eq!(got.form.family, Family::Sword);
        assert_eq!(got.form.name(), "Sword");
    }
}

#[test]
fn nothing_comes_back_when_nothing_fits() {
    let limits = Limits {
        most_grams: Some(1),
        ..Limits::NONE
    };
    assert_eq!(asked_for(1, Family::Sword, &limits), None);
}

#[test]
fn a_commission_keeps_its_name_at_any_size() {
    let order = |name| {
        let mut rng = Seed::new(1).rng();
        commission_blade(&mut rng, name, Quality::Rare, Size::Tiny, &Limits::NONE)
    };
    let tiny = order("Katana").unwrap();
    assert_eq!((tiny.size, tiny.form.name()), (Size::Tiny, "Katana"));
    assert!(order("Lightsaber").is_none());
}

#[test]
fn a_weapon_made_at_another_size_has_only_its_generic_name() {
    for seed in 0..200 {
        let mut rng = Seed::new(seed).rng();
        let form = roll_blade_at(&mut rng, Family::Sword, Quality::Common, Size::Giant);
        assert_eq!(form.name(), "Sword");
    }
}

#[test]
fn better_work_is_more_often_of_finer_metal() {
    let gold = |quality| {
        (0..4000)
            .map(|seed| roll_blade(&mut Seed::new(seed).rng(), Family::Sword, quality))
            .filter(|form| form.blade_metal == Material::Gold)
            .count()
    };
    assert_eq!(gold(Quality::Poor), 0);
    assert!(gold(Quality::Legendary) > 50);
}

#[test]
fn a_form_survives_being_saved() {
    let form = roll_blade(&mut Seed::new(3).rng(), Family::Polearm, Quality::Common);
    let bytes = postcard::to_allocvec(&form).unwrap();
    assert_eq!(postcard::from_bytes::<BladeForm>(&bytes).unwrap(), form);
}
