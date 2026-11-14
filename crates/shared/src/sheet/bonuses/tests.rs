use super::*;
use crate::bonus::{APPLE_TICKS, Effects};
use crate::creature::CreatureKind;
use crate::id::Id;
use crate::item::Quality;
use crate::item_form::{ItemOrder, make_item};
use crate::rng::Seed;
use crate::vitals::{Change, Points, REFILL_TICKS, Vital};

const SECOND: u64 = crate::vitals::TICKS;

fn sheet() -> Sheet {
    let mut sheet = Sheet::new(CreatureKind::Human);
    sheet.attributes = Attributes::all(100);
    sheet
}

fn situation(worn: &[Item]) -> Situation<'_> {
    Situation {
        worn,
        playtime: 0,
        refill: REFILL_TICKS,
    }
}

fn armour() -> Item {
    make_item(
        &ItemOrder::plain(ItemKind::TorsoArmour, Quality::Common),
        CreatureKind::Human,
        Id::for_test(1),
        Seed::new(3),
        None,
        &mut || Id::for_test(2),
    )
}

fn health(sheet: &Sheet, worn: &[Item], now: u64) -> Points {
    let limits = sheet.limits(situation(worn), now);
    sheet.vitals.at(Vital::Health, limits, now)
}

#[test]
fn armour_raises_toughness_and_health_and_lowers_agility_and_mana() {
    let sheet = sheet();
    let worn = [armour()];
    let kg = i64::from(worn[0].own_weight()) / 1000;
    let with = sheet.attributes_with(&worn, 0);
    assert!(i64::from(with.toughness) >= 100 + 10 * kg);
    assert!(i64::from(with.agility) <= 100 - 2 * kg);
    let bare = sheet.limits(situation(&[]), 0);
    let dressed = sheet.limits(situation(&worn), 0);
    assert!(dressed.health > bare.health);
    assert!(dressed.mana < bare.mana);
}

#[test]
fn putting_armour_on_and_off_keeps_the_share() {
    let mut sheet = sheet();
    let worn = [armour()];
    let bare = sheet.limits(situation(&[]), 0);
    sheet.vitals.hurt(Points::whole(10), bare, 0, 0);
    sheet.rebase(bare, situation(&worn), 0);
    let dressed = sheet.limits(situation(&worn), 0);
    let share = |sheet: &Sheet, limits: Limits| {
        sheet.vitals.at(Vital::Health, limits, 0).milli() as u64 * 1000 / u64::from(limits.health)
    };
    assert!((499_000..=500_000).contains(&share(&sheet, dressed)));
    sheet.rebase(dressed, situation(&[]), 0);
    assert!(health(&sheet, &[], 0) <= Points::whole(10));
}

#[test]
fn an_apple_restores_health_until_it_ends_exactly() {
    let mut sheet = sheet();
    let bare = sheet.limits(situation(&[]), 0);
    sheet.vitals.hurt(Points::whole(15), bare, 0, 0);
    let eaten = 1000;
    assert!(sheet.eat(ItemKind::Apple, situation(&[]), eaten));
    assert!(!sheet.eat(ItemKind::Stone, situation(&[]), eaten));
    let end = eaten + APPLE_TICKS;
    // Just before it ends, still restoring.
    let lasting = sheet.limits(situation(&[]), end - 1);
    assert_eq!(lasting.restore.health, 1000);
    assert_eq!(lasting.restore.until, Some(end));
    let at_end = sheet.vitals.at(Vital::Health, lasting, end);
    // Forgotten long after, it still stopped at its end.
    let later = end + 10 * SECOND;
    assert!(sheet.forget_ended(situation(&[]), later));
    assert!(sheet.effects.is_empty());
    let plain = sheet.limits(situation(&[]), later);
    let refilled = crate::vitals::refilled(at_end, plain.health, REFILL_TICKS, 10 * SECOND);
    assert_eq!(health(&sheet, &[], later), refilled);
    // 30 seconds of 1 a second, and refilling: full again.
    assert_eq!(at_end, Points::whole(20));
}

#[test]
fn moving_ends_an_apple_and_keeps_what_it_gave() {
    let mut sheet = sheet();
    let bare = sheet.limits(situation(&[]), 0);
    sheet.vitals.hurt(Points::whole(15), bare, 0, 0);
    sheet.eat(ItemKind::Apple, situation(&[]), 1000);
    let moved = 1000 + 3 * SECOND;
    let before = health(&sheet, &[], moved);
    assert!(sheet.cancel_effects(Cancel::Moving, situation(&[]), moved));
    assert!(sheet.effects.is_empty());
    assert_eq!(health(&sheet, &[], moved), before);
    assert!(before >= Points::whole(8));
    assert!(!sheet.cancel_effects(Cancel::Hurt, situation(&[]), moved));
}

#[test]
fn a_dev_bonus_raises_both_and_ends_keeping_the_share() {
    let mut sheet = sheet();
    sheet.add_effect(Effect::dev_bonus(5, 0, 100), situation(&[]), 0);
    let raised = sheet.limits(situation(&[]), 0);
    assert_eq!((raised.health, raised.mana), (25, 25));
    assert_eq!(health(&sheet, &[], 0), Points::whole(25));
    assert!(!sheet.forget_ended(situation(&[]), 99));
    assert!(sheet.forget_ended(situation(&[]), 100));
    assert_eq!(health(&sheet, &[], 100), Points::whole(20));
}

#[test]
fn older_saves_changes_become_effects() {
    let mut sheet = sheet();
    sheet.vitals.changes = vec![Change {
        vital: Vital::Mana,
        amount: 3,
        until: None,
    }];
    let sheet = sheet.settled(CreatureKind::Human);
    assert!(sheet.vitals.changes.is_empty());
    assert_eq!(sheet.effects.0.len(), 1);
    assert_eq!(sheet.limits(situation(&[]), 0).mana, 23);
}

#[test]
fn a_sheet_without_effects_loads_with_none() {
    let sheet = sheet();
    let text = ron::to_string(&sheet).unwrap();
    let without = text.replace(",effects:([])", "");
    assert_ne!(text, without, "{text}");
    let loaded: Sheet = ron::from_str(&without).unwrap();
    assert_eq!(loaded.effects, Effects::default());
}
