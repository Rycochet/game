use super::*;

fn who(n: u128) -> CharacterId {
    CharacterId(crate::id::Id::for_test(n))
}

#[test]
fn a_friendly_mob_takes_two_hits_to_turn() {
    let mut memory = Memory::default();
    let a = who(1);
    memory.remember_harm(a, Points::whole(3), Feeling::Friendly);
    // Cooled, not angry: an accident is forgiven this far.
    assert_eq!(memory.standing(&a, Feeling::Friendly), Feeling::Neutral);
    assert_eq!(memory.opinion(&a), -10_000);
    memory.remember_harm(a, Points::whole(3), Feeling::Friendly);
    assert_eq!(memory.standing(&a, Feeling::Friendly), Feeling::Unfriendly);
    assert_eq!(memory.opinion(&a), -13_000);
    // Nobody else is any worse off.
    assert_eq!(
        memory.standing(&who(2), Feeling::Friendly),
        Feeling::Friendly
    );
    assert_eq!(memory.opinion(&who(2)), 0);
}

#[test]
fn a_neutral_mob_turns_at_once_and_keeps_getting_worse() {
    let mut memory = Memory::default();
    let a = who(1);
    memory.remember_gift(a);
    // A heavy first blow counts in full, and wipes out the goodwill.
    memory.remember_harm(a, Points::whole(15), Feeling::Neutral);
    assert_eq!(memory.standing(&a, Feeling::Neutral), Feeling::Unfriendly);
    assert_eq!(memory.opinion(&a), -15_000);
    // Later blows count only what they did, even nothing.
    memory.remember_harm(a, Points(500), Feeling::Neutral);
    memory.remember_harm(a, Points::ZERO, Feeling::Neutral);
    assert_eq!(memory.opinion(&a), -15_500);
}

#[test]
fn a_hostile_mob_thinks_still_less_of_whoever_hits_it() {
    let mut memory = Memory::default();
    memory.remember_harm(who(1), Points::whole(1), Feeling::Unfriendly);
    assert_eq!(
        memory.standing(&who(1), Feeling::Unfriendly),
        Feeling::Unfriendly
    );
    assert_eq!(memory.opinion(&who(1)), -10_000);
}

#[test]
fn gifts_raise_the_opinion_but_not_the_standing() {
    let mut memory = Memory::default();
    for _ in 0..3 {
        memory.remember_gift(who(1));
    }
    assert_eq!(memory.opinion(&who(1)), 3 * GIFT_OPINION);
    assert_eq!(
        memory.standing(&who(1), Feeling::Unfriendly),
        Feeling::Unfriendly
    );
    // Goodwill doesn't save anyone from the two-hit rule.
    memory.remember_harm(who(1), Points::whole(1), Feeling::Friendly);
    memory.remember_harm(who(1), Points::whole(1), Feeling::Friendly);
    assert_eq!(
        memory.standing(&who(1), Feeling::Friendly),
        Feeling::Unfriendly
    );
}

#[test]
fn gifts_from_older_saves_count_into_the_opinion() {
    let a = who(1);
    let old = Memory(vec![
        (a, Event::Gave(ItemKind::Coin)),
        (a, Event::Gave(ItemKind::Coin)),
    ]);
    let text = ron::to_string(&old).unwrap();
    let mut memory: Memory = ron::from_str(&text).unwrap();
    assert_eq!(memory.opinion(&a), 2 * GIFT_OPINION);
    assert_eq!(memory.standing(&a, Feeling::Neutral), Feeling::Neutral);
    memory.remember_gift(a);
    assert_eq!(memory.opinion(&a), 3 * GIFT_OPINION);
    assert_eq!(memory.0.len(), 1);
}

#[test]
fn a_trespass_angers_once_and_is_forgotten() {
    let a = who(1);
    let mut memory = Memory::default();
    // Staying in its territory only keeps the latest trespass.
    for tick in 0..10 {
        memory.remember_trespass(a, tick);
    }
    assert_eq!(memory.0.len(), 1);
    assert_eq!(memory.standing(&a, Feeling::Neutral), Feeling::Unfriendly);
    memory.forget_old(9 + TRESPASS_FORGOTTEN_AFTER - 1);
    assert_eq!(memory.standing(&a, Feeling::Neutral), Feeling::Unfriendly);
    memory.forget_old(9 + TRESPASS_FORGOTTEN_AFTER);
    assert_eq!(memory.standing(&a, Feeling::Neutral), Feeling::Neutral);
}

#[test]
fn being_hit_is_never_forgotten() {
    let mut memory = Memory::default();
    memory.remember_harm(who(1), Points::whole(1), Feeling::Neutral);
    // However much else happens, and however long after.
    for n in 0..(2 * MOST_REMEMBERED as u128) {
        memory.remember_trespass(who(100 + n), 0);
    }
    memory.forget_old(u64::MAX);
    assert_eq!(
        memory.standing(&who(1), Feeling::Neutral),
        Feeling::Unfriendly
    );
    assert!(memory.0.len() <= MOST_REMEMBERED);
}
