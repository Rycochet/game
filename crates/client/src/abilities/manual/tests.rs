use bevy::prelude::*;
use shared::ability::Ability;
use shared::id::Id;
use shared::protocol::ClientMessage;

use super::{Manual, ask_wanted};
use crate::common::{OwnAbility, OwnVitals, Target};
use crate::connection::ToServer;
use crate::inventory::Carried;

/// An app asking for what is wanted, with Attack wanted at a target, and the
/// player's ability as `own` says.
fn app(own: OwnAbility) -> App {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<Carried>()
        .init_resource::<OwnVitals>()
        .insert_resource(Target(Some(Id::for_test(1))))
        .insert_resource(own)
        .insert_resource(Manual {
            queued: [Ability::Attack].into(),
            ..default()
        })
        .add_message::<ToServer>()
        .add_systems(Update, ask_wanted);
    app
}

fn sent(app: &mut App) -> Vec<ClientMessage> {
    let messages = app.world().resource::<Messages<ToServer>>();
    let mut reader = messages.get_cursor();
    reader.read(messages).map(|ToServer(m)| m.clone()).collect()
}

#[test]
fn is_asked_for_at_once_when_the_player_can_act() {
    let mut app = app(OwnAbility::default());
    app.update();
    let sent = sent(&mut app);
    assert!(matches!(sent[..], [ClientMessage::UseAbility { .. }]));
    let manual = app.world().resource::<Manual>();
    assert!(manual.queued.is_empty());
    assert_eq!(manual.sent, Some(Ability::Attack));
    assert!(manual.holds(0.0), "automatic ones wait for it");
}

#[test]
fn waits_while_the_player_recovers_and_is_not_lost() {
    let mut app = app(OwnAbility {
        ready: 10.0,
        ..default()
    });
    app.update();
    assert!(sent(&mut app).is_empty());
    assert_eq!(app.world().resource::<Manual>().queued, [Ability::Attack]);
}

#[test]
fn takes_the_place_of_one_still_waiting_on_the_server() {
    // An automatic attack was asked for, early, and not yet begun.
    let mut app = app(OwnAbility {
        ready: 0.1,
        asked_until: 1.0,
        ..default()
    });
    app.update();
    let sent = sent(&mut app);
    assert!(matches!(
        sent[..],
        [
            ClientMessage::WithdrawAbility,
            ClientMessage::UseAbility { .. }
        ]
    ));
}

#[test]
fn holds_automatic_ones_until_it_lands() {
    let mut manual = Manual {
        sent: Some(Ability::Attack),
        sent_until: 1.0,
        ..default()
    };
    manual.begun(Ability::Attack, 3.0);
    assert!(manual.holds(2.0));
    assert!(!manual.holds(3.0));
    manual.forget();
    assert_eq!((manual.queued.len(), manual.sent), (0, None));
}

#[test]
fn heal_without_the_mana_is_not_asked_for() {
    let mut app = app(OwnAbility::default());
    app.world_mut().resource_mut::<Manual>().queued = [Ability::Heal].into();
    app.update();
    assert!(sent(&mut app).is_empty());
}

#[test]
fn heal_by_hand_is_on_the_player_unless_the_target_is_a_friend() {
    use crate::common::ToldVitals;
    use shared::vitals::{Points, VitalNow};
    let mut app = app(OwnAbility::default());
    let full = VitalNow {
        value: Points::whole(20),
        most: 20,
        refills_in: 0,
        refill: shared::vitals::REFILL_TICKS as u32,
    };
    app.insert_resource(OwnVitals(Some(ToldVitals {
        health: full,
        mana: full,
        at: 0.0,
    })));
    app.world_mut().resource_mut::<Manual>().queued = [Ability::Heal].into();
    // The target isn't among those seen, so it isn't known to be a friend.
    app.update();
    assert!(matches!(
        sent(&mut app)[..],
        [ClientMessage::UseAbility {
            ability: Ability::Heal,
            target: None
        }]
    ));
}

#[test]
fn presses_queue_in_order_and_a_second_press_takes_one_out() {
    let mut manual = Manual::default();
    manual.press(Ability::Heal);
    manual.press(Ability::Attack);
    assert_eq!(manual.queued, [Ability::Heal, Ability::Attack]);
    manual.press(Ability::Heal);
    assert_eq!(manual.queued, [Ability::Attack]);
    assert!(
        manual.holds(0.0),
        "automatic ones wait while any are queued"
    );
}

#[test]
fn one_that_cant_be_done_is_passed_over_for_the_next() {
    let mut app = app(OwnAbility::default());
    // Heal without the Mana, then Attack.
    app.world_mut().resource_mut::<Manual>().queued = [Ability::Heal, Ability::Attack].into();
    app.update();
    assert!(matches!(
        sent(&mut app)[..],
        [ClientMessage::UseAbility {
            ability: Ability::Attack,
            ..
        }]
    ));
}

#[test]
fn the_next_waits_for_the_last_asked_for_and_is_not_swapped_for_it() {
    // An ability chosen by hand was asked for and is still waiting on the
    // server: the next in the queue must not take it back.
    let mut app = app(OwnAbility {
        asked_until: 1.0,
        ..default()
    });
    {
        let mut manual = app.world_mut().resource_mut::<Manual>();
        manual.sent = Some(Ability::Heal);
        manual.sent_until = 1.0;
    }
    app.update();
    assert!(sent(&mut app).is_empty());
    assert_eq!(app.world().resource::<Manual>().queued, [Ability::Attack]);
}
