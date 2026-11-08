use super::*;
use crate::creature::CreatureKind;

/// A kind that knows Kick and Trample besides Attack.
fn kicker() -> CreatureKind {
    *CreatureKind::ALL
        .iter()
        .find(|kind| kind.def().abilities.contains(&Ability::Kick))
        .expect("some kind kicks")
}

fn slot(ability: Ability, auto: bool) -> Option<BarSlot> {
    Some(BarSlot { ability, auto })
}

#[test]
fn every_creature_knows_attack_first() {
    for kind in CreatureKind::ALL {
        assert_eq!(known(kind).first(), Some(&Ability::Attack), "{kind:?}");
    }
}

#[test]
fn every_ability_is_listed_once() {
    for ability in Ability::ALL {
        assert_eq!(Ability::ALL.iter().filter(|a| **a == ability).count(), 1);
    }
}

/// A bar with only Attack, in the first slot, as from before Heal.
fn attack_only() -> ActionBar {
    let mut slots = [None; BAR_SLOTS];
    slots[0] = slot(Ability::Attack, false);
    ActionBar {
        slots,
        ..ActionBar::default()
    }
}

#[test]
fn every_creature_knows_heal() {
    for kind in CreatureKind::ALL {
        assert!(Ability::Heal.can_use(kind), "{kind:?}");
    }
    assert!(Ability::Heal.can_be_automatic());
}

#[test]
fn a_new_bar_has_heal_then_attack() {
    let bar = ActionBar::default();
    assert_eq!(bar.slots[0], slot(Ability::Heal, false));
    assert_eq!(bar.slots[1], slot(Ability::Attack, false));
    assert!(bar.slots[2..].iter().all(Option::is_none));
}

#[test]
fn an_old_bar_is_given_heal_once_in_the_first_empty_slot() {
    let mut bar = attack_only();
    bar.given = 0;
    bar.catch_up();
    assert_eq!(bar.slots[1], slot(Ability::Heal, false));
    // Taken off, it stays off.
    bar.apply(BarChange::Clear { slot: 1 }, CreatureKind::Human)
        .unwrap();
    bar.catch_up();
    assert_eq!(bar.slot_of(Ability::Heal), None);
    // A bar saved without `given` reads as 0.
    let old: ActionBar =
        ron::from_str("(slots: (None, None, None, None, None, None, None, None, None, None))")
            .unwrap();
    assert_eq!(old.given, 0);
}

#[test]
fn every_ability_has_a_description() {
    for ability in Ability::ALL {
        assert!(ability.description().ends_with('.'), "{ability:?}");
    }
}

#[test]
fn passive_and_unknown_abilities_stay_off_the_bar() {
    let mut bar = attack_only();
    let place = |ability| BarChange::Place { slot: 3, ability };
    assert_eq!(
        bar.apply(place(Ability::NightVision), CreatureKind::Human),
        Err(BarRefusal::Passive)
    );
    assert_eq!(
        bar.apply(place(Ability::Kick), CreatureKind::Human),
        Err(BarRefusal::Unknown)
    );
    assert_eq!(bar.apply(place(Ability::Kick), kicker()), Ok(()));
    assert_eq!(bar.slots[3], slot(Ability::Kick, false));
}

#[test]
fn placing_an_ability_already_on_the_bar_moves_it_with_its_auto() {
    let mut bar = attack_only();
    let kind = CreatureKind::Human;
    bar.apply(BarChange::ToggleAuto { slot: 0 }, kind).unwrap();
    bar.apply(
        BarChange::Place {
            slot: 4,
            ability: Ability::Attack,
        },
        kind,
    )
    .unwrap();
    assert_eq!(bar.slots[0], None);
    assert_eq!(bar.slots[4], slot(Ability::Attack, true));
}

#[test]
fn swapping_keeps_each_auto_and_clearing_forgets_it() {
    let mut bar = attack_only();
    let kind = kicker();
    bar.apply(
        BarChange::Place {
            slot: 1,
            ability: Ability::Kick,
        },
        kind,
    )
    .unwrap();
    bar.apply(BarChange::ToggleAuto { slot: 0 }, kind).unwrap();
    bar.apply(BarChange::Swap { a: 0, b: 1 }, kind).unwrap();
    assert_eq!(bar.slots[0], slot(Ability::Kick, false));
    assert_eq!(bar.slots[1], slot(Ability::Attack, true));
    bar.apply(BarChange::Clear { slot: 1 }, kind).unwrap();
    bar.apply(
        BarChange::Place {
            slot: 1,
            ability: Ability::Attack,
        },
        kind,
    )
    .unwrap();
    assert_eq!(bar.slots[1], slot(Ability::Attack, false));
}

#[test]
fn only_some_abilities_can_be_made_automatic() {
    let mut bar = attack_only();
    let kind = kicker();
    bar.apply(
        BarChange::Place {
            slot: 1,
            ability: Ability::Kick,
        },
        kind,
    )
    .unwrap();
    assert_eq!(
        bar.apply(BarChange::ToggleAuto { slot: 1 }, kind),
        Err(BarRefusal::NotAutomatic)
    );
    assert!(Ability::Attack.can_be_automatic());
    assert!(bar.apply(BarChange::ToggleAuto { slot: 0 }, kind).is_ok());
}

#[test]
fn bad_slots_and_empty_autos_are_refused_without_change() {
    let mut bar = attack_only();
    let before = bar.clone();
    let kind = CreatureKind::Human;
    assert_eq!(
        bar.apply(BarChange::Swap { a: 0, b: 10 }, kind),
        Err(BarRefusal::NoSlot)
    );
    assert_eq!(
        bar.apply(BarChange::ToggleAuto { slot: 5 }, kind),
        Err(BarRefusal::Empty)
    );
    assert_eq!(bar, before);
}

#[test]
fn only_known_active_abilities_can_be_used() {
    assert!(Ability::Attack.can_use(CreatureKind::Human));
    assert!(!Ability::Kick.can_use(CreatureKind::Human));
    assert!(Ability::Kick.can_use(kicker()));
    assert!(!Ability::NightVision.can_use(CreatureKind::Human));
}

#[test]
fn every_ability_leans_on_something() {
    for ability in Ability::ALL {
        assert!(!ability.leans().is_empty(), "{ability:?}");
    }
    use crate::sheet::{Attribute, Row};
    assert_eq!(Attribute::Perception.row(), Row::Mind);
    for row in Row::ALL {
        for attribute in row.attributes() {
            assert_eq!(attribute.row(), row);
        }
    }
}

#[test]
fn everyone_knows_sneak_a_toggle_for_the_bar() {
    assert!(Ability::Sneak.is_toggle());
    assert!(Ability::Sneak.is_active(), "it goes on the bar");
    assert!(!Ability::Sneak.can_be_automatic());
    for kind in CreatureKind::ALL {
        assert!(known(kind).contains(&Ability::Sneak), "{kind:?}");
    }
}
