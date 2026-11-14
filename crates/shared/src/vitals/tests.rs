use super::*;
use crate::bonus::Totals;

fn w(points: u32) -> Points {
    Points::whole(points)
}

fn rate(most: u32) -> Rate {
    Rate {
        most,
        refill: REFILL_TICKS,
        ..Rate::default()
    }
}

fn median() -> Attributes {
    Attributes::all(100)
}

fn limits(vitals: &Vitals, playtime: u64) -> Limits {
    vitals.limits(&median(), playtime)
}

#[test]
fn a_median_creature_has_20_of_each_and_starts_full() {
    let vitals = Vitals::default();
    let limits = limits(&vitals, 0);
    assert_eq!(
        limits,
        Limits {
            health: 20,
            mana: 20,
            refill: REFILL_TICKS,
            ..Default::default()
        }
    );
    assert_eq!(vitals.at(Vital::Health, limits, 0), w(20));
    assert_eq!(vitals.at(Vital::Mana, limits, 5), w(20));
}

#[test]
fn the_most_comes_from_its_own_attribute() {
    let mut attributes = median();
    attributes.toughness = 150;
    attributes.will = 60;
    let limits = Vitals::default().limits(&attributes, 0);
    assert_eq!(
        limits,
        Limits {
            health: 30,
            mana: 12,
            refill: REFILL_TICKS,
            ..Default::default()
        }
    );
}

#[test]
fn a_day_of_playtime_adds_a_point_to_both() {
    let vitals = Vitals::default();
    assert_eq!(limits(&vitals, PLAYTIME_PER_POINT - 1).health, 20);
    assert_eq!(
        limits(&vitals, PLAYTIME_PER_POINT * 3),
        Limits {
            health: 23,
            mana: 23,
            refill: REFILL_TICKS,
            ..Default::default()
        }
    );
}

#[test]
fn harm_waits_then_refills_in_five_minutes_from_empty() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    vitals.hurt(w(20), most, 1000, 0);
    assert_eq!(vitals.at(Vital::Health, most, 1000), w(0));
    assert_eq!(vitals.at(Vital::Health, most, 1000 + CALM_AFTER), w(0));
    let start = 1000 + CALM_AFTER;
    assert_eq!(
        vitals.at(Vital::Health, most, start + REFILL_TICKS / 2),
        w(10)
    );
    assert_eq!(
        vitals.at(Vital::Health, most, start + REFILL_TICKS - 1),
        Points(19_996)
    );
    assert_eq!(vitals.at(Vital::Health, most, start + REFILL_TICKS), w(20));
    assert_eq!(
        vitals.at(Vital::Health, most, start + REFILL_TICKS * 9),
        w(20)
    );
}

#[test]
fn more_harm_puts_refilling_off_again() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    vitals.hurt(w(10), most, 0, 0);
    let later = CALM_AFTER + REFILL_TICKS / 4;
    assert_eq!(vitals.at(Vital::Health, most, later), w(15));
    vitals.hurt(w(5), most, later, 0);
    assert_eq!(vitals.at(Vital::Health, most, later + CALM_AFTER), w(10));
    // Mana never waits (owner, 2026-11-07).
    assert_eq!(vitals.told(Vital::Mana, most, later).refills_in, 0);
}

#[test]
fn mana_refills_straight_away_after_casting() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    assert!(vitals.spend_mana(w(10), most, 1000, 0));
    assert_eq!(vitals.told(Vital::Mana, most, 1000).refills_in, 0);
    assert_eq!(vitals.at(Vital::Mana, most, 1000 + REFILL_TICKS / 4), w(15));
    // Casting isn't fighting: Health doesn't wait either.
    assert_eq!(vitals.told(Vital::Health, most, 1000).refills_in, 0);
}

#[test]
fn restoring_stops_at_the_most_and_keeps_the_wait() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    vitals.hurt(w(15), most, 0, 0);
    vitals.restore(Vital::Health, w(6), most, 10);
    assert_eq!(vitals.at(Vital::Health, most, 10), w(11));
    assert_eq!(
        vitals.told(Vital::Health, most, 10).refills_in as u64,
        CALM_AFTER - 10
    );
    vitals.restore(Vital::Health, w(120), most, 20);
    assert_eq!(vitals.at(Vital::Health, most, 20), w(20));
    // Full is kept as full, so a higher most is full too.
    let higher = Limits { health: 30, ..most };
    assert_eq!(vitals.at(Vital::Health, higher, 20), w(30));
}

#[test]
fn health_stops_at_nothing() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    vitals.hurt(w(500), most, 0, 0);
    assert_eq!(vitals.at(Vital::Health, most, 0), w(0));
}

#[test]
fn mana_can_only_be_spent_if_there_is_enough() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    assert!(vitals.spend_mana(w(15), most, 0, 0));
    assert!(!vitals.spend_mana(w(6), most, 0, 0));
    assert_eq!(vitals.at(Vital::Mana, most, 0), w(5));
    assert_eq!(vitals.at(Vital::Health, most, 0), w(20));
}

#[test]
fn practice_raises_the_most_but_only_so_much_an_hour() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    vitals.hurt(w(500), most, 0, 0);
    assert_eq!(vitals.health.practice, w(PRACTICE_PER_HOUR).milli());
    vitals.hurt(w(500), most, 0, 10);
    assert_eq!(vitals.health.practice, w(PRACTICE_PER_HOUR).milli());
    // The next hour of playtime has room again.
    for hour in 1..=4 {
        vitals.hurt(w(500), most, 0, hour * PLAYTIME_HOUR);
    }
    assert_eq!(vitals.health.practice, w(100).milli());
    let parts = vitals.most(Vital::Health, &median(), &Totals::default(), 0);
    assert_eq!(parts.practice, 2);
    assert_eq!(
        limits(&vitals, 0),
        Limits {
            health: 22,
            mana: 20,
            refill: REFILL_TICKS,
            ..Default::default()
        }
    );
}

#[test]
fn a_client_refills_what_it_is_told_the_same_way() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    vitals.hurt(w(12), most, 300, 0);
    let told = vitals.told(Vital::Health, most, 300);
    for later in [
        0,
        10,
        CALM_AFTER,
        CALM_AFTER + 777,
        CALM_AFTER + REFILL_TICKS,
    ] {
        assert_eq!(
            told.after(later),
            vitals.at(Vital::Health, most, 300 + later)
        );
    }
}

#[test]
fn saves_from_before_load_full() {
    let vitals: Vitals = serde_json::from_str("{}").unwrap();
    assert_eq!(vitals, Vitals::default());
    assert_eq!(
        vitals.at(
            Vital::Health,
            Limits {
                health: 7,
                mana: 3,
                refill: REFILL_TICKS,
                ..Default::default()
            },
            0
        ),
        w(7)
    );
}

#[test]
fn harm_keeps_its_fractions() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    for _ in 0..3 {
        vitals.hurt(Points(333), most, 0, 0);
    }
    assert_eq!(vitals.at(Vital::Health, most, 0), Points(19_001));
    assert_eq!(vitals.at(Vital::Health, most, 0).shown(), 20);
}

#[test]
fn saves_in_whole_points_turn_into_thousandths() {
    let old = r#"{"value":12,"since":5,"practice":40,"this_hour":3}"#;
    let track: Track = serde_json::from_str(old).unwrap();
    assert_eq!(track.at(rate(20), 5), w(12));
    assert_eq!(track.practice, w(40).milli());
    let full: Track =
        serde_json::from_str(r#"{"value":4294967295,"since":0,"practice":0,"this_hour":0}"#)
            .unwrap();
    assert_eq!(full, Track::default());
    let again: Track = serde_json::from_str(&serde_json::to_string(&track).unwrap()).unwrap();
    assert_eq!(again, track);
}

#[test]
fn a_faster_refill_fills_sooner_and_the_client_agrees() {
    let mut vitals = Vitals::default();
    let usual = limits(&vitals, 0);
    let quick = Limits {
        refill: REFILL_TICKS / 2,
        ..usual
    };
    vitals.hurt(w(20), quick, 0, 0);
    let start = CALM_AFTER;
    assert_eq!(
        vitals.at(Vital::Health, quick, start + REFILL_TICKS / 4),
        w(10)
    );
    assert_eq!(
        vitals.at(Vital::Health, quick, start + REFILL_TICKS / 2),
        w(20)
    );
    assert_eq!(
        vitals.at(Vital::Health, usual, start + REFILL_TICKS / 4),
        w(5)
    );
    let told = vitals.told(Vital::Health, quick, 0);
    assert_eq!(
        told.after(start + REFILL_TICKS / 4),
        vitals.at(Vital::Health, quick, start + REFILL_TICKS / 4)
    );
}

#[test]
fn fighting_lasts_calm_after_the_last_blow_but_not_after_casting() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0);
    assert!(!vitals.fighting(0));
    assert!(vitals.spend_mana(w(5), most, 100, 0));
    assert!(!vitals.fighting(100));
    vitals.in_combat(most, 1000);
    assert!(vitals.fighting(1000 + CALM_AFTER - 1));
    assert!(!vitals.fighting(1000 + CALM_AFTER));
    vitals.hurt(w(1), most, 2000, 0);
    assert!(vitals.fighting(2000));
}
