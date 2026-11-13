use crate::ServerWorlds;
use shared::protocol::{
    ClientLink, ClientMessage, Credentials, EmailAddress, GuestToken, LoginMethod, ServerMessage,
    SessionToken, local_pair,
};
use shared::world::WorldSeed;

use super::tests::{MAKES, answers};
use super::*;
use crate::oidc::Logins;
use crate::oidc::tests::{FakeProvider, come_back};
use crate::{ServerOptions, ServerWorld};

/// A server's connections and world, with logins through `logins`.
struct Server {
    connections: Option<Connections>,
    world: Option<ServerWorld>,
    logins: Option<Logins>,
}

impl Server {
    fn new(logins: Option<Logins>) -> Self {
        Self {
            connections: Some(Connections::default()),
            world: Some(ServerWorld::new(WorldSeed::new(1))),
            logins,
        }
    }

    /// A client, said hello, and the answer.
    fn connect(&mut self) -> (ClientLink, Vec<ServerMessage>) {
        let (end, link) = local_pair();
        self.connections.as_mut().unwrap().add(link);
        end.send(&MAKES);
        self.tick();
        let told = answers(&end);
        (end, told)
    }

    /// Finishes logins and answers messages, as a tick does.
    fn tick(&mut self) {
        let mut app = World::new();
        app.insert_resource(self.connections.take().unwrap());
        app.insert_resource(ServerWorlds::with_home(self.world.take().unwrap()));
        app.insert_resource(ServerOptions {
            guests: true,
            ..Default::default()
        });
        if let Some(logins) = &self.logins {
            app.insert_resource(logins.clone());
        }
        use bevy::ecs::system::RunSystemOnce;
        app.run_system_once(finish_logins).unwrap();
        app.run_system_once(handle_messages).unwrap();
        self.connections = app.remove_resource::<Connections>();
        self.world = Some(app.remove_resource::<ServerWorlds>().unwrap().into_home());
    }

    /// Sends a message and ticks: what is answered.
    fn send(&mut self, end: &ClientLink, message: ClientMessage) -> Vec<ServerMessage> {
        end.send(&message);
        self.tick();
        answers(end)
    }

    fn connected(&self) -> usize {
        self.connections.as_ref().unwrap().clients.len()
    }
}

/// Logs in through the fake provider, coming back with `code`: what the
/// client is told once the login is finished.
fn through_provider(server: &mut Server, end: &ClientLink, code: &str) -> Vec<ServerMessage> {
    let told = server.send(
        end,
        ClientMessage::Login(Credentials::Provider("fake".into())),
    );
    let [ServerMessage::OpenLogin { url }] = &told[..] else {
        panic!("{told:?}");
    };
    come_back(server.logins.as_ref().unwrap(), url, code);
    server.tick();
    answers(end)
}

#[test]
fn a_player_logs_in_through_a_provider_then_with_the_session() {
    let provider = FakeProvider::start();
    let mut server = Server::new(Some(provider.logins("secret")));
    let (end, told) = server.connect();
    let fake = LoginMethod::Provider {
        name: "fake".into(),
        label: "Fake".into(),
    };
    let methods = vec![LoginMethod::Guest, fake];
    assert_eq!(told, [ServerMessage::LoginOptions { methods }]);

    let told = through_provider(&mut server, &end, "good");
    let [
        ServerMessage::LoggedIn { session, email },
        ServerMessage::Characters {
            list, can_create, ..
        },
    ] = &told[..]
    else {
        panic!("{told:?}");
    };
    assert_eq!(email, &EmailAddress(" Someone@Example.com".into()));
    assert!(list.is_empty() && *can_create);
    let account = server.connections.as_ref().unwrap().clients[0].account;
    assert!(account.is_some());

    // Next time, with the session, to the same account.
    let (again, _) = server.connect();
    let told = server.send(&again, ClientMessage::Login(Credentials::Session(*session)));
    assert!(
        matches!(told[..], [ServerMessage::Characters { .. }]),
        "{told:?}"
    );
    let clients = &server.connections.as_ref().unwrap().clients;
    assert_eq!(clients.last().unwrap().account, account);
}

#[test]
fn the_same_address_reaches_the_same_account_on_servers_sharing_the_secret() {
    let provider = FakeProvider::start();
    let account = |secret: &str| {
        let mut server = Server::new(Some(provider.logins(secret)));
        let (end, _) = server.connect();
        through_provider(&mut server, &end, "good");
        server.connections.as_ref().unwrap().clients[0]
            .account
            .unwrap()
    };
    assert_eq!(account("shared"), account("shared"));
    assert_ne!(account("shared"), account("another"));
}

#[test]
fn a_failed_login_can_be_tried_again_or_another_way() {
    let provider = FakeProvider::start();
    let mut server = Server::new(Some(provider.logins("secret")));
    let (end, _) = server.connect();
    let told = through_provider(&mut server, &end, "unverified");
    assert!(
        matches!(told[..], [ServerMessage::LoginFailed(_)]),
        "{told:?}"
    );
    let unknown = Credentials::Session(SessionToken([9; 32]));
    let told = server.send(&end, ClientMessage::Login(unknown));
    assert!(
        matches!(told[..], [ServerMessage::LoginFailed(_)]),
        "{told:?}"
    );
    let nowhere = Credentials::Provider("nowhere".into());
    let told = server.send(&end, ClientMessage::Login(nowhere));
    assert!(
        matches!(told[..], [ServerMessage::LoginFailed(_)]),
        "{told:?}"
    );
    assert_eq!(server.connected(), 1);
    let told = server.send(
        &end,
        ClientMessage::Login(Credentials::Guest(GuestToken([2; 32]))),
    );
    assert!(
        matches!(told[..], [ServerMessage::Characters { .. }]),
        "{told:?}"
    );
}

#[test]
fn without_providers_only_guests_log_in() {
    let mut server = Server::new(None);
    let (end, told) = server.connect();
    let methods = vec![LoginMethod::Guest];
    assert_eq!(told, [ServerMessage::LoginOptions { methods }]);
    for credentials in [
        Credentials::Session(SessionToken([9; 32])),
        Credentials::Provider("google".into()),
    ] {
        let told = server.send(&end, ClientMessage::Login(credentials));
        assert!(
            matches!(told[..], [ServerMessage::LoginFailed(_)]),
            "{told:?}"
        );
    }
    assert_eq!(server.connected(), 1);
}

fn named(name: &str) -> ClientMessage {
    ClientMessage::CreateCharacter(shared::protocol::NewCharacter {
        name: name.into(),
        kind: shared::creature::CreatureKind::Human,
        gender: shared::gender::Gender::Neutral,
        appearance: Default::default(),
        build: Default::default(),
        difficulty: shared::difficulty::Difficulty::Normal,
    })
}

#[test]
fn the_claim_code_as_a_name_makes_the_account_the_owner() {
    let provider = FakeProvider::start();
    let mut server = Server::new(Some(provider.logins("secret")));
    let roles = &mut server.connections.as_mut().unwrap().accounts.roles;
    let code = roles.start_claim().unwrap().unwrap();

    // Not a guest's.
    let (guest, _) = server.connect();
    server.send(
        &guest,
        ClientMessage::Login(Credentials::Guest(GuestToken([3; 32]))),
    );
    let told = server.send(&guest, named(&code));
    assert!(
        matches!(&told[..], [ServerMessage::CharacterRefused(why)] if why.contains("guest")),
        "{told:?}"
    );

    let (end, _) = server.connect();
    through_provider(&mut server, &end, "good");
    let clients = &server.connections.as_ref().unwrap().clients;
    let account = clients.last().unwrap().account.unwrap();
    let told = server.send(&end, named(&code));
    assert_eq!(
        told,
        [ServerMessage::Refused(super::new_character::CLAIMED.into())]
    );
    let connections = server.connections.as_mut().unwrap();
    assert!(
        connections
            .accounts
            .account(&account)
            .unwrap()
            .characters
            .is_empty()
    );
    let held = connections.accounts.roles.held(account, false, 0);
    assert!(held.badge.id.is(shared::role::OWNER));
}

#[test]
fn a_banned_account_is_turned_away_and_told_why() {
    let provider = FakeProvider::start();
    let mut server = Server::new(Some(provider.logins("secret")));
    let (end, _) = server.connect();
    let told = through_provider(&mut server, &end, "good");
    let ServerMessage::LoggedIn { session, .. } = told[0] else {
        panic!("{told:?}");
    };
    let connections = server.connections.as_mut().unwrap();
    let account = connections.clients[0].account.unwrap();
    let banned = shared::role::RoleId::new(shared::role::BANNED);
    let change = crate::roles::Change {
        to: &banned,
        until: None,
        reason: "cheating",
    };
    let roles = &mut connections.accounts.roles;
    roles
        .set_role(crate::roles::Actor::Server, account, false, change, 0)
        .unwrap();

    let (again, _) = server.connect();
    let told = server.send(&again, ClientMessage::Login(Credentials::Session(session)));
    assert_eq!(
        told,
        [ServerMessage::Refused(
            "this account is banned: cheating".into()
        )]
    );
}
