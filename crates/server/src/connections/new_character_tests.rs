//! Making a new guest's or single player's character with the client's screen.

use shared::creature::CreatureKind;
use shared::gender::Gender;
use shared::protocol::{
    ClientLink, ClientMessage, Credentials, GuestToken, ServerMessage, local_pair,
};
use shared::sheet::{Appearance, Build, Dot, Sheet};
use shared::world::WorldSeed;

use super::tests::{GUEST, MAKES, answers, tick};
use super::*;
use crate::ServerWorld;

fn strong() -> Build {
    Build {
        dots: [Dot([1000, 0, 0]), Dot::CENTRE, Dot::CENTRE],
        turn: 0,
    }
}

fn make(name: &str, kind: CreatureKind, build: Build) -> ClientMessage {
    ClientMessage::CreateCharacter(shared::protocol::NewCharacter {
        name: name.into(),
        kind,
        gender: Gender::Male,
        appearance: Appearance::default(),
        build,
        difficulty: shared::difficulty::Difficulty::Normal,
    })
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
        matches!(&told[1], ServerMessage::Characters { list, can_create: true, .. } if list.is_empty()),
        "{told:?}"
    );
    (end, connections, world)
}

#[test]
fn a_new_guest_makes_their_character_before_joining() {
    let (end, connections, world) = new_guest();
    // Nothing happens until they have made it: what they ask meanwhile is ignored.
    end.send(&ClientMessage::UseExit);
    let (connections, _world) = tick(connections, world);
    assert!(answers(&end).is_empty());
    assert_eq!(connections.clients[0].phase, Phase::Choosing);

    let (end, mut connections, world) = new_guest();
    end.send(&make("Brin", CreatureKind::Dwarf, strong()));
    let (connections_after, _world) = tick(connections, world);
    connections = connections_after;
    let told = answers(&end);
    assert!(matches!(told[0], ServerMessage::Welcome { .. }), "{told:?}");
    assert_eq!(
        sheet_in(&told),
        Some(Sheet::built(CreatureKind::Dwarf, Gender::Male, strong()).as_sent())
    );
    // The server added a man's bonus to Body itself.
    let sheet = sheet_in(&told).unwrap();
    let neutral = Sheet::built(CreatureKind::Dwarf, Gender::Neutral, strong());
    assert_eq!(sheet.gender, Gender::Male);
    assert_eq!(
        sheet.attributes.strength,
        neutral.attributes.strength + shared::gender::GENDER_BONUS
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
            matches!(told[..], [ServerMessage::CharacterRefused(_)]),
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
    assert_eq!(
        sheet_in(&told),
        Some(Sheet::built(CreatureKind::Elf, Gender::Male, Build::CENTRED).as_sent())
    );
    assert_eq!(connections.clients.len(), 1);
}

#[test]
fn deleting_a_character_offers_the_rest_and_a_new_one_has_a_new_id() {
    let world = ServerWorld::new(WorldSeed::new(1));
    let mut connections = Connections::default();
    let (end, link) = local_pair();
    connections.add_single_player(link);
    end.send(&MAKES);
    end.send(&make("Ash", CreatureKind::Elf, Build::CENTRED));
    let (connections, world) = tick(connections, world);
    let told = answers(&end);
    assert!(matches!(told[1], ServerMessage::Welcome { .. }), "{told:?}");
    let old = connections.clients[0].character_id.unwrap();

    end.send(&ClientMessage::DevDeleteCharacter);
    // Something sent before, still on its way: ignored, not refused.
    end.send(&ClientMessage::UseExit);
    let (connections, world) = tick(connections, world);
    let told = answers(&end);
    assert!(
        matches!(told[..], [ServerMessage::Characters { .. }]),
        "{told:?}"
    );
    assert_eq!(connections.clients[0].character_id, None);
    assert_eq!(connections.clients[0].phase, Phase::Choosing);

    end.send(&make("Bo", CreatureKind::Dwarf, strong()));
    let (mut connections, _world) = tick(connections, world);
    let told = answers(&end);
    assert!(matches!(told[0], ServerMessage::Welcome { .. }), "{told:?}");
    assert_eq!(
        sheet_in(&told),
        Some(Sheet::built(CreatureKind::Dwarf, Gender::Male, strong()).as_sent())
    );
    let new = connections.clients[0].character_id.unwrap();
    assert_ne!(new, old);
    // The account has only the new one.
    let key = crate::accounts::LoginKey::SinglePlayer;
    let account = connections.accounts.log_in(&key).unwrap();
    let characters = &connections.accounts.account(&account).unwrap().characters;
    assert_eq!(characters.len(), 1);
    assert_eq!((characters[0].id, characters[0].name.as_str()), (new, "Bo"));
}

#[test]
fn a_taken_name_is_turned_down_and_the_player_chooses_again() {
    let (first, connections, world) = new_guest();
    first.send(&make("Brin", CreatureKind::Dwarf, strong()));
    let (mut connections, world) = tick(connections, world);
    let _ = answers(&first);

    let (end, link) = local_pair();
    connections.add(link);
    end.send(&MAKES);
    // Another guest.
    end.send(&ClientMessage::Login(Credentials::Guest(GuestToken(
        [2; 32],
    ))));
    let (connections, world) = tick(connections, world);
    let _ = answers(&end);
    // Whatever its capitals.
    end.send(&make("brin", CreatureKind::Dwarf, strong()));
    let (connections, world) = tick(connections, world);
    let told = answers(&end);
    assert!(
        matches!(&told[..], [ServerMessage::CharacterRefused(why)] if why.contains("taken")),
        "{told:?}"
    );
    // Still connected and choosing: another name will do.
    assert_eq!(connections.clients[1].phase, Phase::Choosing);
    end.send(&make("Ash", CreatureKind::Dwarf, strong()));
    let _ = tick(connections, world);
    let told = answers(&end);
    assert!(matches!(told[0], ServerMessage::Welcome { .. }), "{told:?}");
}

#[test]
fn a_female_character_may_not_have_a_beard() {
    let bearded = Appearance(std::collections::BTreeMap::from([(
        shared::sheet::Trait::Beard,
        2,
    )]));
    let asked = |gender| {
        ClientMessage::CreateCharacter(shared::protocol::NewCharacter {
            gender,
            appearance: bearded.clone(),
            ..match make("Brin", CreatureKind::Dwarf, strong()) {
                ClientMessage::CreateCharacter(new) => new,
                _ => unreachable!(),
            }
        })
    };
    let (end, connections, world) = new_guest();
    end.send(&asked(Gender::Female));
    let _ = tick(connections, world);
    let told = answers(&end);
    assert!(
        matches!(told[..], [ServerMessage::CharacterRefused(_)]),
        "{told:?}"
    );
    let (end, connections, world) = new_guest();
    end.send(&asked(Gender::Male));
    let _ = tick(connections, world);
    assert!(matches!(answers(&end)[0], ServerMessage::Welcome { .. }));
}
