use crate::ServerWorlds;
use bevy::ecs::system::RunSystemOnce;
use shared::protocol::{ClientLink, ServerMessage, local_pair};
use shared::role::{OWNER, Permission, RoleId};
use shared::world::WorldSeed;

use super::lockdown::{LOCKED_DOWN, apply_lockdown};
use super::login::logged_in;
use super::tests::answers;
use super::*;
use crate::accounts::LoginKey;
use crate::roles::{Actor, Change};
use crate::{ServerOptions, ServerWorld};

const LOCKABLE: ServerOptions = ServerOptions {
    lockable: true,
    ..default_options()
};

const fn default_options() -> ServerOptions {
    ServerOptions {
        dev_tools: false,
        claims: false,
        exit_when_alone: false,
        guests: true,
        pausable: false,
        online: false,
        player_limit: 0,
        idle_minutes: 0,
        difficulties: shared::difficulty::Difficulties::ALL,
        difficulties_fixed: false,
        lockable: false,
        lockdown: false,
    }
}

/// Logs in to the account `key` (made the Owner first if `owner`), as
/// `options` allow: whether it was let in, and what it was told.
fn log_in(
    connections: &mut Connections,
    key: &'static str,
    owner: bool,
    options: &ServerOptions,
) -> (ClientLink, bool, Vec<ServerMessage>) {
    let id = connections.accounts.log_in(&LoginKey::Test(key)).unwrap();
    if owner {
        let to = RoleId::new(OWNER);
        let change = Change {
            to: &to,
            until: None,
            reason: "",
        };
        let roles = &mut connections.accounts.roles;
        roles.set_role(Actor::Server, id, false, change, 0).unwrap();
        roles.take_changed();
    }
    let (end, link) = local_pair();
    connections.add(link);
    let client = connections.clients.last_mut().unwrap();
    client.phase = Phase::LoggingIn;
    client.makes_characters = true;
    let world = ServerWorld::new(WorldSeed::new(1));
    let let_in = logged_in(
        client,
        id,
        "in a test",
        &mut connections.accounts,
        &world,
        options,
    );
    let told = answers(&end);
    (end, let_in, told)
}

#[test]
fn only_staff_log_in_while_locked_down() {
    let mut connections = Connections::default();
    let locked = ServerOptions {
        lockdown: true,
        ..LOCKABLE
    };
    let (_, let_in, told) = log_in(&mut connections, "player", false, &locked);
    assert!(!let_in);
    assert_eq!(told, [ServerMessage::Refused(LOCKED_DOWN.into())]);
    let (_, let_in, _) = log_in(&mut connections, "owner", true, &locked);
    assert!(let_in);
    assert!(
        connections
            .clients
            .last()
            .unwrap()
            .may(Permission::ServerOptions)
    );
}

#[test]
fn locking_down_sends_the_others_away_and_is_kept_and_noted() {
    let mut connections = Connections::default();
    let (player, ..) = log_in(&mut connections, "player", false, &LOCKABLE);
    let (owner, ..) = log_in(&mut connections, "owner", true, &LOCKABLE);
    connections.clients[1].asked_lockdown = Some(true);
    let mut app = World::new();
    app.insert_resource(connections);
    let world = ServerWorld::new(WorldSeed::new(1));
    app.insert_resource(ServerWorlds::with_home(world));
    app.insert_resource(LOCKABLE);
    app.run_system_once(apply_lockdown).unwrap();

    assert!(app.resource::<ServerOptions>().lockdown);
    assert!(app.resource::<ServerWorlds>().saved_lockdown());
    assert_eq!(answers(&owner), [ServerMessage::Lockdown(true)]);
    assert_eq!(
        answers(&player),
        [ServerMessage::Refused(LOCKED_DOWN.into())]
    );
    let connections = app.resource::<Connections>();
    assert_eq!(connections.clients.len(), 1);
    let noted = connections.accounts.roles.audit_lines();
    assert!(
        noted
            .iter()
            .any(|line| line.contains("locked the server down"))
    );
}

#[test]
fn only_a_lockable_server_tells_its_staff() {
    let mut connections = Connections::default();
    let (owner, ..) = log_in(&mut connections, "owner", true, &LOCKABLE);
    let client = &connections.clients[0];
    assert!(client.send_lockdown(&default_options()));
    assert_eq!(answers(&owner), []);
    assert!(client.send_lockdown(&LOCKABLE));
    assert_eq!(answers(&owner), [ServerMessage::Lockdown(false)]);
}
