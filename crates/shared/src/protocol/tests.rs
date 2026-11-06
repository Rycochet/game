use super::level::{LevelChunk, LevelInfo, RememberedLevel, SeenStairs};
use super::link::{LinkError, local_pair, wired};
use super::login::{CharacterId, CharacterSummary, Credentials, GuestToken, LoginMethod};
use super::world_chunk::GateInfo;
use super::*;
use crate::creature::{CreatureKind, Size};
use crate::dungeon::Cell;
use crate::ground::Load;
use crate::id::Id;
use crate::item::StackLimits;
use crate::item::{Favour, Holder, Inventory, ItemKind, Quality, Rotation};
use crate::mob::EntityLook;
use crate::movement::Step;
use crate::poi::{ChunkPos, PoiKind};
use crate::rng::Seed;
use crate::sheet::{Appearance, Sheet};
use crate::site::PoiLook;
use crate::site::{StairsDirection, StairsStyle};
use crate::world::{ClockRate, WorldTime};

#[test]
fn pings_and_pongs_are_recognised_from_their_bytes() {
    let ping = ClientMessage::ping_bytes(1.5);
    assert_eq!(ClientMessage::read_ping(&ping), Some(1.5));
    let pong = ServerMessage::pong_bytes(1.5, 6.75);
    assert_eq!(ServerMessage::read_pong(&pong), Some((1.5, 6.75)));

    // Whatever else is sent, however long, isn't taken for one.
    let (client, server) = every_message();
    for message in client {
        let bytes = postcard::to_allocvec(&message).unwrap();
        assert_eq!(ClientMessage::read_ping(&bytes), None, "{message:?}");
    }
    for message in server {
        let bytes = postcard::to_allocvec(&message).unwrap();
        let is_pong = matches!(message, ServerMessage::Pong { .. });
        assert_eq!(ServerMessage::read_pong(&bytes).is_some(), is_pong);
    }
}

/// One of every message, so a message that can't be sent fails here.
fn every_message() -> (Vec<ClientMessage>, Vec<ServerMessage>) {
    let client = vec![
        ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            makes_characters: false,
        },
        ClientMessage::Login(Credentials::Guest(GuestToken([7; 32]))),
        ClientMessage::ChooseCharacter {
            id: CharacterId(crate::id::Id::for_test(1)),
        },
        ClientMessage::CreateCharacter {
            name: "Some One".into(),
            kind: CreatureKind::Kobold,
            appearance: Appearance::default(),
            build: crate::sheet::Build::CENTRED,
        },
        ClientMessage::Step {
            map: 2,
            number: 9,
            step: Step::new(-1, 1).unwrap(),
        },
        ClientMessage::EnterSite {
            chunk: ChunkPos { x: 2, y: -1 },
        },
        ClientMessage::UseExit,
        ClientMessage::SetPaused(true),
        ClientMessage::MoveItem {
            item: Id(uuid::Uuid::from_u128(7)),
            to: Holder::Inside(Id(uuid::Uuid::from_u128(8))),
            x: 2,
            y: 3,
            rotation: Rotation::Quarter,
        },
        ClientMessage::StackItem {
            item: Id(uuid::Uuid::from_u128(7)),
            onto: Id(uuid::Uuid::from_u128(8)),
        },
        ClientMessage::SortGrid {
            holder: Holder::Inside(Id(uuid::Uuid::from_u128(8))),
        },
        ClientMessage::FromStack {
            stack: Id(uuid::Uuid::from_u128(7)),
            items: vec![Id(uuid::Uuid::from_u128(8))],
            then: Box::new(ClientMessage::BinItem {
                item: Id(uuid::Uuid::from_u128(8)),
            }),
        },
        ClientMessage::StackFromGround {
            item: Id(uuid::Uuid::from_u128(7)),
            onto: Id(uuid::Uuid::from_u128(8)),
        },
        ClientMessage::BinItem {
            item: Id(uuid::Uuid::from_u128(7)),
        },
        ClientMessage::GiveItem {
            item: Id(uuid::Uuid::from_u128(7)),
            to: Id(uuid::Uuid::from_u128(8)),
        },
        ClientMessage::WearItem {
            item: Id(uuid::Uuid::from_u128(7)),
        },
        ClientMessage::DevGiveItem {
            kind: ItemKind::Pouch,
            size: Some(Size::Large),
            magical: true,
            quality: Quality::Epic,
            set: false,
            favour: Some(Favour::Cursed),
            seed: None,
            count: 1,
            identical: true,
            request: crate::item_form::RollRequest::NONE,
        },
        ClientMessage::OpenTrade {
            with: Id::for_test(3),
        },
        ClientMessage::TakeItem {
            with: Id::for_test(3),
            item: Id::for_test(4),
            to: Holder::Inventory,
            x: 1,
            y: 2,
            rotation: Rotation::Quarter,
        },
        ClientMessage::DropItem {
            item: Id::for_test(5),
        },
        ClientMessage::TakeFromGround {
            item: Id::for_test(6),
            to: Holder::Inventory,
            x: 1,
            y: 2,
            rotation: Rotation::Quarter,
        },
        ClientMessage::WearFromGround {
            item: Id::for_test(7),
        },
        ClientMessage::BinFromGround {
            item: Id::for_test(8),
        },
        ClientMessage::DevSetCreature(CreatureKind::Chicken),
        ClientMessage::DevDetectMonsters(true),
        ClientMessage::DevTruesight(true),
        ClientMessage::DevGodMode(true),
        ClientMessage::DevKillMob {
            mob: Id::for_test(9),
        },
        ClientMessage::DevTeleport { x: -3, y: 4 },
        ClientMessage::DevGoToWorld { seed: 77 },
        ClientMessage::EnterGate { entrance: 5 },
        ClientMessage::DevSetWorldClock {
            minutes: Some(90.5),
            rate: ClockRate::new(3, 2),
        },
        ClientMessage::DevBackToStart,
        ClientMessage::DevToShowcase,
        ClientMessage::DevToHall,
        ClientMessage::DevRevealPois,
        ClientMessage::DevRevealMap,
        ClientMessage::DevForgetMap,
        ClientMessage::DevForgetMapOnServer,
        ClientMessage::DevGenerateSite {
            kind: PoiKind::Castle,
            seed: 9,
            ruin: 2,
        },
        ClientMessage::DevEnterSite {
            kind: PoiKind::Crypt,
            seed: 3,
            level: 1,
            ruin: 2,
        },
    ];
    let server = vec![
        ServerMessage::LoginOptions {
            methods: vec![LoginMethod::Guest],
        },
        ServerMessage::Characters {
            list: vec![CharacterSummary {
                id: CharacterId(crate::id::Id::for_test(1)),
                name: "Some One".into(),
                last_played: 99,
            }],
            can_create: true,
        },
        ServerMessage::Welcome {
            world: WorldInfo {
                id: crate::world::WorldSeed::new(7).id(),
                terrain_seed: Seed::new(8),
                world_map_version: 1,
                gates: vec![GateInfo {
                    entrance: 3,
                    x: 40,
                    y: -12,
                    place: 1_234_567,
                }],
            },
            time: WorldTime::at(5.25, 2.0, ClockRate::NORMAL),
            server_clock: 2.0,
            x: -4,
            y: 9,
            dev_tools: true,
            can_pause: true,
            stacks: StackLimits::default(),
        },
        ServerMessage::ForgetWorldMap,
        ServerMessage::Hour(6),
        ServerMessage::WorldClock(WorldTime::at(405.0, 10.0, ClockRate::NORMAL)),
        ServerMessage::Paused {
            paused: true,
            clock: 405.0,
        },
        ServerMessage::Pong {
            sent: 1.5,
            clock: 6.75,
        },
        ServerMessage::Refused("no".into()),
        ServerMessage::TradeStock {
            with: Id::for_test(3),
            items: Vec::new(),
        },
        ServerMessage::GroundItems {
            x: 1,
            y: -2,
            items: Vec::new(),
        },
        ServerMessage::GroundSeen {
            x: 1,
            y: -2,
            load: Load::Half,
        },
        ServerMessage::GroundGone { x: 1, y: -2 },
        ServerMessage::Keepers(vec![(3, Id::for_test(4))]),
        ServerMessage::WorldChunk(WorldChunk {
            chunk: ChunkPos { x: 1, y: -2 },
            poi: Some(SeenPoi {
                kind: PoiKind::Castle,
                look: PoiLook::Town { buildings: 9 },
                chunk: ChunkPos { x: 1, y: -2 },
                x: 70,
                y: -100,
            }),
        }),
        ServerMessage::WorldChunk(WorldChunk {
            chunk: ChunkPos { x: 0, y: 0 },
            poi: None,
        }),
        ServerMessage::Road(crate::road::Road {
            link: crate::road::RoadLink::new(ChunkPos { x: 2, y: 0 }, ChunkPos { x: 1, y: -1 }),
            kind: crate::road::RoadKind::Track,
            points: vec![(70, -10), (71, -9), (-3, 4)],
        }),
        ServerMessage::EnterLevel(LevelInfo {
            kind: LevelKind::Poi(PoiKind::RockCave),
            ruined: true,
            building: false,
            index: 1,
            levels: 3,
            basement: false,
            bounds: (-20, -10, 40, 30),
            arrival: (4, 5),
            entering_site: true,
        }),
        ServerMessage::RememberedLevel(RememberedLevel {
            index: 2,
            bounds: (-20, -10, 40, 30),
            chunks: vec![LevelChunk {
                chunk: (0, 0),
                mask: 1,
                cells: vec![Cell::Floor],
                stairs: Vec::new(),
                remembered: true,
            }],
        }),
        ServerMessage::Buildings(vec![crate::site::BuildingLook {
            id: 3,
            kind: crate::site::BuildingKind::Bank,
            walls: crate::site::WallKind::Brick,
            roof: crate::site::RoofKind::SlateTiles,
            rects: vec![crate::site::Rect {
                x: 1,
                y: -2,
                width: 3,
                height: 4,
            }],
            door: (2, -2),
            faces: (0, -1),
        }]),
        ServerMessage::Fields(vec![crate::site::FieldLook {
            rect: crate::site::Rect {
                x: -12,
                y: 3,
                width: 8,
                height: 9,
            },
            livestock: true,
            variety: 77,
        }]),
        ServerMessage::LevelCells(LevelChunk {
            chunk: (-1, 2),
            mask: 0b101,
            cells: vec![Cell::Floor, Cell::Door],
            stairs: vec![SeenStairs {
                open_to_sky: false,
                x: -6,
                y: 16,
                direction: StairsDirection::Down,
                descends: (0, 1),
                style: StairsStyle::Spiral,
            }],
            remembered: true,
        }),
        ServerMessage::OnWorldMap { x: 100, y: -4 },
        ServerMessage::EnterHall {
            x: 8,
            y: 0,
            origin: 1 << 32,
        },
        ServerMessage::Teleported { x: 6, y: -7 },
        ServerMessage::Position {
            map: 1,
            step: 30,
            x: -2,
            y: 5,
        },
        ServerMessage::Sheet(Sheet::new(CreatureKind::Horse)),
        ServerMessage::VisitedGates(vec![3, 175_562_835]),
        ServerMessage::EntitySeen(EntityLook {
            id: Id(uuid::Uuid::from_u128(4)),
            kind: CreatureKind::Giant,
            appearance: Appearance::default(),
            feeling: crate::sheet::Feeling::Friendly,
            x: 3,
            y: -4,
            worn: Vec::new(),
        }),
        ServerMessage::EntityMoved {
            id: Id(uuid::Uuid::from_u128(4)),
            x: 4,
            y: -4,
        },
        ServerMessage::EntityGone {
            id: Id(uuid::Uuid::from_u128(4)),
        },
        ServerMessage::Inventory({
            let mut inventory = Inventory::default();
            let pouch = crate::item::Item::new(Id(uuid::Uuid::from_u128(1)), ItemKind::Pouch);
            inventory.give(pouch, StackLimits::NONE).unwrap();
            inventory
        }),
    ];
    (client, server)
}

#[test]
fn every_message_survives_the_link() {
    let (client, server) = local_pair();
    let (to_server, to_client) = every_message();
    for message in &to_server {
        assert!(client.send(message));
        assert_eq!(server.receive(), Ok(Some(message.clone())));
    }
    for message in &to_client {
        assert!(server.send(message));
        assert_eq!(client.receive(), Ok(Some(message.clone())));
    }
    assert_eq!(client.receive(), Ok(None));
}

#[test]
fn a_link_knows_when_the_other_end_has_gone() {
    let (client, server) = local_pair();
    drop(server);
    assert_eq!(client.receive(), Err(LinkError::Closed));
    assert!(!client.send(&ClientMessage::UseExit));
}

#[test]
fn wires_carry_the_bytes_both_ways() {
    // Two wired links joined by their wires, as over a socket.
    let (client, client_wire) = wired::<ClientMessage, ServerMessage>();
    let (server, server_wire) = wired::<ServerMessage, ClientMessage>();
    let (to_server, to_client) = every_message();
    for message in &to_server {
        assert!(client.send(message));
        let bytes = client_wire.take().unwrap().unwrap();
        assert!(server_wire.deliver(bytes));
        assert_eq!(server.receive(), Ok(Some(message.clone())));
    }
    for message in &to_client {
        assert!(server.send(message));
        let bytes = server_wire.take().unwrap().unwrap();
        assert!(client_wire.deliver(bytes));
        assert_eq!(client.receive(), Ok(Some(message.clone())));
    }
    // Garbage from the socket is noticed, not misread.
    assert!(server_wire.deliver(vec![0xff, 0xff, 0xff]));
    assert_eq!(server.receive(), Err(LinkError::Garbled));
    // Either side going is noticed by the other.
    drop(client);
    assert_eq!(client_wire.take(), Err(LinkError::Closed));
    drop(server_wire);
    assert_eq!(server.receive(), Err(LinkError::Closed));
}
