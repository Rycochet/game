//! The action bar through a connected client: told on joining, changed by the
//! rules, put right when refused, saved.

use shared::ability::{Ability, ActionBar, BarChange, BarSlot};
use shared::protocol::{ClientMessage, ServerMessage};
use shared::world::WorldSeed;

use super::tests::TestClient;
use crate::ServerWorld;
use crate::storage::{Storage, tests::TestFolder};

/// The last `ActionBar` among these answers.
fn bar(answers: &[ServerMessage]) -> ActionBar {
    answers
        .iter()
        .rev()
        .find_map(|message| match message {
            ServerMessage::ActionBar(bar) => Some(bar.clone()),
            _ => None,
        })
        .expect("an action bar among the answers")
}

fn change(change: BarChange) -> ClientMessage {
    ClientMessage::ActionBar(change)
}

#[test]
fn a_new_character_is_told_heal_then_attack() {
    assert_eq!(bar(&TestClient::new().welcome), ActionBar::default());
}

#[test]
fn changes_are_made_by_the_rules_and_refusals_put_right() {
    let mut test = TestClient::new();
    let moved = test.send(&[
        change(BarChange::Swap { a: 1, b: 2 }),
        change(BarChange::ToggleAuto { slot: 2 }),
    ]);
    let attack = Some(BarSlot {
        ability: Ability::Attack,
        auto: true,
    });
    assert_eq!(bar(&moved).slots[2], attack);
    // A human knows no Night Vision, and it is passive anyway.
    let refused = test.send(&[change(BarChange::Place {
        slot: 0,
        ability: Ability::NightVision,
    })]);
    assert_eq!(
        bar(&refused).slots[0].map(|s| s.ability),
        Some(Ability::Heal)
    );
    assert_eq!(bar(&refused).slots[2], attack);
}

#[test]
fn kept_in_the_save() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    test.send(&[change(BarChange::Swap { a: 1, b: 9 })]);
    test.leave();
    let again = TestClient::of(open());
    assert_eq!(
        bar(&again.welcome).slots[9].map(|s| s.ability),
        Some(Ability::Attack)
    );
}
