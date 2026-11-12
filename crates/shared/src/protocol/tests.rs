use super::level::{LevelChunk, LevelInfo, RememberedLevel, SeenStairs};
use super::link::{LinkError, local_pair, wired};
use super::login::{CharacterId, Credentials, GuestToken, LoginMethod};
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
        ClientMessage::SetLockdown(true),
        ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            makes_characters: false,
            difficulty: crate::difficulty::Difficulty::Hard,
        },
        ClientMessage::Login(Credentials::Guest(GuestToken([7; 32]))),
        ClientMessage::ChooseCharacter {
            id: CharacterId(crate::id::Id::for_test(1)),
        },
        ClientMessage::DeleteCharacter {
            id: CharacterId(crate::id::Id::for_test(2)),
        },
        ClientMessage::Role(super::RoleAsk::ShowAs(crate::role::RoleId::new(
            crate::role::PLAYER,
        ))),
        ClientMessage::Role(super::RoleAsk::Give {
            character: CharacterId(crate::id::Id::for_test(3)),
            role: crate::role::RoleId::new(crate::role::BANNED),
            minutes: Some(60),
            reason: "spam".into(),
        }),
        ClientMessage::CreateCharacter(super::NewCharacter {
            name: "Some One".into(),
            kind: CreatureKind::Kobold,
            gender: crate::gender::Gender::Female,
            appearance: Appearance::default(),
            build: crate::sheet::Build::CENTRED,
            difficulty: crate::difficulty::Difficulty::Ironman,
        }),
        ClientMessage::SetDifficulty(crate::difficulty::Difficulty::Easy),
        ClientMessage::SetDifficulties(crate::difficulty::Difficulties::only(
            crate::difficulty::Difficulty::Normal,
        )),
        ClientMessage::Step {
            map: 2,
            number: 9,
            step: Step::new(-1, 1).unwrap(),
        },
        ClientMessage::Move {
            map: 2,
            number: 10,
            to: crate::spot::Spot::new(-250, 481),
        },
        ClientMessage::EnterSite {
            chunk: ChunkPos { x: 2, y: -1 },
        },
        ClientMessage::UseExit,
        ClientMessage::Down,
        ClientMessage::Up,
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
        ClientMessage::PartOfStack {
            stack: Id(uuid::Uuid::from_u128(7)),
            count: 40,
            then: Box::new(ClientMessage::DropItem {
                item: Id::stand_in(1),
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
        ClientMessage::DevGiveItem(crate::protocol::DevGive {
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
            to: None,
        }),
        ClientMessage::OpenTrade {
            with: Id::for_test(3),
        },
        ClientMessage::Door(super::DoorAsk::Open { x: -2, y: 5 }),
        ClientMessage::Door(super::DoorAsk::CloseNear),
        ClientMessage::Door(super::DoorAsk::DevAll { open: false }),
        ClientMessage::Face(crate::facing::Heading(40_000)),
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
        ClientMessage::DevSetCreature {
            kind: CreatureKind::Chicken,
            how: crate::protocol::KindChange::Temporary,
        },
        ClientMessage::DevVitals(super::DevVitals::Hurt(4)),
        ClientMessage::DevSkills {
            who: None,
            change: super::DevSkills::LearnSome,
        },
        ClientMessage::ActionBar(crate::ability::BarChange::Swap { a: 0, b: 9 }),
        ClientMessage::DevDetectMonsters(true),
        ClientMessage::DevTruesight(true),
        ClientMessage::DevGodMode(true),
        ClientMessage::DevKillMob {
            mob: Id::for_test(9),
        },
        ClientMessage::LookUpItems(vec![Id::for_test(10), Id::for_test(11)]),
        ClientMessage::DevTeleport { x: -3, y: 4 },
        ClientMessage::DevGoToWorld {
            seed: Some(77),
            gate: Some(3),
        },
        ClientMessage::EnterGate { entrance: 5 },
        ClientMessage::DevSetWorldClock {
            minutes: Some(90.5),
            rate: ClockRate::new(3, 2),
        },
        ClientMessage::DevBackToStart,
        ClientMessage::DevToHall {
            seed: None,
            gate: None,
        },
        ClientMessage::DevLightning,
        ClientMessage::DevRevealPois,
        ClientMessage::DevRevealMap,
        ClientMessage::DevForgetMap,
        ClientMessage::DevForgetMapOnServer,
        ClientMessage::DevGenerateSite {
            kind: PoiKind::Castle,
            seed: 9,
            ruin: 2,
        },
        ClientMessage::DevGenerateArea(crate::site::DevArea {
            size: 64,
            layout: crate::site::DevLayout::Moat,
            mobs: crate::site::DevMobs::Kind {
                kind: crate::creature::CreatureKind::Wolf,
                count: 3,
            },
            seed: 5,
        }),
        ClientMessage::DevMobs(super::DevMobTool::Create(super::DevMob {
            kind: crate::creature::CreatureKind::Goblin,
            x: -4,
            y: 9,
            attitude: Some(crate::sheet::Feeling::Friendly),
            gear: true,
            still: true,
            seed: 12,
        })),
        ClientMessage::DevMobs(super::DevMobTool::Inspect(crate::id::Id::for_test(4))),
        ClientMessage::DevMobs(super::DevMobTool::Freeze(true)),
        ClientMessage::DevEnterSite {
            kind: PoiKind::Crypt,
            seed: 3,
            level: 1,
            ruin: 2,
        },
    ];
    let server = vec![
        ServerMessage::Lockdown(true),
        ServerMessage::DevMobInfo {
            mob: crate::id::Id::for_test(4),
            lines: vec!["Goblin".into(), "Doing: Idle".into()],
        },
        ServerMessage::LoginOptions {
            methods: vec![LoginMethod::Guest],
        },
        ServerMessage::Characters {
            list: vec![ListedCharacter {
                summary: CharacterSummary {
                    id: CharacterId(crate::id::Id::for_test(1)),
                    name: "Some One".into(),
                    last_played: 99,
                    created: 42,
                },
                kind: crate::creature::CreatureKind::default(),
                appearance: crate::sheet::Appearance::default(),
                worn: Vec::new(),
                sheet: crate::sheet::Sheet::default(),
                difficulty: Some(crate::difficulty::Difficulty::Hard),
            }],
            can_create: true,
            difficulties: crate::difficulty::Difficulties::ALL,
        },
        ServerMessage::Killed { permanent: true },
        ServerMessage::Difficulty {
            current: Some(crate::difficulty::Difficulty::Easy),
            allowed: crate::difficulty::Difficulties::ALL,
        },
        ServerMessage::Welcome {
            name: "Brin".into(),
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
            debug_tools: crate::role::DebugTool::ALL.into(),
            can_pause: true,
            stacks: StackLimits::default(),
            role: crate::role::RoleBadge {
                id: crate::role::RoleId::new(crate::role::ADMIN),
                name: "Admin".into(),
                title: Some("Archmage".into()),
            },
            powers: crate::role::Powers {
                permissions: [crate::role::Permission::LookUp].into(),
                grants: [crate::role::RoleId::new(crate::role::MODERATED)].into(),
                appears_as: Default::default(),
            },
            shown: crate::role::ShownRole {
                current: crate::role::RoleId::new(crate::role::PLAYER),
                choices: Vec::new(),
            },
        },
        ServerMessage::Role(super::RoleAnswer::Shown(crate::role::ShownRole::default())),
        ServerMessage::Role(super::RoleAnswer::Found(super::FoundAccount {
            character: CharacterId(crate::id::Id::for_test(3)),
            name: "Brin".into(),
            others: vec!["Bo".into()],
            guest: false,
            yours: false,
            role: crate::role::RoleBadge {
                id: crate::role::RoleId::new(crate::role::BANNED),
                name: "Banned".into(),
                title: None,
            },
            until: Some("2026-11-10 10:00:00 UTC".into()),
            reason: "spam".into(),
            may_give: Vec::new(),
        })),
        ServerMessage::ForgetWorldMap,
        ServerMessage::Hour(6),
        ServerMessage::Lightning {
            flash: true,
            strike: Some(Strike {
                x: -40,
                y: 7,
                seed: 9,
            }),
        },
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
            bags: vec![crate::spot::Spot::new(1, -2)],
            items: Vec::new(),
        },
        ServerMessage::GroundSeen {
            at: crate::spot::Spot::new(1, -2),
            load: Load::Half,
        },
        ServerMessage::GroundGone {
            at: crate::spot::Spot::new(1, -2),
        },
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
            name: Some("Village of Thornbury".into()),
        }),
        ServerMessage::WorldChunk(WorldChunk {
            chunk: ChunkPos { x: 0, y: 0 },
            poi: None,
            name: None,
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
            arrival: crate::spot::Spot::new(4 * 240, 5 * 240 + 17),
            entering_site: true,
            place_name: None,
            ground: Default::default(),
        }),
        ServerMessage::RememberedLevel(RememberedLevel {
            index: 2,
            bounds: (-20, -10, 40, 30),
            chunks: vec![LevelChunk {
                chunk: (0, 0),
                mask: 1,
                cells: vec![Cell::Floor],
                stairs: Vec::new(),
                doors: Vec::new(),
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
            doors: vec![crate::door::DoorLook {
                x: -6,
                y: 16,
                swing: crate::door::Swing::Open(crate::door::Side::North),
                material: crate::material::Material::Wood,
                style: crate::door::DoorStyle::Plain,
                lock: true,
            }],
            remembered: true,
        }),
        ServerMessage::DoorRefused {
            x: 3,
            y: -4,
            locked: true,
        },
        ServerMessage::OnWorldMap { x: 100, y: -4 },
        ServerMessage::WildernessWindow {
            bounds: (-64, 128, 192, 192),
        },
        ServerMessage::EnterHall {
            at: crate::spot::Spot::new(8 * 240, 30),
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
        ServerMessage::Vitals {
            who: None,
            health: crate::vitals::VitalNow {
                value: crate::vitals::Points(3_500),
                most: 20,
                refills_in: 150,
                refill: 3_000,
            },
            mana: crate::vitals::VitalNow::default(),
        },
        ServerMessage::VisitedGates(vec![3, 175_562_835]),
        ServerMessage::EntitySeen(EntityLook {
            id: Id(uuid::Uuid::from_u128(4)),
            kind: CreatureKind::Giant,
            gender: crate::gender::Gender::Male,
            name: Some("Brin".into()),
            appearance: Appearance::default(),
            feeling: crate::sheet::Feeling::Friendly,
            x: 3,
            y: -4,
            worn: Vec::new(),
            mark: Some(crate::role::RoleBadge {
                id: crate::role::RoleId::new(crate::role::MODERATOR),
                name: "Moderator".into(),
                title: None,
            }),
            heading: crate::facing::Heading::WEST,
        }),
        ServerMessage::Moves(vec![crate::protocol::Stretch {
            id: Id(uuid::Uuid::from_u128(4)),
            from: (4, -4),
            to: (1200, -960),
            ticks: 12,
        }]),
        ServerMessage::Turned(vec![(
            Id(uuid::Uuid::from_u128(4)),
            crate::facing::Heading::NORTH,
        )]),
        ServerMessage::EntityGone {
            id: Id(uuid::Uuid::from_u128(4)),
            died: true,
        },
        ServerMessage::ActionBar(crate::ability::ActionBar::default()),
        ServerMessage::ItemsLookedUp {
            found: vec![crate::item::Item::new(Id::for_test(10), ItemKind::Ring)],
            unknown: vec![Id::for_test(11)],
        },
        ServerMessage::Inventory({
            let mut inventory = Inventory::default();
            let pouch = crate::item::Item::new(Id(uuid::Uuid::from_u128(1)), ItemKind::Pouch);
            inventory
                .give(pouch, StackLimits::NONE, &mut crate::id::stand_ins())
                .unwrap();
            // A counted stack: its count goes with it.
            let coins = crate::item::Item {
                count: 37,
                ..crate::item::Item::new(Id::for_test(2), ItemKind::SilverCoin)
            };
            inventory
                .give(coins, StackLimits::NONE, &mut crate::id::stand_ins())
                .unwrap();
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
