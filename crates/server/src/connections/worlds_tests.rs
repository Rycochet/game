//! Players in more than one world at once.

use bevy::ecs::system::RunSystemOnce;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::item::Inventory;
use shared::protocol::{CharacterId, ClientLink, ClientMessage, ServerMessage, local_pair};
use shared::sheet::Sheet;
use shared::world::{WorldId, WorldSeed};
use std::collections::BTreeMap;

use super::tests::HELLO;
use super::*;
use crate::storage::Storage;
use crate::storage::tests::TestFolder;
use crate::{ServerOptions, ServerWorlds};

const HOME: WorldSeed = WorldSeed::new(1);
const OTHER: WorldSeed = WorldSeed::new(2);

fn character(number: u128) -> CharacterId {
    CharacterId(Id::for_test(number))
}

/// A server saving in `folder`, with these characters saved in these worlds
/// (opened, saved and closed again first, so they open from their saves).
fn server(folder: &TestFolder, saved_in: &[(CharacterId, WorldSeed)]) -> bevy::prelude::World {
    let storage = || Storage::in_folder(&folder.0, HOME.id());
    {
        let mut worlds = ServerWorlds::start(HOME, storage()).unwrap();
        for (id, seed) in saved_in {
            let world = worlds.open(*seed).unwrap();
            worlds.home().storage.save_character(
                id,
                world,
                &CharacterWorldSave::default(),
                &BTreeMap::new(),
                &Inventory::default(),
                &Sheet::new(CreatureKind::Human),
            );
        }
        worlds.save_worlds();
        worlds.home().storage.finish_writing();
    }
    let mut app = bevy::prelude::World::new();
    app.insert_resource(Connections::default());
    app.insert_resource(ServerWorlds::start(HOME, storage()).unwrap());
    app.insert_resource(ServerOptions {
        dev_tools: true,
        guests: true,
        ..Default::default()
    });
    app
}

/// Connects a character and has them join, answering with their end and the
/// world they were welcomed into.
fn join(app: &mut bevy::prelude::World, id: CharacterId) -> (ClientLink, WorldId) {
    let (end, link) = local_pair();
    app.resource_mut::<Connections>().add_playing(link, id);
    end.send(&HELLO);
    app.run_system_once(handle_messages).unwrap();
    let welcomed = std::iter::from_fn(|| end.receive().ok().flatten())
        .find_map(|message| match message {
            ServerMessage::Welcome { world, .. } => Some(world.id),
            _ => None,
        })
        .expect("welcomed");
    (end, welcomed)
}

#[test]
fn characters_join_in_the_world_they_were_saved_in() {
    let folder = TestFolder::new();
    let (there, new) = (character(1), character(2));
    let mut app = server(&folder, &[(there, OTHER)]);
    // Kept, or they would leave.
    let (_there_end, world) = join(&mut app, there);
    assert_eq!(world, OTHER.id());
    // Someone new arrives in the home world.
    let (_new_end, world) = join(&mut app, new);
    assert_eq!(world, HOME.id());
    let worlds = app.resource::<ServerWorlds>();
    assert!(worlds.get(OTHER.id()).is_some());
    let clients = &app.resource::<Connections>().clients;
    assert_eq!(clients[0].world, Some(OTHER.id()));
    assert_eq!(clients[1].world, Some(HOME.id()));
}

#[test]
fn a_world_that_cant_be_run_sends_them_home() {
    let folder = TestFolder::new();
    let id = character(1);
    let mut app = server(&folder, &[(id, OTHER)]);
    let file = folder.0.join(format!("worlds/{}/world.ron", OTHER.id().0));
    std::fs::write(&file, "not a world").unwrap();
    assert_eq!(join(&mut app, id).1, HOME.id());
}

/// Whether the first character sees the second, both at the same place on the
/// world map, or in the Hall.
fn sees_the_other(saved_in: [WorldSeed; 2], in_hall: bool) -> bool {
    let folder = TestFolder::new();
    let ids = [character(1), character(2)];
    let mut app = server(&folder, &[(ids[0], saved_in[0]), (ids[1], saved_in[1])]);
    let (first, _) = join(&mut app, ids[0]);
    let (_second, _) = join(&mut app, ids[1]);
    for client in &mut app.resource_mut::<Connections>().clients {
        client.character.world_position = (100, 100);
        client.character.hall = in_hall.then_some((100, 1));
    }
    while first.receive().ok().flatten().is_some() {}
    app.run_system_once(sync_entities).unwrap();
    std::iter::from_fn(|| first.receive().ok().flatten())
        .any(|message| matches!(message, ServerMessage::EntitySeen(look) if look.id == ids[1].0))
}

#[test]
fn players_only_see_others_in_their_world() {
    assert!(sees_the_other([OTHER, OTHER], false));
    assert!(!sees_the_other([HOME, OTHER], false));
}

#[test]
fn everyone_in_the_hall_sees_each_other_whatever_world_they_came_from() {
    assert!(sees_the_other([HOME, OTHER], true));
}

#[test]
fn moving_one_worlds_time_tells_only_those_in_it() {
    let folder = TestFolder::new();
    let ids = [character(1), character(2)];
    let mut app = server(&folder, &[(ids[0], HOME), (ids[1], OTHER)]);
    let (home, _) = join(&mut app, ids[0]);
    let (other, _) = join(&mut app, ids[1]);
    while home.receive().ok().flatten().is_some() {}
    while other.receive().ok().flatten().is_some() {}
    let mut worlds = app.resource_mut::<ServerWorlds>();
    let world = worlds.get_mut(OTHER.id()).unwrap();
    world.set_time(None, shared::world::ClockRate::new(2, 1));
    app.run_system_once(tell_the_time).unwrap();
    let told = |end: &ClientLink| {
        std::iter::from_fn(|| end.receive().ok().flatten())
            .any(|message| matches!(message, ServerMessage::WorldClock(_)))
    };
    assert!(told(&other));
    assert!(!told(&home));
}

/// Everything waiting at a client's end.
fn answers(end: &ClientLink) -> Vec<ServerMessage> {
    std::iter::from_fn(|| end.receive().ok().flatten()).collect()
}

#[test]
fn dev_tools_move_a_player_to_another_world_and_back() {
    let folder = TestFolder::new();
    let id = character(1);
    let mut app = server(&folder, &[]);
    let (end, home) = join(&mut app, id);
    assert_eq!(home, HOME.id());
    let start = app.resource::<Connections>().clients[0].world_position();
    answers(&end);

    end.send(&ClientMessage::DevGoToWorld {
        seed: OTHER.value(),
    });
    app.run_system_once(handle_messages).unwrap();
    let told = answers(&end);
    let welcomed = told.iter().find_map(|message| match message {
        ServerMessage::Welcome { world, x, y, .. } => Some((world.id, (*x, *y))),
        _ => None,
    });
    let (world, arrived) = welcomed.expect("welcomed into the other world");
    assert_eq!(world, OTHER.id());
    assert!(told.contains(&ServerMessage::OnWorldMap {
        x: arrived.0,
        y: arrived.1,
    }));
    let client = &app.resource::<Connections>().clients[0];
    assert_eq!(client.world, Some(OTHER.id()));
    // Its own World Gates: another planet.
    let other = app.resource::<ServerWorlds>().get(OTHER.id()).unwrap();
    let gates = &other.pois.world_gates().gates;
    assert!(gates.iter().any(|gate| (gate.x, gate.y) == arrived));
    // Saved as there.
    let storage = app.resource::<ServerWorlds>().home().storage.clone();
    assert_eq!(storage.character_world(&id), Ok(Some(OTHER.id())));

    // Back where they were.
    end.send(&ClientMessage::DevGoToWorld { seed: HOME.value() });
    app.run_system_once(handle_messages).unwrap();
    let told = answers(&end);
    assert!(told.contains(&ServerMessage::OnWorldMap {
        x: start.0,
        y: start.1,
    }));
    assert_eq!(
        app.resource::<Connections>().clients[0].world,
        Some(HOME.id())
    );
}

#[test]
fn going_to_the_world_they_are_in_changes_nothing() {
    let folder = TestFolder::new();
    let mut app = server(&folder, &[]);
    let (end, _) = join(&mut app, character(1));
    answers(&end);
    end.send(&ClientMessage::DevGoToWorld { seed: HOME.value() });
    app.run_system_once(handle_messages).unwrap();
    assert_eq!(answers(&end), []);
}
