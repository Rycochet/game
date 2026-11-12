//! Killing moves the killer's alignment, unless it was self-defence.

use shared::ability::Ability;
use shared::alignment::{Alignment, Deed};
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::protocol::ClientMessage;

use super::tests::{beside, mob_health, run};
use crate::connections::tests::TestClient;

/// Kills the mob with blows, and lets its killer be told.
fn kill(test: &mut TestClient, mob: Id) {
    for _ in 0..200 {
        if mob_health(test, mob).is_none() {
            break;
        }
        test.send(&[ClientMessage::UseAbility {
            ability: Ability::Attack,
            target: Some(mob),
        }]);
        run(test, 10, false);
    }
    assert_eq!(mob_health(test, mob), None);
    crate::entities::sync(std::slice::from_mut(&mut test.client), &mut test.world, &[]);
}

/// A mob beside the player, of this kind and alignment.
fn one(kind: CreatureKind, alignment: Option<Alignment>) -> (TestClient, Id) {
    let (mut test, mob) = beside(kind);
    let population = test.world.populations.0.values_mut().next().unwrap();
    population.mobs[0].alignment = alignment;
    (test, mob)
}

#[test]
fn killing_someone_good_unprovoked_lowers_it() {
    let good = Alignment::new(800);
    let (mut test, mob) = one(CreatureKind::Dwarf, Some(good));
    assert_eq!(test.client.sheet.alignment, Some(Alignment::NEUTRAL));
    kill(&mut test, mob);
    let deed = Deed::Killed {
        victim: good,
        provoked: false,
    };
    assert!(deed.shift() < 0);
    assert_eq!(
        test.client.sheet.alignment,
        Some(Alignment::NEUTRAL.after(deed))
    );
}

#[test]
fn killing_someone_evil_or_an_animal_changes_nothing() {
    let (mut test, mob) = one(CreatureKind::Dwarf, Some(Alignment::new(-700)));
    kill(&mut test, mob);
    assert_eq!(test.client.sheet.alignment, Some(Alignment::NEUTRAL));
    let (mut test, mob) = one(CreatureKind::Cattle, None);
    kill(&mut test, mob);
    assert_eq!(test.client.sheet.alignment, Some(Alignment::NEUTRAL));
}

#[test]
fn killing_a_hostile_mob_is_self_defence() {
    // Orcs dislike everyone from the start: neutral as this one may be.
    let (mut test, mob) = one(CreatureKind::Orc, Some(Alignment::NEUTRAL));
    kill(&mut test, mob);
    assert_eq!(test.client.sheet.alignment, Some(Alignment::NEUTRAL));
}

#[test]
fn a_players_alignment_is_never_sent() {
    let (mut test, _) = one(CreatureKind::Dwarf, None);
    test.client.sheet.alignment = Some(Alignment::new(-500));
    let sent = test.send(
        &[ClientMessage::DevVitals(shared::protocol::DevVitals::Hurt(
            1,
        ))],
    );
    let sheets: Vec<_> = sent
        .iter()
        .filter_map(|message| match message {
            shared::protocol::ServerMessage::Sheet(sheet) => Some(sheet),
            _ => None,
        })
        .collect();
    assert!(!sheets.is_empty());
    assert!(sheets.iter().all(|sheet| sheet.alignment.is_none()));
}

#[test]
fn the_debug_panel_asks_and_moves_it() {
    let (mut test, _) = one(CreatureKind::Dwarf, None);
    use shared::protocol::{DevAlignment, ServerMessage};
    let told = test.send(&[ClientMessage::DevAlignment(DevAlignment::Shift(-400))]);
    assert!(told.contains(&ServerMessage::DevAlignment(Alignment::new(-400))));
    assert_eq!(test.client.sheet.alignment, Some(Alignment::new(-400)));
    test.send(&[ClientMessage::DevAlignment(DevAlignment::Set(0))]);
    assert_eq!(test.client.sheet.alignment, Some(Alignment::NEUTRAL));
}
