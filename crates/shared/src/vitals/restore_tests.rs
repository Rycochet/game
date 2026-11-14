//! Restoring over time and keeping shares when the most changes.

use super::*;

fn w(points: u32) -> Points {
    Points::whole(points)
}

fn with(health: u32, mana: u32, restore: Restore) -> Limits {
    Limits {
        health,
        mana,
        refill: REFILL_TICKS,
        restore,
    }
}

const SECOND: u64 = TICKS;

#[test]
fn restoring_adds_to_refilling_until_it_stops() {
    let mut vitals = Vitals::default();
    let plain = with(100, 20, Restore::default());
    vitals.hurt(w(60), plain, 0, 0);
    let start = CALM_AFTER;
    let restoring = with(
        100,
        20,
        Restore {
            health: 1000,
            mana: 0,
            until: Some(start + 10 * SECOND),
        },
    );
    // 10 seconds of refilling (100 over 5 minutes: 1/3 a second) plus 1 a second.
    let refilled = refilled(w(40), 100, REFILL_TICKS, 10 * SECOND);
    assert_eq!(
        vitals.at(Vital::Health, restoring, start + 10 * SECOND),
        refilled.plus(w(10))
    );
    // It stops at its end, while refilling goes on.
    let later = start + 20 * SECOND;
    let refilled = refilled_at(40, later - start);
    assert_eq!(
        vitals.at(Vital::Health, restoring, later),
        refilled.plus(w(10))
    );
    // Not while waiting to refill.
    assert_eq!(vitals.at(Vital::Health, restoring, start - 1), w(40));
}

fn refilled_at(from: u32, ticks: u64) -> Points {
    refilled(w(from), 100, REFILL_TICKS, ticks)
}

#[test]
fn a_client_restores_what_it_is_told_the_same_way() {
    let mut vitals = Vitals::default();
    let limits = with(
        50,
        20,
        Restore {
            health: 1000,
            mana: 500,
            until: Some(900),
        },
    );
    vitals.hurt(w(30), limits, 300, 0);
    vitals.spend_mana(w(15), limits, 300, 0);
    for vital in Vital::ALL {
        let told = vitals.told(vital, limits, 300);
        for later in [0, 10, CALM_AFTER, CALM_AFTER + 77, 600, 601, 5_000] {
            assert_eq!(told.after(later), vitals.at(vital, limits, 300 + later));
        }
    }
}

#[test]
fn never_past_the_most() {
    let vitals = Vitals::default();
    let limits = with(
        20,
        20,
        Restore {
            health: 50_000,
            mana: 0,
            until: None,
        },
    );
    assert_eq!(vitals.at(Vital::Health, limits, 10_000), w(20));
}

#[test]
fn bonuses_keep_the_share_rounding_down() {
    let mut vitals = Vitals::default();
    let before = with(100, 100, Restore::default());
    vitals.hurt(w(50), before, 0, 0);
    let after = with(120, 100, Restore::default());
    vitals.rebase(before, after, 0);
    assert_eq!(vitals.at(Vital::Health, after, 0), w(60));
    vitals.rebase(after, before, 0);
    assert_eq!(vitals.at(Vital::Health, before, 0), w(50));
    // A third of 100 to 120 and back never comes out ahead.
    let mut vitals = Vitals::default();
    vitals.hurt(Points(66_667), before, 0, 0);
    let third = vitals.at(Vital::Health, before, 0);
    vitals.rebase(before, after, 0);
    vitals.rebase(after, before, 0);
    assert!(vitals.at(Vital::Health, before, 0) <= third);
}

#[test]
fn full_stays_full_when_bonuses_change() {
    let mut vitals = Vitals::default();
    let before = with(100, 100, Restore::default());
    let after = with(120, 80, Restore::default());
    vitals.rebase(before, after, 0);
    assert_eq!(vitals.at(Vital::Health, after, 0), w(120));
    assert_eq!(vitals.at(Vital::Mana, after, 0), w(80));
}

#[test]
fn mana_can_go_to_nothing_and_come_back_as_it_was() {
    let mut vitals = Vitals::default();
    let free = with(20, 20, Restore::default());
    vitals.spend_mana(w(10), free, 0, 0);
    let armoured = with(20, 0, Restore::default());
    vitals.rebase(free, armoured, 0);
    assert_eq!(vitals.at(Vital::Mana, armoured, 0), w(0));
    assert!(!vitals.spend_mana(w(1), armoured, 0, 0));
    vitals.settle(armoured, 5_000);
    vitals.rebase(armoured, free, 5_000);
    assert_eq!(vitals.at(Vital::Mana, free, 5_000), w(10));
}

#[test]
fn the_most_floors_are_one_health_and_no_mana() {
    let parts = |vital| MostParts {
        vital,
        attribute: 5,
        playtime: 0,
        practice: 0,
        bonus: -50,
    };
    assert_eq!(parts(Vital::Health).total(), 1);
    assert_eq!(parts(Vital::Mana).total(), 0);
}

#[test]
fn older_saves_changes_still_load() {
    let old = r#"{"changes":[{"vital":"Health","amount":5,"until":100}]}"#;
    let vitals: Vitals = serde_json::from_str(old).unwrap();
    assert_eq!(vitals.changes.len(), 1);
    assert_eq!(vitals.changes[0].amount, 5);
}
