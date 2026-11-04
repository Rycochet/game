use super::answer::logged_in;
use super::messages::act;
use crate::accounts::{Accounts, LoginKey};
use crate::ground::{self};
use crate::sites::{self};
use crate::{ServerOptions, ServerWorld};
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::item::{Inventory, Item, ItemKind, Quality};
use shared::movement::Step;
use shared::protocol::{CharacterId, ClientMessage, LoginMethod, PROTOCOL_VERSION, ServerMessage};
use shared::sheet::{Appearance, Sheet};
use shared::world::MINUTES_PER_HOUR;
fn tester() -> CharacterId {
    CharacterId(Id::for_test(1000))
}

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use shared::item::{Favour, Holder, Rotation};
use shared::movement::steps_between;
use shared::poi::PoiKind;
use shared::protocol::{ClientLink, Credentials, GuestToken, local_pair};
use shared::world::WorldSeed;

use super::*;
use crate::save_queue::{SaveQueue, tests::MemoryBackend};
use crate::saves::SaveKey;
use crate::storage::{SiteId, SiteSave, Storage, tests::TestFolder};

/// A server world, a client connected to it, and the client's end.
pub(crate) struct TestClient {
    pub world: ServerWorld,
    accounts: Accounts,
    pub client: Client,
    pub end: ClientLink,
    /// The answers to `Hello`.
    pub welcome: Vec<ServerMessage>,
    /// Whether the server allows dev tools (it does, unless set).
    pub dev_tools: bool,
    /// Whether the server pauses when asked (it doesn't, unless set).
    pub pausable: bool,
    /// Whether as much time passes as what is asked takes (it does, unless set).
    pub time_passes: bool,
    next_step: u32,
}

impl TestClient {
    /// A client of a world that isn't saved, welcomed.
    pub fn new() -> Self {
        Self::of(ServerWorld::new(WorldSeed::new(1)))
    }

    /// A client of `world`, welcomed (its answer to `Hello` is read).
    pub fn of(world: ServerWorld) -> Self {
        let (end, server_link) = local_pair();
        let mut test = Self {
            world,
            accounts: Accounts::default(),
            client: Client::new(server_link, Some(tester())),
            end,
            welcome: Vec::new(),
            dev_tools: true,
            pausable: false,
            time_passes: true,
            next_step: 0,
        };
        test.welcome = test.send(&[HELLO]);
        test
    }

    /// Sends messages (in one tick), and returns everything answered
    /// since last time.
    pub fn send(&mut self, messages: &[ClientMessage]) -> Vec<ServerMessage> {
        let options = ServerOptions {
            dev_tools: self.dev_tools,
            exit_when_alone: false,
            pausable: self.pausable,
            guests: true,
            ..Default::default()
        };
        for message in messages {
            answer(
                &mut self.client,
                message.clone(),
                &mut self.world,
                &mut self.accounts,
                &options,
            );
            if self.client.phase == Phase::Joining {
                welcome(&mut self.client, &mut self.world, &options);
            }
            // A long walk in one go shouldn't fill what can wait.
            self.let_time_pass();
        }
        self.let_time_pass();
        self.client.send_position();
        crate::entities::sync(std::slice::from_mut(&mut self.client), &mut self.world);
        ground::sync(std::slice::from_mut(&mut self.client), &mut self.world);
        std::iter::from_fn(|| self.end.receive().unwrap()).collect()
    }

    /// As much time as what was asked takes, unless a test is watching time.
    fn let_time_pass(&mut self) {
        act(&mut self.client, &mut self.world);
        while self.time_passes && !self.client.waiting.is_empty() {
            self.world.action_clock.advance(1);
            act(&mut self.client, &mut self.world);
        }
    }

    /// Takes steps (`(dx, dy)`) on the current map, numbered on from
    /// the last ones, and returns the answers.
    pub fn steps(&mut self, steps: &[(i64, i64)]) -> Vec<ServerMessage> {
        let messages: Vec<_> = steps
            .iter()
            .map(|&(dx, dy)| {
                self.next_step += 1;
                ClientMessage::Step {
                    map: self.client.map,
                    number: self.next_step,
                    step: Step::new(dx, dy).expect("a step"),
                }
            })
            .collect();
        self.send(&messages)
    }

    /// How many maps the player has been put on (`Step::map`).
    pub fn client_map(&self) -> u32 {
        self.client.map
    }

    /// Walks along the steps from where the player is to `to`.
    pub fn walk_to(&mut self, to: (i64, i64)) -> Vec<ServerMessage> {
        let from = match &self.client.visit {
            Some(visit) => visit.position(),
            None => self.client.world_position(),
        };
        let steps: Vec<_> = steps_between(from, to)
            .into_iter()
            .map(|s| (s.dx(), s.dy()))
            .collect();
        self.steps(&steps)
    }

    /// The client leaves: saves it (as `handle_messages` does), and
    /// returns the world.
    pub fn leave(mut self) -> ServerWorld {
        sites::leave(&self.client, &mut self.world);
        self.client.save(&mut self.world);
        self.world.save_sites();
        self.world
    }
}

pub(crate) const HELLO: ClientMessage = ClientMessage::Hello {
    protocol: PROTOCOL_VERSION,
};

const GUEST: ClientMessage = ClientMessage::Login(Credentials::Guest(GuestToken([1; 32])));

fn give(kind: ItemKind) -> ClientMessage {
    ClientMessage::DevGiveItem {
        kind,
        size: None,
        magical: false,
        quality: Quality::Rare,
        set: false,
        favour: None,
    }
}

/// What the last `Inventory` among these answers says.
fn inventory(answers: &[ServerMessage]) -> Inventory {
    answers
        .iter()
        .rev()
        .find_map(|answer| match answer {
            ServerMessage::Inventory(inventory) => Some(inventory.clone()),
            _ => None,
        })
        .expect("an inventory among the answers")
}

#[test]
fn a_character_is_told_what_they_carry_on_joining() {
    let first = inventory(&TestClient::new().welcome);
    assert!(first.items.is_empty());
    let kinds: Vec<_> = first.worn.iter().map(|item| item.kind).collect();
    assert_eq!(
        kinds,
        [ItemKind::TShirt, ItemKind::Trousers, ItemKind::Shoes]
    );
}

#[test]
fn items_move_by_the_rules_and_a_refused_move_is_put_right() {
    let mut test = TestClient::new();
    let answers = test.send(&[give(ItemKind::Pouch)]);
    let pouch = inventory(&answers).items[0].item.id;
    let answers = test.send(&[give(ItemKind::Stone)]);
    let carried = inventory(&answers);
    let stone = carried.items[1].item.id;

    // Too big for the pouch: the answer still says where it really is.
    let refused = ClientMessage::MoveItem {
        item: stone,
        to: Holder::Inside(pouch),
        x: 0,
        y: 0,
        rotation: Rotation::None,
    };
    assert_eq!(inventory(&test.send(&[refused])), carried);

    let answers = test.send(&[ClientMessage::MoveItem {
        item: stone,
        to: Holder::Inventory,
        x: 4,
        y: 2,
        rotation: Rotation::None,
    }]);
    let carried = inventory(&answers);
    let (_, placed) = carried.placed(stone).expect("still carried");
    assert_eq!((placed.x, placed.y), (4, 2));
    assert!(test.client.unsaved);
}

#[test]
fn moves_between_containers_take_time_and_sorting_one_doesnt() {
    let mut test = TestClient::new();
    test.time_passes = false;
    let answers = test.send(&[give(ItemKind::Pouch), give(ItemKind::Coin)]);
    let carried = inventory(&answers);
    let (pouch, coin) = (carried.items[0].item.id, carried.items[1].item.id);
    let tick = test.world.action_clock.ticks();
    let energy = test.client.energy.at(tick, test.client.speed);
    test.send(&[ClientMessage::MoveItem {
        item: coin,
        to: Holder::Inventory,
        x: 5,
        y: 3,
        rotation: Rotation::None,
    }]);
    assert_eq!(test.client.energy.at(tick, test.client.speed), energy);
    test.send(&[ClientMessage::MoveItem {
        item: coin,
        to: Holder::Inside(pouch),
        x: 0,
        y: 0,
        rotation: Rotation::None,
    }]);
    assert!(test.client.energy.at(tick, test.client.speed) < energy);
}

#[test]
fn destroying_takes_what_is_carried_and_only_that() {
    let mut test = TestClient::new();
    let answers = test.send(&[give(ItemKind::Coin)]);
    let coin = inventory(&answers).items[0].item.id;
    let destroy = ClientMessage::BinItem { item: coin };
    assert!(
        inventory(&test.send(std::slice::from_ref(&destroy)))
            .items
            .is_empty()
    );
    assert!(test.client.unsaved);
    // Gone already, so nothing changes, and the client is still told.
    test.client.unsaved = false;
    assert!(inventory(&test.send(&[destroy])).items.is_empty());
    assert!(!test.client.unsaved);
}

#[test]
fn clothes_go_on_by_the_rules_and_come_off_with_a_move() {
    let mut test = TestClient::new();
    let answers = test.send(&[give(ItemKind::FootArmour)]);
    let sabatons = inventory(&answers).items[0].item.id;
    let wear = ClientMessage::WearItem { item: sabatons };
    let worn = inventory(&test.send(std::slice::from_ref(&wear)));
    assert!(worn.is_worn(sabatons));
    assert!(worn.items.is_empty());

    // The shoes are under the sabatons, so a refused move is put right.
    let shoes = worn
        .worn
        .iter()
        .find(|item| item.kind == ItemKind::Shoes)
        .unwrap()
        .id;
    let take_off = |item| ClientMessage::MoveItem {
        item,
        to: Holder::Inventory,
        x: 0,
        y: 0,
        rotation: Rotation::None,
    };
    assert!(inventory(&test.send(&[take_off(shoes)])).is_worn(shoes));
    let answers = test.send(&[take_off(sabatons)]);
    assert!(!inventory(&answers).is_worn(sabatons));
}

#[test]
fn a_new_body_takes_everything_off_and_is_kept() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    let answers = test.send(&[ClientMessage::DevSetCreature(CreatureKind::Kobold)]);
    let changed = inventory(&answers);
    assert_eq!(changed.creature, CreatureKind::Kobold);
    assert!(changed.worn.is_empty());
    assert_eq!(changed.size, (4, 3));
    test.leave();
    // They aren't dressed again on coming back.
    assert_eq!(inventory(&TestClient::of(open()).welcome), changed);
}

#[test]
fn new_items_have_the_quality_asked_for_and_the_time_they_were_made() {
    let mut test = TestClient::new();
    // A dagger, as a long enough sword wouldn't fit in the inventory.
    let answers = test.send(&[give(ItemKind::Dagger), give(ItemKind::Coin)]);
    let carried = inventory(&answers);
    let (sword, coin) = (&carried.items[0].item, &carried.items[1].item);
    assert_eq!(sword.quality, Some(Quality::Rare));
    assert_eq!(coin.quality, None, "a coin has no quality");
    let made = sword.made_at.expect("new things know when they were made");
    assert_eq!(made.world, test.world.seed.id());
    assert_eq!(sword.name, None);
}

#[test]
fn only_things_with_quality_can_be_part_of_a_set() {
    let mut test = TestClient::new();
    let give = |kind| ClientMessage::DevGiveItem {
        kind,
        size: None,
        magical: false,
        quality: Quality::Poor,
        set: true,
        favour: None,
    };
    test.send(&[give(ItemKind::Hat), give(ItemKind::Apple)]);
    let set_of = |kind| {
        let carried = test.client.inventory.items.iter();
        let mut carried = carried.filter(|placed| placed.item.kind == kind);
        carried.next_back().unwrap().item.set
    };
    assert!(set_of(ItemKind::Hat));
    assert!(!set_of(ItemKind::Apple));
}

#[test]
fn a_new_ring_has_a_form_and_fits_its_wearer_and_a_new_gem_rolls_its_size() {
    let mut test = TestClient::new();
    let give = |kind| ClientMessage::DevGiveItem {
        kind,
        size: None,
        magical: false,
        quality: Quality::Rare,
        set: false,
        favour: None,
    };
    let carried = inventory(&test.send(&[give(ItemKind::Ring)]));
    let ring = &carried.items[0].item;
    assert!(ring.form.is_some());
    assert_eq!(ring.made.size, carried.creature.def().size);
    let carried = inventory(&test.send(&[give(ItemKind::Gem)]));
    assert!(
        carried
            .items
            .iter()
            .any(|placed| { placed.item.kind == ItemKind::Gem && placed.item.form.is_some() })
    );
}

#[test]
fn artifacts_are_always_named() {
    let mut test = TestClient::new();
    let artifact = ClientMessage::DevGiveItem {
        kind: ItemKind::Sword,
        size: None,
        magical: false,
        quality: Quality::Artifact,
        set: false,
        favour: None,
    };
    let carried = inventory(&test.send(&[artifact]));
    let name = carried.items[0].item.name();
    assert!(name.ends_with(" Sword") && name != "Sword", "{name}");
}

#[test]
fn items_saved_before_quality_still_load() {
    let saved = ron::to_string(&Item::new(shared::id::Id::for_test(1), ItemKind::Sword)).unwrap();
    let old = saved.replace(",quality:None,made_at:None,name:None", "");
    assert_ne!(old, saved);
    let item: Item = ron::from_str(&old).unwrap();
    assert_eq!((item.quality, item.made_at), (None, None));
}

#[test]
fn items_are_made_for_the_wearer_or_the_size_asked_for() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevSetCreature(CreatureKind::Giant)]);
    test.send(&[ClientMessage::DevTruesight(true)]);
    let answers = test.send(&[give(ItemKind::Gloves)]);
    let gloves = inventory(&answers).items.last().unwrap().item.made;
    assert_eq!(gloves.size, shared::creature::Size::Giant);
    assert_eq!(gloves.fingers, Some(10));
    let large = |kind| ClientMessage::DevGiveItem {
        kind,
        size: Some(shared::creature::Size::Large),
        magical: true,
        quality: Quality::Epic,
        set: false,
        favour: None,
    };
    let mut made = |kind| {
        let answers = test.send(&[large(kind)]);
        inventory(&answers).items.last().unwrap().item.made
    };
    let hat = made(ItemKind::Hat);
    assert_eq!(hat.size, shared::creature::Size::Large);
    assert!(hat.magical);
    // Not something that comes in sizes, whoever asks or carries it.
    let coin = made(ItemKind::Coin);
    assert_eq!(coin.size, shared::creature::Size::Medium);
}

/// Something new in the inventory, with these secrets, and its id.
fn given(
    test: &mut TestClient,
    kind: ItemKind,
    size: Option<shared::creature::Size>,
    magical: bool,
    favour: Option<Favour>,
) -> Id {
    let answers = test.send(&[ClientMessage::DevGiveItem {
        kind,
        size,
        magical,
        quality: Quality::Rare,
        set: false,
        favour,
    }]);
    inventory(&answers).items.last().unwrap().item.id
}

fn secretly_cursed_hat() -> (TestClient, Id) {
    let mut test = TestClient::new();
    let hat = given(&mut test, ItemKind::Hat, None, true, Some(Favour::Cursed));
    test.send(&[ClientMessage::WearItem { item: hat }]);
    (test, hat)
}

#[test]
fn secrets_are_only_sent_to_truesight_or_once_found_out() {
    let (mut test, hat) = secretly_cursed_hat();
    assert!(test.client.inventory.find(hat).unwrap().is_magical());
    let shoes = given(&mut test, ItemKind::Shoes, None, false, None);
    let answers = test.send(&[ClientMessage::BinItem { item: shoes }]);
    let seen = inventory(&answers);
    let item = seen.find(hat).unwrap();
    assert!(!item.made.magical);
    assert_eq!(item.favour, None);

    let seen = inventory(&test.send(&[ClientMessage::DevTruesight(true)]));
    let item = seen.find(hat).unwrap();
    assert!(item.made.magical);
    assert_eq!(item.favour, Some(Favour::Cursed));
    assert!(
        !item.known.magic && !item.known.favour,
        "seen, not found out"
    );

    let seen = inventory(&test.send(&[ClientMessage::DevTruesight(false)]));
    assert_eq!(seen.find(hat).unwrap().favour, None);
}

#[test]
fn trying_to_take_off_a_curse_fails_and_gives_it_away() {
    let attempts = |hat| {
        [
            ClientMessage::MoveItem {
                item: hat,
                to: Holder::Inventory,
                x: 0,
                y: 0,
                rotation: Rotation::None,
            },
            ClientMessage::BinItem { item: hat },
            ClientMessage::GiveItem {
                item: hat,
                to: Id::for_test(9),
            },
        ]
    };
    for index in 0..3 {
        let (mut test, hat) = secretly_cursed_hat();
        let answers = test.send(&[attempts(hat)[index].clone()]);
        let seen = inventory(&answers);
        assert!(seen.is_worn(hat), "attempt {index}");
        let item = seen.find(hat).unwrap();
        assert_eq!(item.favour, Some(Favour::Cursed), "attempt {index}");
        assert!(!item.made.magical, "only the curse is found out");
    }
}

#[test]
fn in_god_mode_a_curse_comes_off_and_stays_secret() {
    let (mut test, hat) = secretly_cursed_hat();
    test.send(&[ClientMessage::DevGodMode(true)]);
    let answers = test.send(&[ClientMessage::BinItem { item: hat }]);
    let seen = inventory(&answers);
    assert!(!seen.is_worn(hat));
    assert_eq!(seen.bin.unwrap().favour, None);
}

#[test]
fn a_blessing_lets_its_wearer_take_it_off_and_stays_secret() {
    let mut test = TestClient::new();
    let hat = given(&mut test, ItemKind::Hat, None, false, Some(Favour::Blessed));
    test.send(&[ClientMessage::WearItem { item: hat }]);
    let seen = inventory(&test.send(&[ClientMessage::BinItem { item: hat }]));
    assert!(seen.in_bin(hat));
    assert_eq!(seen.bin.unwrap().favour, None);
}

#[test]
fn wrong_sized_magic_gives_itself_away_by_fitting() {
    use shared::creature::Size;
    let mut test = TestClient::new();
    let own = given(&mut test, ItemKind::Hat, None, true, None);
    let answers = test.send(&[ClientMessage::WearItem { item: own }]);
    assert!(!inventory(&answers).find(own).unwrap().made.magical);

    // Too small for a medium wearer, were it not magical. Rare reaches 2.
    let mut test = TestClient::new();
    let tiny = given(&mut test, ItemKind::Hat, Some(Size::Tiny), true, None);
    let answers = test.send(&[ClientMessage::WearItem { item: tiny }]);
    let seen = inventory(&answers);
    assert!(seen.is_worn(tiny));
    assert!(seen.find(tiny).unwrap().made.magical);

    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevSetCreature(CreatureKind::Giant)]);
    let far = given(&mut test, ItemKind::Hat, Some(Size::Tiny), true, None);
    let answers = test.send(&[ClientMessage::WearItem { item: far }]);
    assert!(!inventory(&answers).is_worn(far), "past its reach");
}

#[test]
fn asking_for_magic_on_something_plain_raises_its_quality() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevTruesight(true)]);
    let answers = test.send(&[ClientMessage::DevGiveItem {
        kind: ItemKind::Hat,
        size: None,
        magical: true,
        quality: Quality::Common,
        set: false,
        favour: None,
    }]);
    let hat = inventory(&answers).items.last().unwrap().item.clone();
    assert!(hat.is_magical());
    assert_eq!(hat.quality, Some(Quality::Uncommon));
}

#[test]
fn a_character_is_told_their_sheet_and_a_new_body_gives_a_new_one() {
    let sheet = |answers: &[ServerMessage]| {
        answers.iter().rev().find_map(|answer| match answer {
            ServerMessage::Sheet(sheet) => Some(sheet.clone()),
            _ => None,
        })
    };
    let mut test = TestClient::new();
    assert_eq!(sheet(&test.welcome), Some(Sheet::new(CreatureKind::Human)));
    let answers = test.send(&[ClientMessage::DevSetCreature(CreatureKind::Giant)]);
    assert_eq!(sheet(&answers), Some(Sheet::new(CreatureKind::Giant)));
}

#[test]
fn steps_wait_for_energy_in_order_and_time_brings_them() {
    let mut test = TestClient::new();
    enter_a_room(&mut test);
    test.time_passes = false;
    let before = test.client.visit.as_ref().unwrap().position();
    // Three straight steps: the first at once, the others waiting.
    let east = (1, 0);
    test.steps(&[east, east, east]);
    let after_first = (before.0 + 1, before.1);
    assert_eq!(test.client.visit.as_ref().unwrap().position(), after_first);
    assert_eq!(test.client.waiting.len(), 2);

    // A step costs 1⅔ ticks: one more after 2 ticks, and the last after 2 more.
    test.world.action_clock.advance(1);
    test.send(&[]);
    assert_eq!(test.client.waiting.len(), 2);
    test.world.action_clock.advance(1);
    test.send(&[]);
    assert_eq!(test.client.waiting.len(), 1);
    test.world.action_clock.advance(2);
    let answers = test.send(&[]);
    assert!(test.client.waiting.is_empty());
    assert_eq!(
        test.client.visit.as_ref().unwrap().position(),
        (before.0 + 3, before.1)
    );
    assert!(
        answers
            .iter()
            .any(|answer| matches!(answer, ServerMessage::Position { .. }))
    );
}

/// Into a made-up site's wide room, away from walls.
fn enter_a_room(test: &mut TestClient) {
    test.send(&[ClientMessage::DevEnterSite {
        kind: shared::poi::PoiKind::Castle,
        seed: 1,
        level: 0,
        ruin: 0,
    }]);
    // Somewhere with room for three steps east.
    let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(3);
    test.send(&[ClientMessage::DevTeleport { x, y }]);
}

#[test]
fn exits_wait_behind_the_steps_leading_to_them() {
    let mut test = TestClient::new();
    test.time_passes = false;
    // On the world map, where a chunk with a poi is further than one step.
    test.send(&[ClientMessage::DevTeleport { x: 0, y: 0 }]);
    test.client.energy.spend(0, 210, 1000);
    test.send(&[ClientMessage::UseExit]);
    assert_eq!(test.client.waiting.len(), 1);
    assert!(matches!(test.client.waiting[0], Action::UseExit));
}

#[test]
fn too_many_steps_are_dropped_and_the_client_put_right() {
    let mut test = TestClient::new();
    test.time_passes = false;
    let steps = vec![(1, 0); MOST_WAITING + 10];
    let answers = test.steps(&steps);
    assert!(test.client.waiting.len() <= MOST_WAITING);
    assert!(
        answers
            .iter()
            .any(|answer| matches!(answer, ServerMessage::Position { .. }))
    );
}

#[test]
fn heavy_loads_slow_a_character_and_dressing_takes_time() {
    let mut test = TestClient::new();
    let normal = test.client.speed;
    let stones = vec![give(ItemKind::Stone); 5];
    test.send(&stones);
    assert!(test.client.speed < normal);

    let mut test = TestClient::new();
    test.time_passes = false;
    let answers = test.send(&[give(ItemKind::Hat)]);
    let hat = inventory(&answers).items[0].item.id;
    test.send(&[ClientMessage::WearItem { item: hat }]);
    assert!(!test.client.energy.ready(0, test.client.speed));
}

#[test]
fn giving_an_item_needs_dev_tools_and_room() {
    let mut test = TestClient::new();
    test.dev_tools = false;
    let give = give(ItemKind::Coin);
    assert!(test.send(std::slice::from_ref(&give)).is_empty());

    test.dev_tools = true;
    // Coins given apart, so they fill the inventory.
    test.world.stacks = shared::item::StackLimits::NONE;
    let answers = test.send(&vec![give; 30]);
    // A six by four inventory, and no more.
    assert_eq!(inventory(&answers).items.len(), 24);
}

#[test]
fn what_is_carried_comes_back_with_its_ids_after_a_restart() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    let answers = test.send(&[give(ItemKind::Backpack)]);
    let carried = inventory(&answers);
    test.leave();
    assert_eq!(inventory(&TestClient::of(open()).welcome), carried);
}

/// One real tick's messages and sight, in place.
pub(crate) fn tick_both(connections: &mut Connections, world: &mut ServerWorld) {
    let mut app = World::new();
    app.insert_resource(std::mem::take(connections));
    let seed = world.seed;
    app.insert_resource(std::mem::replace(world, ServerWorld::new(seed)));
    app.insert_resource(ServerOptions {
        dev_tools: true,
        exit_when_alone: false,
        pausable: false,
        guests: true,
        ..Default::default()
    });
    bevy::ecs::system::RunSystemOnce::run_system_once(&mut app, handle_messages).unwrap();
    bevy::ecs::system::RunSystemOnce::run_system_once(&mut app, sync_entities).unwrap();
    *connections = app.remove_resource::<Connections>().unwrap();
    *world = app.remove_resource::<ServerWorld>().unwrap();
}

/// Runs `handle_messages` once over these connections.
fn tick(connections: Connections, world: ServerWorld) -> (Connections, ServerWorld) {
    tick_with(connections, world, true)
}

/// `tick`, on a server that does or doesn't take guests.
fn tick_with(
    connections: Connections,
    world: ServerWorld,
    guests: bool,
) -> (Connections, ServerWorld) {
    let mut app = World::new();
    app.insert_resource(connections);
    app.insert_resource(world);
    app.insert_resource(ServerOptions {
        dev_tools: true,
        exit_when_alone: false,
        pausable: false,
        guests,
        ..Default::default()
    });
    bevy::ecs::system::RunSystemOnce::run_system_once(&mut app, handle_messages).unwrap();
    (
        app.remove_resource::<Connections>().unwrap(),
        app.remove_resource::<ServerWorld>().unwrap(),
    )
}

fn answers(end: &ClientLink) -> Vec<ServerMessage> {
    std::iter::from_fn(|| end.receive().ok().flatten()).collect()
}

/// A client that has logged in to an account that isn't a guest's, and been
/// told its characters, with the list.
fn logged_in_to_account(
    connections: &mut Connections,
    key: &'static str,
) -> (ClientLink, Vec<ServerMessage>) {
    let (end, link) = local_pair();
    connections.add(link);
    let client = connections.clients.last_mut().unwrap();
    client.phase = Phase::LoggingIn;
    assert!(logged_in(
        client,
        &LoginKey::Test(key),
        &mut connections.accounts
    ));
    let told = answers(&end);
    (end, told)
}

#[test]
fn logging_in_comes_after_hello_and_before_anything_else() {
    let world = ServerWorld::new(WorldSeed::new(1));
    let mut connections = Connections::default();
    let (too_soon, link) = local_pair();
    connections.add(link);
    too_soon.send(&GUEST);
    let (end, link) = local_pair();
    connections.add(link);
    end.send(&HELLO);
    end.send(&ClientMessage::UseExit);
    let (mut connections, world) = tick(connections, world);
    assert!(matches!(
        answers(&too_soon)[..],
        [ServerMessage::Refused(_)]
    ));
    let told = answers(&end);
    assert_eq!(
        told[0],
        ServerMessage::LoginOptions {
            methods: vec![LoginMethod::Guest]
        }
    );
    assert!(matches!(told[1], ServerMessage::Refused(_)));
    assert!(connections.clients.is_empty());

    // Logged in, a second login is refused.
    let (end, link) = local_pair();
    connections.add(link);
    end.send(&HELLO);
    end.send(&GUEST);
    let (connections, world) = tick(connections, world);
    assert!(matches!(answers(&end)[1], ServerMessage::Welcome { .. }));
    end.send(&GUEST);
    let (connections, _world) = tick(connections, world);
    assert!(matches!(answers(&end)[..], [ServerMessage::Refused(_)]));
    assert!(connections.clients.is_empty());
}

#[test]
fn a_server_can_turn_guests_away() {
    let world = ServerWorld::new(WorldSeed::new(1));
    let mut connections = Connections::default();
    let (end, link) = local_pair();
    connections.add(link);
    end.send(&HELLO);
    end.send(&GUEST);
    let (connections, _world) = tick_with(connections, world, false);
    let told = answers(&end);
    assert_eq!(
        told[0],
        ServerMessage::LoginOptions {
            methods: Vec::new()
        }
    );
    assert!(matches!(told[1], ServerMessage::Refused(_)));
    assert!(connections.clients.is_empty());
}

#[test]
fn a_new_character_is_what_they_asked_if_it_is_allowed() {
    use std::collections::BTreeMap;

    use shared::sheet::Trait;

    let make = |key, kind, appearance| {
        let world = ServerWorld::new(WorldSeed::new(1));
        let mut connections = Connections::default();
        let (end, _) = logged_in_to_account(&mut connections, key);
        end.send(&ClientMessage::CreateCharacter {
            name: "Some One".into(),
            kind,
            appearance,
        });
        let _ = tick(connections, world);
        answers(&end)
    };
    let red = Appearance(BTreeMap::from([(Trait::HairColour, 3)]));

    // Kinds nobody may play, and looks that don't suit them.
    let told = make("a", CreatureKind::Horse, Appearance::default());
    assert!(matches!(told[..], [ServerMessage::Refused(_)]), "{told:?}");
    let told = make("b", CreatureKind::Kobold, red.clone());
    assert!(matches!(told[..], [ServerMessage::Refused(_)]), "{told:?}");

    let told = make("c", CreatureKind::Giant, red.clone());
    let sheet = told.iter().find_map(|answer| match answer {
        ServerMessage::Sheet(sheet) => Some(sheet.clone()),
        _ => None,
    });
    assert_eq!(
        sheet,
        Some(Sheet {
            appearance: red,
            ..Sheet::new(CreatureKind::Giant)
        })
    );
    let carried = inventory(&told);
    assert_eq!(carried.creature, CreatureKind::Giant);
    assert_eq!(carried.size, CreatureKind::Giant.def().inventory);
}

#[test]
fn accounts_choose_or_make_a_character_and_only_their_own() {
    let world = ServerWorld::new(WorldSeed::new(1));
    let mut connections = Connections::default();

    // New: nothing to choose, and room to make one.
    let (end, told) = logged_in_to_account(&mut connections, "someone");
    assert_eq!(
        told,
        [ServerMessage::Characters {
            list: Vec::new(),
            can_create: true
        }]
    );
    end.send(&ClientMessage::CreateCharacter {
        kind: CreatureKind::Human,
        appearance: Appearance::default(),
        name: "Some One".into(),
    });
    let (connections, world) = tick(connections, world);
    assert!(matches!(answers(&end)[0], ServerMessage::Welcome { .. }));
    let mut connections = connections;

    // Back later: there they are. (Dropping the first connection is `join`'s
    // job, when this one chooses.)
    let (second, told) = logged_in_to_account(&mut connections, "someone");
    let [ServerMessage::Characters { list, can_create }] = &told[..] else {
        panic!("no characters: {told:?}");
    };
    assert!(*can_create);
    assert_eq!(list[0].name, "Some One");
    assert!(list[0].last_played > 0);
    let theirs = list[0].id;

    // Someone else's can't be chosen.
    let (other, _) = logged_in_to_account(&mut connections, "someone else");
    other.send(&ClientMessage::ChooseCharacter { id: theirs });
    // Their own can, and an account plays one at a time.
    second.send(&ClientMessage::ChooseCharacter { id: theirs });
    let (connections, _world) = tick(connections, world);
    assert!(matches!(answers(&other)[..], [ServerMessage::Refused(_)]));
    assert!(matches!(answers(&second)[0], ServerMessage::Welcome { .. }));
    assert!(matches!(answers(&end)[..], [ServerMessage::Refused(_)]));
    assert_eq!(connections.clients.len(), 1);
}

#[test]
fn choosing_is_only_for_accounts_about_to_choose() {
    let world = ServerWorld::new(WorldSeed::new(1));
    let mut connections = Connections::default();
    let (end, link) = local_pair();
    connections.add(link);
    end.send(&HELLO);
    end.send(&GUEST);
    let (connections, world) = tick(connections, world);
    answers(&end);
    end.send(&ClientMessage::CreateCharacter {
        kind: CreatureKind::Human,
        appearance: Appearance::default(),
        name: "Another".into(),
    });
    let (connections, _world) = tick(connections, world);
    assert!(matches!(answers(&end)[..], [ServerMessage::Refused(_)]));
    assert!(connections.clients.is_empty());
}

/// A world saved in a backend that can be told to fail, and its queue.
fn world_in_memory() -> (Arc<MemoryBackend>, SaveQueue, ServerWorld) {
    let backend = Arc::new(MemoryBackend::default());
    let queue = SaveQueue::new(backend.clone());
    let seed = WorldSeed::new(1);
    let storage = Storage::new(queue.clone(), seed.id());
    let world = ServerWorld::open(seed, storage).unwrap();
    (backend, queue, world)
}

fn refused(answers: &[ServerMessage]) -> bool {
    answers
        .iter()
        .any(|answer| matches!(answer, ServerMessage::Refused(_)))
}

#[test]
fn a_character_that_cant_be_reached_is_refused_and_not_saved_over() {
    let (backend, queue, world) = world_in_memory();
    let mut test = TestClient::of(world);
    test.send(&[ClientMessage::DevTeleport { x: 5, y: 6 }]);
    let world = test.leave();
    assert!(queue.flush(Duration::from_secs(5)));
    let key = SaveKey::Character(tester());
    let before = backend.holds(&key);
    assert!(before.is_some());

    backend.cant_read.store(true, Ordering::Relaxed);
    let test = TestClient::of(world);
    assert!(refused(&test.welcome));
    let world = test.leave();
    assert!(queue.flush(Duration::from_secs(5)));
    assert_eq!(backend.holds(&key), before);

    // Back to normal: there they are.
    backend.cant_read.store(false, Ordering::Relaxed);
    let test = TestClient::of(world);
    assert!(!refused(&test.welcome));
    assert_eq!(test.client.world_position(), (5, 6));
}

#[test]
fn a_character_saved_while_writes_fail_is_written_later_and_read_back_meanwhile() {
    let (backend, queue, world) = world_in_memory();
    backend.cant_write.store(true, Ordering::Relaxed);
    let mut test = TestClient::of(world);
    test.send(&[ClientMessage::DevTeleport { x: 5, y: 6 }]);
    let world = test.leave();
    let key = SaveKey::Character(tester());
    assert_eq!(backend.holds(&key), None);

    let test = TestClient::of(world);
    assert_eq!(test.client.world_position(), (5, 6));

    backend.cant_write.store(false, Ordering::Relaxed);
    assert!(queue.flush(Duration::from_secs(5)));
    assert!(backend.holds(&key).is_some());
}

#[test]
fn a_site_that_cant_be_reached_is_refused_and_not_saved_over() {
    let (backend, queue, world) = world_in_memory();
    let id = SiteId::Dev {
        kind: PoiKind::Castle,
        seed: 1,
        ruin: 0,
    };
    let key = SaveKey::Site(world.seed.id(), id);
    let saved = SiteSave {
        version: 1,
        ..SiteSave::default()
    };
    world.storage.save_site(id, &saved);
    assert!(queue.flush(Duration::from_secs(5)));
    let before = backend.holds(&key);

    let mut test = TestClient::of(world);
    backend.cant_read.store(true, Ordering::Relaxed);
    let answers = test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: 1,
        level: 0,
        ruin: 0,
    }]);
    assert!(refused(&answers));

    // Asked for anyway, somewhere that can't refuse: blank, and not saved.
    test.world
        .site_mut(id, PoiKind::Castle)
        .levels
        .entry(9)
        .or_default();
    test.world.save_sites();
    assert!(queue.flush(Duration::from_secs(5)));
    assert_eq!(backend.holds(&key), before);

    // Back to normal: the real one again.
    backend.cant_read.store(false, Ordering::Relaxed);
    test.world.open_site(id, PoiKind::Castle).unwrap();
    assert!(test.world.site(id, PoiKind::Castle).levels.is_empty());
}

#[test]
fn guests_come_back_by_their_token_and_the_newest_connection_wins() {
    let folder = crate::storage::tests::TestFolder::new();
    let seed = WorldSeed::new(1);
    let storage = crate::storage::Storage::in_folder(&folder.0, seed.id());
    let world = ServerWorld::open(seed, storage).unwrap();
    let mut connections = Connections::new(Accounts::new(SaveQueue::in_folder(&folder.0)));
    let (first, link) = local_pair();
    connections.add(link);
    first.send(&HELLO);
    first.send(&GUEST);
    let step = |n| ClientMessage::Step {
        map: 0,
        number: n,
        step: Step::new(1, 0).unwrap(),
    };
    let (connections, world) = tick(connections, world);
    assert!(matches!(
        answers(&first)[1],
        ServerMessage::Welcome { x: 0, .. }
    ));
    first.send(&step(1));
    let (mut connections, world) = tick(connections, world);
    answers(&first);

    // The same guest again (a reconnect): the old connection goes, and
    // the character comes back where it was.
    let (second, link) = local_pair();
    connections.add(link);
    second.send(&HELLO);
    second.send(&GUEST);
    let (connections, _world) = tick(connections, world);
    assert!(matches!(answers(&first)[..], [ServerMessage::Refused(_)]));
    assert!(matches!(
        answers(&second)[1],
        ServerMessage::Welcome { x: 1, .. }
    ));
    assert_eq!(connections.clients.len(), 1);
}

#[test]
fn forgetting_a_site_on_the_server_puts_only_others_out_at_the_poi() {
    let mut connections = Connections::default();
    let (first, link) = local_pair();
    connections.add_playing(link, CharacterId(Id::for_test(1)));
    let (second, link) = local_pair();
    connections.add_playing(link, CharacterId(Id::for_test(2)));
    first.send(&HELLO);
    second.send(&HELLO);
    let world = ServerWorld::new(WorldSeed::new(1));
    let (connections, world) = tick(connections, world);
    let enter = ClientMessage::DevEnterSite {
        kind: shared::poi::PoiKind::Castle,
        seed: 5,
        level: 0,
        ruin: 0,
    };
    first.send(&enter);
    second.send(&enter);
    let (connections, world) = tick(connections, world);
    answers(&first);
    answers(&second);

    first.send(&ClientMessage::DevForgetMapOnServer);
    let (connections, _world) = tick(connections, world);
    // The one who asked is still in it, arriving afresh.
    assert!(connections.clients[0].visit.is_some());
    assert!(matches!(answers(&first)[0], ServerMessage::EnterLevel(_)));
    assert!(connections.clients[1].visit.is_none());
    assert!(matches!(
        answers(&second)[0],
        ServerMessage::OnWorldMap { x: 0, y: 0 }
    ));
}

#[test]
fn the_hour_and_moved_clocks_are_told_and_a_restart_carries_on() {
    let folder = crate::storage::tests::TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || {
        let storage = crate::storage::Storage::in_folder(&folder.0, seed.id());
        ServerWorld::open(seed, storage).unwrap()
    };
    let mut connections = Connections::default();
    let (end, link) = local_pair();
    connections.add_playing(link, tester());
    end.send(&HELLO);
    let (connections, mut world) = tick(connections, open());
    answers(&end);

    let tell = |connections: Connections, world: ServerWorld| {
        let mut app = World::new();
        app.insert_resource(connections);
        app.insert_resource(world);
        bevy::ecs::system::RunSystemOnce::run_system_once(&mut app, tell_the_time).unwrap();
        (
            app.remove_resource::<Connections>().unwrap(),
            app.remove_resource::<ServerWorld>().unwrap(),
        )
    };
    // Within an hour says nothing; the next whole hour is told.
    world.hour = 40;
    world.clock.set(40.25 * MINUTES_PER_HOUR);
    let (connections, world) = tell(connections, world);
    assert_eq!(answers(&end), []);
    world.clock.set(41.0 * MINUTES_PER_HOUR);
    let (connections, mut world) = tell(connections, world);
    assert_eq!(answers(&end), [ServerMessage::Hour(41)]);
    // Moved by a dev tool, back as well: told the new time and hour.
    let mut connections = connections;
    let options = ServerOptions {
        dev_tools: true,
        exit_when_alone: false,
        pausable: false,
        guests: true,
        ..Default::default()
    };
    assert!(answer(
        &mut connections.clients[0],
        ClientMessage::DevSetTime { minutes: 1200.0 },
        &mut world,
        &mut connections.accounts,
        &options
    ));
    let (_connections, world) = tell(connections, world);
    let told = answers(&end);
    assert!(
        matches!(told[0], ServerMessage::Clock(minutes) if (1200.0..1210.0).contains(&minutes))
    );
    assert_eq!(told[1], ServerMessage::Hour(20));
    // Saved on stopping, and a restart carries on from there.
    drop(world);
    let restarted = open();
    assert_eq!(restarted.hour, 20);
    assert!((1200.0..1210.0).contains(&restarted.clock.minutes()));
}

#[test]
fn a_pausable_server_stops_its_clock_when_asked() {
    let mut test = TestClient::new();
    test.pausable = true;
    let answers = test.send(&[ClientMessage::SetPaused(true)]);
    let stopped = test.world.clock.minutes();
    assert_eq!(
        answers,
        vec![ServerMessage::Paused {
            paused: true,
            clock: stopped,
        }]
    );
    std::thread::sleep(std::time::Duration::from_millis(5));
    assert_eq!(test.world.clock.minutes(), stopped);

    // Pings are still answered, with the clock where it stopped.
    let answers = test.send(&[ClientMessage::Ping { sent: 1.5 }]);
    assert_eq!(
        answers,
        vec![ServerMessage::Pong {
            sent: 1.5,
            clock: stopped,
        }]
    );

    let answers = test.send(&[ClientMessage::SetPaused(false)]);
    assert!(matches!(
        answers[..],
        [ServerMessage::Paused { paused: false, .. }]
    ));
    std::thread::sleep(std::time::Duration::from_millis(5));
    assert!(test.world.clock.minutes() > stopped);
}

#[test]
fn other_servers_never_pause() {
    // Dev tools being on makes no difference.
    let mut test = TestClient::new();
    let answers = test.send(&[ClientMessage::SetPaused(true)]);
    assert!(answers.is_empty());
    let before = test.world.clock.minutes();
    std::thread::sleep(std::time::Duration::from_millis(5));
    assert!(test.world.clock.minutes() > before);
    assert!(matches!(
        test.welcome[0],
        ServerMessage::Welcome {
            can_pause: false,
            ..
        }
    ));
}

#[test]
fn messages_before_hello_are_refused() {
    let (end, server_link) = local_pair();
    let mut client = Client::new(server_link, Some(tester()));
    let mut world = ServerWorld::new(WorldSeed::new(1));
    let mut accounts = Accounts::default();
    let options = ServerOptions {
        dev_tools: false,
        exit_when_alone: false,
        pausable: false,
        guests: true,
        ..Default::default()
    };
    let message = ClientMessage::UseExit;
    assert!(!answer(
        &mut client,
        message,
        &mut world,
        &mut accounts,
        &options
    ));
    assert!(matches!(end.receive(), Ok(Some(ServerMessage::Refused(_)))));
}

#[test]
fn steps_are_answered_with_the_position_once_a_tick() {
    let mut test = TestClient::new();
    let answers = test.steps(&[(1, 0), (1, 1), (0, 1)]);
    let positions: Vec<_> = answers
        .iter()
        .filter(|a| matches!(a, ServerMessage::Position { .. }))
        .collect();
    assert_eq!(
        positions,
        [&ServerMessage::Position {
            map: 0,
            step: 3,
            x: 2,
            y: 2
        }]
    );
    // Nothing moved, nothing to say.
    assert!(test.send(&[]).is_empty());
}

#[test]
fn steps_meant_for_another_map_are_ignored() {
    let mut test = TestClient::new();
    let stale = ClientMessage::Step {
        map: 7,
        number: 1,
        step: Step::new(1, 0).unwrap(),
    };
    assert!(test.send(&[stale]).is_empty());
    assert_eq!(test.client.world_position(), (0, 0));
}

#[test]
fn things_given_stack_and_wearing_a_stack_puts_on_only_its_top() {
    let mut test = TestClient::new();
    let answers = test.send(&[give(ItemKind::Hat), give(ItemKind::Hat)]);
    let carried = inventory(&answers);
    assert_eq!(carried.items.len(), 1);
    let hats = &carried.items[0].item;
    assert_eq!(hats.count(), 2);
    let (bottom, top) = (hats.id, hats.top().id);
    let worn = inventory(&test.send(&[ClientMessage::WearItem { item: bottom }]));
    assert!(worn.is_worn(top));
    assert_eq!(worn.find(bottom).map(Item::count), Some(1));
}

#[test]
fn a_stack_goes_on_another_as_far_as_the_server_allows() {
    let mut test = TestClient::new();
    // Given apart, then allowed three to a stack.
    test.world.stacks = shared::item::StackLimits::NONE;
    let answers = test.send(&[
        give(ItemKind::Coin),
        give(ItemKind::Coin),
        give(ItemKind::Coin),
    ]);
    let carried = inventory(&answers);
    let coins: Vec<_> = carried.items.iter().map(|placed| placed.item.id).collect();
    test.world.stacks = shared::item::StackLimits {
        grams_per_cell: 2000,
        most: 2,
    };
    let stack = |item, onto| ClientMessage::StackItem { item, onto };
    let carried = inventory(&test.send(&[stack(coins[1], coins[0])]));
    assert_eq!(carried.items.len(), 2);
    assert_eq!(carried.find(coins[0]).map(Item::count), Some(2));
    // Full: refused, and the client is put right.
    let carried = inventory(&test.send(&[stack(coins[2], coins[0])]));
    assert_eq!(carried.find(coins[2]).map(Item::count), Some(1));
    assert_eq!(carried.find(coins[0]).map(Item::count), Some(2));
}

#[test]
fn part_of_a_stack_moves_on_its_own_and_goes_back_if_refused() {
    let mut test = TestClient::new();
    let answers = test.send(&[
        give(ItemKind::Coin),
        give(ItemKind::Coin),
        give(ItemKind::Coin),
    ]);
    let coins = inventory(&answers).items[0].item.clone();
    let part = coins.above[0].id;
    let move_to = |x| ClientMessage::MoveItem {
        item: part,
        to: Holder::Inventory,
        x,
        y: 0,
        rotation: Rotation::None,
    };
    // Onto its own stack's place: refused, and nothing changes.
    let carried = inventory(&test.send(&[move_to(0)]));
    assert_eq!(carried.items.len(), 1);
    assert_eq!(carried.find(coins.id).map(Item::count), Some(3));
    // Elsewhere: the top two go.
    let carried = inventory(&test.send(&[move_to(4)]));
    assert_eq!(carried.find(coins.id).map(Item::count), Some(1));
    assert_eq!(carried.find(part).map(Item::count), Some(2));
}

#[test]
fn a_group_from_a_mixed_stack_moves_together() {
    let mut test = TestClient::new();
    let answers = test.send(&[
        give(ItemKind::Hat),
        give(ItemKind::Hat),
        give(ItemKind::Hat),
    ]);
    let hats = inventory(&answers).items[0].item.clone();
    let cursed = hats.above[0].id;
    {
        let stack = &mut test.client.inventory.items[0].item;
        stack.above[0].favour = Some(Favour::Cursed);
        stack.above[0].known.favour = true;
    }
    let plain = vec![hats.id, hats.above[1].id];
    let from_stack = |x| ClientMessage::FromStack {
        stack: hats.id,
        items: plain.clone(),
        then: Box::new(ClientMessage::MoveItem {
            item: hats.id,
            to: Holder::Inventory,
            x,
            y: 0,
            rotation: Rotation::None,
        }),
    };
    // Back where it was: refused, and it goes back on the stack.
    let carried = inventory(&test.send(&[from_stack(0)]));
    assert_eq!(carried.items.len(), 1);
    assert_eq!(carried.items[0].item.count(), 3);
    // Elsewhere: the plain two go, and the cursed one is left alone.
    let bottom = carried.items[0].item.id;
    let from_stack = ClientMessage::FromStack {
        stack: bottom,
        items: plain.clone(),
        then: Box::new(ClientMessage::MoveItem {
            item: hats.id,
            to: Holder::Inventory,
            x: 3,
            y: 0,
            rotation: Rotation::None,
        }),
    };
    let carried = inventory(&test.send(&[from_stack]));
    assert_eq!(carried.find(hats.id).map(Item::count), Some(2));
    assert_eq!(carried.find(cursed).map(Item::count), Some(1));
}

#[test]
fn a_grid_is_sorted_as_the_client_would_sort_it() {
    let mut test = TestClient::new();
    let answers = test.send(&[give(ItemKind::Coin), give(ItemKind::Stone)]);
    let carried = inventory(&answers);
    let stone = carried
        .items
        .iter()
        .find(|placed| placed.item.kind == ItemKind::Stone)
        .map(|placed| placed.item.id)
        .unwrap();
    // Into the far corner, so that sorting moves it.
    let (width, height) = carried.size;
    let to_corner = ClientMessage::MoveItem {
        item: stone,
        to: Holder::Inventory,
        x: width - 2,
        y: height - 2,
        rotation: Rotation::None,
    };
    let before = inventory(&test.send(&[to_corner]));
    let expected = before
        .sorted_grid(Holder::Inventory, test.world.stacks)
        .unwrap();
    assert_ne!(before.items, expected);
    let sort = ClientMessage::SortGrid {
        holder: Holder::Inventory,
    };
    let after = inventory(&test.send(&[sort]));
    assert_eq!(after.items, expected);
}
