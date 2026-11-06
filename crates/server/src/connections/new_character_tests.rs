//! Making a new guest's or single player's character with the client's screen.

use shared::creature::CreatureKind;
use shared::protocol::{ClientLink, ClientMessage, PROTOCOL_VERSION, ServerMessage, local_pair};
use shared::sheet::{Appearance, Build, Dot, Sheet};
use shared::world::WorldSeed;

use super::tests::{GUEST, answers, tick};
use super::*;
use crate::ServerWorld;

const MAKES: ClientMessage = ClientMessage::Hello {
    protocol: PROTOCOL_VERSION,
    makes_characters: true,
};

fn strong() -> Build {
    Build {
        dots: [Dot([1000, 0, 0]), Dot::CENTRE, Dot::CENTRE],
        turn: 0,
    }
}

fn make(name: &str, kind: CreatureKind, build: Build) -> ClientMessage {
    ClientMessage::CreateCharacter {
        name: name.into(),
        kind,
        appearance: Appearance::default(),
        build,
    }
}

fn sheet_in(told: &[ServerMessage]) -> Option<Sheet> {
    told.iter().find_map(|answer| match answer {
        ServerMessage::Sheet(sheet) => Some(sheet.clone()),
        _ => None,
    })
}

/// A new guest on a client that makes characters: logged in, and asked to make one.
fn new_guest() -> (ClientLink, Connections, ServerWorld) {
    let world = ServerWorld::new(WorldSeed::new(1));
    let mut connections = Connections::default();
    let (end, link) = local_pair();
    connections.add(link);
    end.send(&MAKES);
    end.send(&GUEST);
    let (connections, world) = tick(connections, world);
    let told = answers(&end);
    assert!(
        matches!(&told[1], ServerMessage::Characters { list, can_create: true } if list.is_empty()),
        "{told:?}"
    );
    (end, connections, world)
}

#[test]
fn a_new_guest_makes_their_character_before_joining() {
    let (end, connections, world) = new_guest();
    // Nothing happens until they have made it.
    end.send(&ClientMessage::UseExit);
    let (connections, _world) = tick(connections, world);
    assert!(matches!(answers(&end)[..], [ServerMessage::Refused(_)]));
    assert!(connections.clients.is_empty());

    let (end, mut connections, world) = new_guest();
    end.send(&make("Brin", CreatureKind::Dwarf, strong()));
    let (connections_after, _world) = tick(connections, world);
    connections = connections_after;
    let told = answers(&end);
    assert!(matches!(told[0], ServerMessage::Welcome { .. }), "{told:?}");
    assert_eq!(
        sheet_in(&told),
        Some(Sheet::built(CreatureKind::Dwarf, strong()))
    );
    // Their one character has the name they chose.
    let account = connections.clients[0].account.unwrap();
    let summary = &connections.accounts.account(&account).unwrap().characters[0];
    assert_eq!(summary.name, "Brin");
}

#[test]
fn what_a_new_guest_asks_for_is_checked() {
    let outside = Build {
        dots: [Dot([600, 600, 0]), Dot::CENTRE, Dot::CENTRE],
        turn: 0,
    };
    for asked in [
        make("Brin", CreatureKind::Horse, Build::CENTRED),
        make("Brin", CreatureKind::Dwarf, outside),
        make("   ", CreatureKind::Dwarf, Build::CENTRED),
    ] {
        let (end, connections, world) = new_guest();
        end.send(&asked);
        let _ = tick(connections, world);
        let told = answers(&end);
        assert!(
            matches!(told[..], [ServerMessage::Refused(_)]),
            "{asked:?}: {told:?}"
        );
    }
}

#[test]
fn a_client_without_the_screen_joins_plain_and_cant_make_one_afterwards() {
    let world = ServerWorld::new(WorldSeed::new(1));
    let mut connections = Connections::default();
    let (end, link) = local_pair();
    connections.add(link);
    end.send(&super::tests::HELLO);
    end.send(&GUEST);
    let (connections, world) = tick(connections, world);
    let told = answers(&end);
    assert!(matches!(told[1], ServerMessage::Welcome { .. }), "{told:?}");
    end.send(&make("Brin", CreatureKind::Dwarf, strong()));
    let _ = tick(connections, world);
    assert!(matches!(answers(&end)[..], [ServerMessage::Refused(_)]));
}

#[test]
fn the_single_player_makes_theirs_too() {
    let world = ServerWorld::new(WorldSeed::new(1));
    let mut connections = Connections::default();
    let (end, link) = local_pair();
    connections.add_single_player(link);
    end.send(&MAKES);
    let (mut connections, world) = tick(connections, world);
    let told = answers(&end);
    assert!(
        matches!(told[..], [ServerMessage::Characters { .. }]),
        "{told:?}"
    );
    end.send(&make("Ash", CreatureKind::Elf, Build::CENTRED));
    (connections, _) = tick(connections, world);
    let told = answers(&end);
    assert!(matches!(told[0], ServerMessage::Welcome { .. }), "{told:?}");
    assert_eq!(sheet_in(&told), Some(Sheet::new(CreatureKind::Elf)));
    assert_eq!(connections.clients.len(), 1);
}
