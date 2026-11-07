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

#[test]
fn a_new_bar_has_attack_first() {
    let bar = ActionBar::default();
    assert_eq!(bar.slots[0], slot(Ability::Attack, false));
    assert!(bar.slots[1..].iter().all(Option::is_none));
}

#[test]
fn passive_and_unknown_abilities_stay_off_the_bar() {
    let mut bar = ActionBar::default();
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
    let mut bar = ActionBar::default();
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
    let mut bar = ActionBar::default();
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
    let mut bar = ActionBar::default();
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
    let mut bar = ActionBar::default();
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
