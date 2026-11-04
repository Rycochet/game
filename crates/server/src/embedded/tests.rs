use std::time::{Duration, Instant};

use shared::protocol::{ClientMessage, PROTOCOL_VERSION, ServerMessage};

use super::*;
use crate::ServerWorld;
use crate::storage::tests::TestFolder;

#[test]
fn an_embedded_server_welcomes_its_client_and_saves_when_it_leaves() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(99);
    let (client, server) = start_embedded(seed, Some(folder.0.clone()));
    assert!(client.send(&ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
    }));
    assert!(client.send(&ClientMessage::DevTeleport { x: 5, y: 6 }));
    let deadline = Instant::now() + Duration::from_secs(10);
    let next_answer = || loop {
        if let Some(message) = client.receive().expect("still connected") {
            break message;
        }
        assert!(Instant::now() < deadline, "no answer from the server");
        thread::sleep(Duration::from_millis(10));
    };
    let world = ServerWorld::new(seed).info();
    // The clock has been running since the server started.
    let ServerMessage::Welcome {
        time, server_clock, ..
    } = next_answer()
    else {
        panic!("no welcome");
    };
    let clock = time.minutes(server_clock);
    assert!((0.0..30.0).contains(&clock), "{clock}");
    let _ = world;
    // Not leaving before the teleport is dealt with: a server still
    // welcoming a client that has gone doesn't read what it sent next.
    let teleported = ServerMessage::Teleported { x: 5, y: 6 };
    while next_answer() != teleported {}

    // Leaving: the server saves and stops, so dropping it returns.
    drop(client);
    drop(server);
    // The one character there is, whatever their id.
    let saved = std::fs::read_dir(folder.0.join("characters"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
    let character = std::fs::read_to_string(saved.path()).unwrap();
    assert!(character.contains("world_position: (5, 6)"), "{character}");
}

#[test]
fn its_player_can_open_it_to_others_and_close_it_again() {
    use shared::protocol::{OnlineSettings, OnlineState};
    use std::net::{TcpListener, TcpStream};

    let folder = TestFolder::new();
    let (client, server) = start_embedded(WorldSeed::new(99), Some(folder.0.clone()));
    assert!(client.send(&ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
    }));
    let deadline = Instant::now() + Duration::from_secs(20);
    let next_online = || loop {
        if let Some(ServerMessage::Online(state)) = client.receive().expect("still connected") {
            break state;
        }
        assert!(Instant::now() < deadline, "no answer from the server");
        thread::sleep(Duration::from_millis(10));
    };
    // A port nothing is using.
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let settings = OnlineSettings {
        bind_address: "127.0.0.1".into(),
        port,
        ..Default::default()
    };
    assert!(client.send(&ClientMessage::SetOnline(Some(settings))));
    let OnlineState::Online { fingerprint, .. } = next_online() else {
        panic!("not online");
    };
    assert_eq!(fingerprint.len(), 64);
    assert!(TcpStream::connect(("127.0.0.1", port)).is_ok());

    assert!(client.send(&ClientMessage::SetOnline(None)));
    assert_eq!(next_online(), OnlineState::Offline);
    assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
    drop(client);
    drop(server);
}
