use super::*;
use crate::creature::CreatureKind;
use crate::id::Id;
use crate::item::{Item, ItemKind, Quality};
use crate::item_form::{ItemForm, ItemOrder, make_item};
use crate::material::Material;
use crate::rng::Seed;
use crate::sheet::{Attribute, Attributes};
use crate::underclothes::{UnderForm, UnderPiece};
use crate::vitals::Vital;

const TOUGHNESS: Stat = Stat::Attribute(Attribute::Toughness);
const AGILITY: Stat = Stat::Attribute(Attribute::Agility);

fn armour(kind: ItemKind, quality: Quality) -> Item {
    make_item(
        &ItemOrder::plain(kind, quality),
        CreatureKind::Human,
        Id::for_test(1),
        Seed::new(3),
        None,
        &mut || Id::for_test(2),
    )
}

fn grams(item: &Item) -> i64 {
    i64::from(item.own_weight())
}

#[test]
fn flat_ones_add_up_then_percentages_of_the_base_rounded_once() {
    let bonuses = [
        Bonus::flat(TOUGHNESS, 400),
        Bonus::flat(TOUGHNESS, 400),
        Bonus::percent(TOUGHNESS, 10 * PER_CENT),
    ];
    let totals = Totals::of(&bonuses);
    assert_eq!(
        totals.sum(TOUGHNESS),
        Sum {
            flat: 800,
            percent: 10_000
        }
    );
    // 100 + 0.8 + 10 = 110.8, rounded once: 111. Rounding each 0.4 would
    // have lost both.
    assert_eq!(totals.whole(TOUGHNESS, 100), 111);
    assert_eq!(totals.whole(AGILITY, 100), 100);
}

#[test]
fn attributes_never_go_below_one() {
    let totals = Totals::of(&[Bonus::flat(AGILITY, -500_000)]);
    let with = totals.attributes(&Attributes::all(100));
    assert_eq!(with.agility, 1);
    assert_eq!(with.toughness, 100);
}

#[test]
fn rounding_goes_to_the_nearest() {
    assert_eq!(round_milli(1_499), 1);
    assert_eq!(round_milli(1_500), 2);
    assert_eq!(round_milli(-1_499), -1);
    assert_eq!(round_milli(-2_500), -2);
}

#[test]
fn armour_gives_toughness_and_costs_agility_and_mana_by_the_kilogram() {
    let piece = armour(ItemKind::TorsoArmour, Quality::Common);
    let g = grams(&piece);
    assert!(g > 1000, "a torso piece weighs over a kilogram");
    let totals = Totals::of(&item_bonuses(&piece));
    assert_eq!(totals.sum(TOUGHNESS).flat, 10 * g);
    assert_eq!(totals.sum(AGILITY).flat, -2 * g);
    assert_eq!(totals.sum(Stat::Most(Vital::Mana)).flat, -g);
}

#[test]
fn better_armour_gives_more_toughness_but_costs_the_same() {
    for (quality, factor) in [
        (Quality::Poor, 750),
        (Quality::Common, 1000),
        (Quality::Uncommon, 1250),
        (Quality::Legendary, 2000),
        (Quality::Artifact, 2500),
    ] {
        assert_eq!(quality_factor(Some(quality)), factor);
        let piece = armour(ItemKind::TorsoArmour, quality);
        let g = grams(&piece);
        let totals = Totals::of(&item_bonuses(&piece));
        assert_eq!(totals.sum(TOUGHNESS).flat, 10 * g * factor / 1000);
        assert_eq!(totals.sum(AGILITY).flat, -2 * g);
    }
}

#[test]
fn a_hat_or_an_apple_gives_nothing() {
    let hat = Item::new(Id::for_test(1), ItemKind::Hat);
    assert!(item_bonuses(&hat).is_empty());
    let apple = Item::new(Id::for_test(1), ItemKind::Apple);
    assert!(item_bonuses(&apple).is_empty());
}

/// The first of many rolled pieces of underclothing `wanted` accepts.
fn under(piece: UnderPiece, wanted: impl Fn(&UnderForm) -> bool) -> Item {
    (0..5000u64)
        .map(|seed| {
            make_item(
                &ItemOrder::plain(piece.kind(), Quality::Common),
                CreatureKind::Human,
                Id::for_test(1),
                Seed::new(seed),
                None,
                &mut || Id::for_test(2),
            )
        })
        .find(|item| matches!(&item.form, Some(ItemForm::Under(form)) if wanted(form)))
        .expect("rolled")
}

#[test]
fn a_hair_shirt_costs_five_percent_will() {
    let hair = |form: &UnderForm| form.material == Material::Haircloth;
    let shirt = under(UnderPiece::Shirt, hair);
    assert_eq!(
        item_bonuses(&shirt),
        vec![Bonus::percent(
            Stat::Attribute(Attribute::Will),
            -5 * PER_CENT
        )]
    );
    let linen = under(UnderPiece::Shirt, |form| !hair(form));
    assert!(item_bonuses(&linen).is_empty());
    // A horsehair petticoat is haircloth, but no hair shirt.
    let petticoat = under(UnderPiece::Petticoat, hair);
    assert!(item_bonuses(&petticoat).is_empty());
}

#[test]
fn every_material_is_as_usual_for_now() {
    for material in crate::material::Material::ALL {
        assert_eq!(material_factors(material), MaterialFactors::USUAL);
    }
}

#[test]
fn an_apple_restores_health_for_thirty_seconds_until_moving_or_hurt() {
    let effect = food_effect(ItemKind::Apple, 100).expect("apples do something");
    assert_eq!(effect.end, Some(100 + APPLE_TICKS));
    assert!(effect.lasts(100 + APPLE_TICKS - 1));
    assert!(!effect.lasts(100 + APPLE_TICKS));
    assert!(effect.ends_on.by(Cancel::Moving) && effect.ends_on.by(Cancel::Hurt));
    let totals = Totals::of(&effect.bonuses);
    assert_eq!(totals.restoring(Vital::Health), 1000);
    assert_eq!(totals.restoring(Vital::Mana), 0);
    assert!(food_effect(ItemKind::Stone, 0).is_none());
}

#[test]
fn a_second_from_the_same_source_takes_the_firsts_place() {
    let mut effects = Effects::default();
    effects.add(food_effect(ItemKind::Apple, 0).unwrap());
    effects.add(Effect::dev_bonus(5, 0, 50));
    effects.add(food_effect(ItemKind::Apple, 40).unwrap());
    assert_eq!(effects.0.len(), 2);
    let apple = effects
        .0
        .iter()
        .find(|e| e.source == Source::Eaten(ItemKind::Apple));
    assert_eq!(apple.unwrap().start, 40);
}

#[test]
fn effects_end_by_time_or_by_what_cancels_them() {
    let mut effects = Effects::default();
    effects.add(food_effect(ItemKind::Apple, 0).unwrap());
    effects.add(Effect::dev_bonus(5, 0, 50));
    assert_eq!(effects.next_end(0), Some(50));
    assert_eq!(effects.first_ended(49), None);
    assert_eq!(effects.first_ended(60), Some(50));
    assert_eq!(effects.bonuses(60).count(), 1);
    assert!(effects.cancelled_by(Cancel::Hurt));
    assert!(effects.cancel(Cancel::Hurt));
    assert!(!effects.cancel(Cancel::Moving));
    assert_eq!(effects.0.len(), 1);
    effects.forget_ended(50);
    assert!(effects.is_empty());
}
