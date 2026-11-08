//! Difficulty through a connected client: told on joining, changed when asked
//! (never to or from Ironman), refilling by it, and moved when the host
//! changes what the server allows.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use shared::difficulty::{Difficulties, Difficulty};
use shared::protocol::{ClientMessage, ServerMessage};
use shared::vitals::REFILL_TICKS;

use super::difficulty::apply_difficulties;
use super::tests::TestClient;
use super::{Client, Connections};
use crate::{ServerOptions, ServerWorld, ServerWorlds};

/// The last `Difficulty` among these answers.
fn told(answers: &[ServerMessage]) -> Option<(Option<Difficulty>, Difficulties)> {
    answers.iter().rev().find_map(|message| match message {
        ServerMessage::Difficulty { current, allowed } => Some((*current, *allowed)),
        _ => None,
    })
}

/// How fast Health refills, as the client was last told.
fn refill(answers: &[ServerMessage]) -> Option<u32> {
    answers.iter().rev().find_map(|message| match message {
        ServerMessage::Vitals {
            who: None, health, ..
        } => Some(health.refill),
        _ => None,
    })
}

fn set(difficulty: Difficulty) -> ClientMessage {
    ClientMessage::SetDifficulty(difficulty)
}

#[test]
fn told_on_joining() {
    let test = TestClient::new();
    assert_eq!(
        told(&test.welcome),
        Some((Some(Difficulty::Normal), Difficulties::ALL))
    );
    assert_eq!(refill(&test.welcome), Some(REFILL_TICKS as u32));
}

#[test]
fn changed_when_asked_and_easy_refills_faster() {
    let mut test = TestClient::new();
    let answers = test.send(&[set(Difficulty::Easy)]);
    assert_eq!(
        told(&answers),
        Some((Some(Difficulty::Easy), Difficulties::ALL))
    );
    assert_eq!(refill(&answers), Some(REFILL_TICKS as u32 / 2));
    assert!(test.client.unsaved);
    let answers = test.send(&[set(Difficulty::Hard)]);
    assert_eq!(told(&answers).unwrap().0, Some(Difficulty::Hard));
    assert_eq!(refill(&answers), Some(REFILL_TICKS as u32));
}

#[test]
fn ironman_is_never_changed_to_or_from() {
    let mut test = TestClient::new();
    let answers = test.send(&[set(Difficulty::Ironman)]);
    assert_eq!(told(&answers).unwrap().0, Some(Difficulty::Normal));
    test.client.difficulty = Some(Difficulty::Ironman);
    let answers = test.send(&[set(Difficulty::Easy)]);
    assert_eq!(told(&answers).unwrap().0, Some(Difficulty::Ironman));
}

#[test]
fn joining_keeps_the_saved_one_or_takes_the_players_default() {
    let mut client = TestClient::new().client;
    let allowed = Difficulties::ALL;
    // From `Hello`, never Ironman: that is only chosen when making them.
    client.difficulty = Some(Difficulty::Ironman.for_existing());
    client.decide_difficulty(None, allowed);
    assert_eq!(client.difficulty, Some(Difficulty::Hard));
    client.decide_difficulty(Some(Difficulty::Easy), allowed);
    assert_eq!(client.difficulty, Some(Difficulty::Easy));
    // The server's rules come first.
    client.unsaved = false;
    client.decide_difficulty(
        Some(Difficulty::Ironman),
        Difficulties::only(Difficulty::Normal),
    );
    assert_eq!(client.difficulty, Some(Difficulty::Normal));
    assert!(client.unsaved);
}

/// Runs `apply_difficulties` over this client with these options.
fn apply(client: Client, options: ServerOptions) -> (Client, ServerOptions) {
    let mut app = World::new();
    let mut connections = Connections::default();
    connections.clients.push(client);
    app.insert_resource(connections);
    app.insert_resource(options);
    app.insert_resource(ServerWorlds::with_home(ServerWorld::new(
        shared::world::WorldSeed::new(1),
    )));
    app.run_system_once(apply_difficulties).unwrap();
    let mut connections = app.remove_resource::<Connections>().unwrap();
    let options = app.remove_resource::<ServerOptions>().unwrap();
    (connections.clients.remove(0), options)
}

#[test]
fn the_host_changing_what_is_allowed_moves_players_to_the_nearest() {
    let mut test = TestClient::new();
    test.send(&[set(Difficulty::Hard)]);
    let only_easy = Difficulties::only(Difficulty::Easy);
    test.send(&[ClientMessage::SetDifficulties(only_easy)]);
    let (client, options) = apply(test.client, ServerOptions::default());
    assert_eq!(options.difficulties, only_easy);
    assert_eq!(client.difficulty, Some(Difficulty::Easy));
    let answers: Vec<_> = std::iter::from_fn(|| test.end.receive().unwrap()).collect();
    assert_eq!(told(&answers), Some((Some(Difficulty::Easy), only_easy)));
    assert_eq!(refill(&answers), Some(REFILL_TICKS as u32 / 2));
}

#[test]
fn the_environment_wins_over_the_host() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::SetDifficulties(Difficulties::only(
        Difficulty::Ironman,
    ))]);
    let options = ServerOptions {
        difficulties: Difficulties::only(Difficulty::Normal).with(Difficulty::Hard),
        difficulties_fixed: true,
        ..Default::default()
    };
    let (client, options) = apply(test.client, options);
    assert_eq!(
        options.difficulties,
        Difficulties::only(Difficulty::Normal).with(Difficulty::Hard)
    );
    assert_eq!(client.difficulty, Some(Difficulty::Normal));
}
