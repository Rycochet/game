//! Dying by difficulty: what is dropped where they fell, and Ironman's
//! characters marked to be deleted.

use shared::creature::CreatureKind;
use shared::difficulty::Difficulty;
use shared::item::{Item, ItemKind};
use shared::protocol::ServerMessage;
use shared::vitals::{Points, Vital};

use super::tests::{beside, run};
use crate::connections::tests::TestClient;

/// A player on this difficulty killed by a kobold beside them: what lay where
/// they fell, and what they were told.
fn killed_on(difficulty: Difficulty) -> (TestClient, Vec<Item>, Vec<ServerMessage>) {
    let (mut test, _) = beside(CreatureKind::Kobold);
    test.client.difficulty = Some(difficulty);
    let visit = test.client.visit.as_ref().unwrap();
    let (site, level, at) = (visit.id(), visit.index(), visit.position());
    let tick = test.world.action_clock.ticks();
    let limits = test.client.limits(tick);
    let left = test.client.sheet.vitals.at(Vital::Health, limits, tick);
    let harm = Points(left.milli().saturating_sub(1));
    test.client.sheet.vitals.hurt(harm, limits, tick, 0);
    run(&mut test, 80, true);
    assert!(test.client.visit.is_none(), "killed, on the world map");
    let ground = test
        .world
        .site(site)
        .levels
        .get(&level)
        .and_then(|saved| saved.ground.get(&at))
        .cloned()
        .unwrap_or_default();
    let told: Vec<_> = std::iter::from_fn(|| test.end.receive().unwrap()).collect();
    (test, ground, told)
}

fn worn_kinds(test: &TestClient) -> Vec<ItemKind> {
    test.client
        .inventory
        .worn
        .iter()
        .map(|item| item.kind)
        .collect()
}

#[test]
fn normal_keeps_everything() {
    let (test, ground, told) = killed_on(Difficulty::Normal);
    assert!(ground.is_empty());
    assert_eq!(test.client.inventory.worn.len(), 4, "their first clothes");
    assert!(!test.client.died_for_good);
    assert!(told.contains(&ServerMessage::Killed { permanent: false }));
}

#[test]
fn hard_keeps_only_skin_clothes() {
    let (test, ground, told) = killed_on(Difficulty::Hard);
    // Underclothes stay on (owner, 2026-11-09): wearing better ones is the player's choice.
    assert_eq!(worn_kinds(&test), vec![ItemKind::Shirt, ItemKind::Braies]);
    let mut dropped: Vec<_> = ground.iter().map(|item| item.kind).collect();
    dropped.sort_by_key(|kind| format!("{kind:?}"));
    assert_eq!(dropped, vec![ItemKind::Shoes, ItemKind::Trousers]);
    assert!(!test.client.died_for_good);
    assert!(told.contains(&ServerMessage::Killed { permanent: false }));
}

#[test]
fn ironman_drops_everything_and_is_to_be_deleted() {
    let (test, ground, told) = killed_on(Difficulty::Ironman);
    assert!(test.client.inventory.worn.is_empty());
    assert_eq!(ground.len(), 4, "their first clothes");
    assert!(test.client.died_for_good);
    assert!(told.contains(&ServerMessage::Killed { permanent: true }));
}

#[test]
fn ironman_is_deleted_once_the_mobs_turn_is_done() {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::World;

    let (mut test, _, _) = killed_on(Difficulty::Ironman);
    let old = test.client.character_id;
    let seed = test.world.seed;
    let world = std::mem::replace(&mut test.world, crate::ServerWorld::new(seed));
    let mut app = World::new();
    let mut connections = crate::Connections::default();
    connections.clients.push(test.client);
    app.insert_resource(connections);
    app.insert_resource(crate::ServerWorlds::with_home(world));
    app.insert_resource(crate::ServerOptions::default());
    app.run_system_once(crate::connections::run_mobs).unwrap();
    let connections = app.remove_resource::<crate::Connections>().unwrap();
    let client = &connections.clients[0];
    assert_ne!(client.character_id, old);
    assert!(!client.died_for_good);
    assert_eq!(client.phase, crate::connections::Phase::Choosing);
    let told: Vec<_> = std::iter::from_fn(|| test.end.receive().unwrap()).collect();
    assert!(told.iter().any(|message| matches!(
        message,
        ServerMessage::Characters { list, can_create: true, .. } if list.is_empty()
    )));
}
