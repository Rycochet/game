//! Choosing between, making and deleting an account's characters.

use shared::creature::CreatureKind;
use shared::protocol::{
    ClientLink, ClientMessage, GuestToken, ListedCharacter, NewCharacter, ServerMessage, local_pair,
};
use shared::sheet::{Appearance, Build};

use super::super::tests::{GUEST, HELLO, MAKES, answers, tick, world_in_memory};
use super::super::{Connections, Phase};
use crate::ServerWorld;

fn make(name: &str) -> ClientMessage {
    ClientMessage::CreateCharacter(NewCharacter {
        name: name.into(),
        kind: CreatureKind::Dwarf,
        gender: shared::gender::Gender::Neutral,
        appearance: Appearance::default(),
        build: Build::CENTRED,
        difficulty: shared::difficulty::Difficulty::Hard,
    })
}

/// What `Characters` offered, if that is all that was told.
fn offered(told: &[ServerMessage]) -> (Vec<ListedCharacter>, bool) {
    match told {
        [
            ServerMessage::Characters {
                list, can_create, ..
            },
        ] => (list.clone(), *can_create),
        _ => panic!("not offered characters: {told:?}"),
    }
}

/// The single player connects again (the old connection still there until
/// they choose), and is offered their characters.
fn single_player(
    connections: Connections,
    world: ServerWorld,
) -> (ClientLink, Connections, ServerWorld) {
    let mut connections = connections;
    let (end, link) = local_pair();
    connections.add_single_player(link);
    end.send(&MAKES);
    let (connections, world) = tick(connections, world);
    (end, connections, world)
}

/// Makes a character, and saves them as the server would soon after.
fn made(
    end: &ClientLink,
    name: &str,
    connections: Connections,
    world: ServerWorld,
) -> (Connections, ServerWorld) {
    end.send(&make(name));
    let (mut connections, mut world) = tick(connections, world);
    let told = answers(end);
    assert!(matches!(told[0], ServerMessage::Welcome { .. }), "{told:?}");
    let client = connections.clients.last_mut().unwrap();
    client.save(&mut world);
    (connections, world)
}

#[test]
fn the_single_player_may_have_several_shown_as_they_are() {
    let (_backend, _queue, world) = world_in_memory();
    let (end, connections, world) = single_player(Connections::default(), world);
    assert_eq!(offered(&answers(&end)), (Vec::new(), true));
    let (connections, world) = made(&end, "Ash", connections, world);

    let (end, connections, world) = single_player(connections, world);
    let (list, can_create) = offered(&answers(&end));
    assert!(can_create);
    let [ash] = &list[..] else {
        panic!("{list:?}");
    };
    assert_eq!(ash.summary.name, "Ash");
    assert_eq!(ash.kind, CreatureKind::Dwarf);
    assert_ne!(ash.summary.created, 0);
    assert_eq!(ash.difficulty, Some(shared::difficulty::Difficulty::Hard));
    let (connections, world) = made(&end, "Bo", connections, world);

    let (end, _connections, _world) = single_player(connections, world);
    let (list, _) = offered(&answers(&end));
    assert_eq!(list.len(), 2);
}

#[test]
fn a_guest_may_have_only_one() {
    let (_backend, _queue, world) = world_in_memory();
    let mut connections = Connections::default();
    let (end, link) = local_pair();
    connections.add(link);
    end.send(&MAKES);
    end.send(&GUEST);
    let (connections, world) = tick(connections, world);
    assert_eq!(offered(&answers(&end)[1..]), (Vec::new(), true));
    let (mut connections, world) = made(&end, "Ash", connections, world);

    let (again, link) = local_pair();
    connections.add(link);
    again.send(&MAKES);
    again.send(&GUEST);
    let (connections, world) = tick(connections, world);
    let (list, can_create) = offered(&answers(&again)[1..]);
    assert_eq!((list.len(), can_create), (1, false));
    again.send(&make("Bo"));
    let _ = tick(connections, world);
    assert!(matches!(answers(&again)[..], [ServerMessage::Refused(_)]));
}

#[test]
fn a_character_is_deleted_only_by_their_own_account_while_choosing() {
    let (_backend, _queue, world) = world_in_memory();
    let (end, connections, world) = single_player(Connections::default(), world);
    answers(&end);
    let (connections, world) = made(&end, "Ash", connections, world);
    let ash = connections.clients[0].character_id.unwrap();

    // Not while playing.
    end.send(&ClientMessage::DeleteCharacter { id: ash });
    let (mut connections, world) = tick(connections, world);
    assert!(matches!(answers(&end)[..], [ServerMessage::Refused(_)]));

    // Not someone else's.
    let (guest, link) = local_pair();
    connections.add(link);
    guest.send(&MAKES);
    guest.send(&ClientMessage::Login(shared::protocol::Credentials::Guest(
        GuestToken([6; 32]),
    )));
    guest.send(&ClientMessage::DeleteCharacter { id: ash });
    let (connections, world) = tick(connections, world);
    assert!(matches!(
        answers(&guest)[..],
        [
            _,
            ServerMessage::Characters { .. },
            ServerMessage::Refused(_)
        ]
    ));

    // Their own, while choosing: gone, save and all.
    let (end, connections, world) = single_player(connections, world);
    assert_eq!(offered(&answers(&end)).0.len(), 1);
    end.send(&ClientMessage::DeleteCharacter { id: ash });
    let (mut connections, world) = tick(connections, world);
    assert_eq!(offered(&answers(&end)), (Vec::new(), true));
    assert!(world.storage.load_sheet(&ash).unwrap().is_none());
    let client = connections.clients.last().unwrap();
    assert_eq!(client.phase, Phase::Choosing);
    let account = client.account.unwrap();
    assert!(
        connections
            .accounts
            .account(&account)
            .unwrap()
            .characters
            .is_empty()
    );
}

#[test]
fn a_client_without_the_screens_plays_the_latest() {
    let (_backend, _queue, world) = world_in_memory();
    let (end, connections, world) = single_player(Connections::default(), world);
    answers(&end);
    let (connections, world) = made(&end, "Ash", connections, world);
    let (end, connections, world) = single_player(connections, world);
    answers(&end);
    let (mut connections, world) = made(&end, "Bo", connections, world);
    let bo = connections.clients.last().unwrap().character_id;

    let (plain, link) = local_pair();
    connections.add_single_player(link);
    plain.send(&HELLO);
    let (connections, _world) = tick(connections, world);
    assert!(matches!(answers(&plain)[0], ServerMessage::Welcome { .. }));
    assert_eq!(connections.clients.last().unwrap().character_id, bo);
}
