use super::*;

fn median() -> Attributes {
    Attributes::all(100)
}

fn limits(vitals: &Vitals, playtime: u64, now: u64) -> Limits {
    vitals.limits(&median(), playtime, now)
}

#[test]
fn a_median_creature_has_20_of_each_and_starts_full() {
    let vitals = Vitals::default();
    let limits = limits(&vitals, 0, 0);
    assert_eq!(
        limits,
        Limits {
            health: 20,
            mana: 20
        }
    );
    assert_eq!(vitals.at(Vital::Health, limits, 0), 20);
    assert_eq!(vitals.at(Vital::Mana, limits, 5), 20);
}

#[test]
fn the_most_comes_from_its_own_attribute() {
    let mut attributes = median();
    attributes.toughness = 150;
    attributes.will = 60;
    let limits = Vitals::default().limits(&attributes, 0, 0);
    assert_eq!(
        limits,
        Limits {
            health: 30,
            mana: 12
        }
    );
}

#[test]
fn a_day_of_playtime_adds_a_point_to_both() {
    let vitals = Vitals::default();
    assert_eq!(limits(&vitals, PLAYTIME_PER_POINT - 1, 0).health, 20);
    assert_eq!(
        limits(&vitals, PLAYTIME_PER_POINT * 3, 0),
        Limits {
            health: 23,
            mana: 23
        }
    );
}

#[test]
fn harm_waits_then_refills_in_five_minutes_from_empty() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0, 0);
    vitals.hurt(20, most, 1000, 0);
    assert_eq!(vitals.at(Vital::Health, most, 1000), 0);
    assert_eq!(vitals.at(Vital::Health, most, 1000 + CALM_AFTER), 0);
    let start = 1000 + CALM_AFTER;
    assert_eq!(vitals.at(Vital::Health, most, start + REFILL_TICKS / 2), 10);
    assert_eq!(vitals.at(Vital::Health, most, start + REFILL_TICKS - 1), 19);
    assert_eq!(vitals.at(Vital::Health, most, start + REFILL_TICKS), 20);
    assert_eq!(vitals.at(Vital::Health, most, start + REFILL_TICKS * 9), 20);
}

#[test]
fn more_harm_puts_refilling_off_again() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0, 0);
    vitals.hurt(10, most, 0, 0);
    let later = CALM_AFTER + REFILL_TICKS / 4;
    assert_eq!(vitals.at(Vital::Health, most, later), 15);
    vitals.hurt(5, most, later, 0);
    assert_eq!(vitals.at(Vital::Health, most, later + CALM_AFTER), 10);
    // Mana waits too, though none was spent.
    assert_eq!(
        vitals.told(Vital::Mana, most, later).refills_in as u64,
        CALM_AFTER
    );
}

#[test]
fn health_stops_at_nothing() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0, 0);
    vitals.hurt(500, most, 0, 0);
    assert_eq!(vitals.at(Vital::Health, most, 0), 0);
}

#[test]
fn mana_can_only_be_spent_if_there_is_enough() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0, 0);
    assert!(vitals.spend_mana(15, most, 0, 0));
    assert!(!vitals.spend_mana(6, most, 0, 0));
    assert_eq!(vitals.at(Vital::Mana, most, 0), 5);
    assert_eq!(vitals.at(Vital::Health, most, 0), 20);
}

#[test]
fn practice_raises_the_most_but_only_so_much_an_hour() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0, 0);
    vitals.hurt(500, most, 0, 0);
    assert_eq!(vitals.health.practice, PRACTICE_PER_HOUR);
    vitals.hurt(500, most, 0, 10);
    assert_eq!(vitals.health.practice, PRACTICE_PER_HOUR);
    // The next hour of playtime has room again.
    for hour in 1..=4 {
        vitals.hurt(500, most, 0, hour * PLAYTIME_HOUR);
    }
    assert_eq!(vitals.health.practice, 100);
    let parts = vitals.most(Vital::Health, &median(), 0, 0);
    assert_eq!(parts.practice, 2);
    assert_eq!(
        limits(&vitals, 0, 0),
        Limits {
            health: 22,
            mana: 20
        }
    );
}

#[test]
fn changes_last_until_they_end() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0, 0);
    let bonus = Change {
        vital: Vital::Health,
        amount: 5,
        until: Some(100),
    };
    vitals.add_change(bonus, most, 0);
    let raised = limits(&vitals, 0, 50);
    assert_eq!(raised.health, 25);
    // Full stays full when the most rises.
    assert_eq!(vitals.at(Vital::Health, raised, 50), 25);
    assert!(!vitals.forget_ended(raised, 99));
    assert!(vitals.forget_ended(raised, 100));
    assert!(vitals.changes.is_empty());
    let after = limits(&vitals, 0, 100);
    assert_eq!(vitals.at(Vital::Health, after, 100), 20);
}

#[test]
fn a_change_can_never_take_the_most_below_one() {
    let mut vitals = Vitals::default();
    let curse = Change {
        vital: Vital::Mana,
        amount: -500,
        until: None,
    };
    vitals.add_change(curse, limits(&vitals, 0, 0), 0);
    assert_eq!(limits(&vitals, 0, 0).mana, 1);
}

#[test]
fn a_client_refills_what_it_is_told_the_same_way() {
    let mut vitals = Vitals::default();
    let most = limits(&vitals, 0, 0);
    vitals.hurt(12, most, 300, 0);
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
        vitals.at(Vital::Health, Limits { health: 7, mana: 3 }, 0),
        7
    );
}
