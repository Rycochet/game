//! Playtime and sending the idle away, through a connected client.

use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::protocol::ServerMessage;
use shared::world::WorldSeed;

use super::IDLE_MINUTES;
use super::playtime::ACTIVE_FOR;
use super::tests::TestClient;
use crate::ServerWorld;
use crate::storage::{Storage, tests::TestFolder};

/// Action ticks in the usual idle limit.
const IDLE_TICKS: u64 = IDLE_MINUTES as u64 * 60 * ACTION_TICKS_PER_SECOND as u64;

/// A player on a networked server, not its host.
fn joined() -> TestClient {
    let mut test = TestClient::new();
    test.client.host = false;
    test
}

fn waits(test: &mut TestClient, ticks: u64) -> bool {
    test.world.action_clock.advance(ticks);
    test.client
        .keep_time(test.world.action_clock.ticks(), IDLE_MINUTES)
}

#[test]
fn the_idle_are_sent_away() {
    let mut test = joined();
    assert!(waits(&mut test, IDLE_TICKS - 1));
    assert!(!waits(&mut test, 1));
    let told = test.end.receive().unwrap();
    assert!(matches!(told, Some(ServerMessage::Refused(reason)) if reason.contains("idle")));
}

#[test]
fn doing_something_keeps_them() {
    let mut test = joined();
    assert!(waits(&mut test, IDLE_TICKS - 1));
    // What `handle_messages` does with a message that shows activity.
    test.client
        .activity
        .did_something(test.world.action_clock.ticks());
    assert!(waits(&mut test, IDLE_TICKS - 1));
}

#[test]
fn the_host_and_servers_without_a_limit_keep_everyone() {
    let mut host = TestClient::new();
    assert!(waits(&mut host, IDLE_TICKS * 4));
    let mut test = joined();
    test.world.action_clock.advance(IDLE_TICKS * 4);
    assert!(test.client.keep_time(test.world.action_clock.ticks(), 0));
}

#[test]
fn playtime_counts_only_while_about_and_is_saved() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    waits(&mut test, ACTIVE_FOR * 3);
    assert_eq!(test.client.activity.played, ACTIVE_FOR);
    test.leave();
    let again = TestClient::of(open());
    assert_eq!(again.client.activity.played, ACTIVE_FOR);
}
