use crate::ServerWorld;
use crate::storage::SiteId;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::item::{Inventory, ItemKind, MadeAt, Quality};
use shared::mob::{Event, GIVE_REACH, Memory};
use shared::poi::PoiKind;
use shared::protocol::ServerMessage;
use shared::sheet::{Feeling, Sheet};
use std::collections::BTreeSet;

use shared::protocol::{ClientMessage, local_pair};
use shared::world::WorldSeed;

use super::*;
use crate::connections::tests::{HELLO, TestClient};
use crate::storage::Storage;
use crate::storage::tests::TestFolder;

const CASTLE: ClientMessage = ClientMessage::DevEnterSite {
    kind: PoiKind::Castle,
    seed: 7,
    level: 0,
    ruin: 0,
};

#[test]
fn mobs_only_get_blades_they_can_carry_and_lift() {
    let made_at = MadeAt {
        world: WorldSeed::new(1).id(),
        minute: 0,
    };
    for kind in [CreatureKind::Kobold, CreatureKind::Giant] {
        let lifting = Sheet::new(kind).capacity(kind.def().size) / 2;
        let (mut armed, tries) = (0, 200);
        for _ in 0..tries {
            let gear = [(ItemKind::Sword, Some(Quality::Common))];
            let carrying = dressed(kind, &gear, made_at);
            assert!(carrying.total_weight() <= lifting);
            armed += !carrying.items.is_empty() as u32;
        }
        assert_eq!(armed, tries, "{kind:?} should always find something");
    }
}

#[test]
fn mobs_wear_clothes_made_for_them_and_carry_ordinary_things() {
    use shared::creature::Size;
    let made_at = MadeAt {
        world: WorldSeed::new(1).id(),
        minute: 0,
    };
    let gear = [
        (ItemKind::Trousers, Some(Quality::Common)),
        (ItemKind::Stone, None),
    ];
    // A beggar's trousers are bigger than its inventory.
    let beggar = dressed(CreatureKind::Beggar, &gear, made_at);
    assert_eq!(beggar.worn[0].kind, ItemKind::Trousers);
    let giant = dressed(CreatureKind::Giant, &gear, made_at);
    assert_eq!(giant.worn[0].made.size, Size::Giant);
    assert_eq!(giant.items[0].item.made.size, Size::Medium);
}

fn seen_ids(answers: &[ServerMessage]) -> BTreeSet<Id> {
    answers
        .iter()
        .filter_map(|answer| match answer {
            ServerMessage::EntitySeen(look) => Some(look.id),
            _ => None,
        })
        .collect()
}

/// The id saved for each of the level's mobs.
fn mob_ids(test: &mut TestClient) -> BTreeSet<Id> {
    let id = SiteId::Dev {
        kind: PoiKind::Castle,
        seed: 7,
        ruin: 0,
    };
    let site = test.world.site(id);
    site.levels[&0].mobs.values().copied().collect()
}

#[test]
fn only_mobs_in_sight_are_told_of_and_they_say_nothing_of_stats() {
    let mut test = TestClient::new();
    let answers = test.send(&[CASTLE]);
    let everyone = mob_ids(&mut test);
    assert!(!everyone.is_empty());
    let seen = seen_ids(&answers);
    assert!(seen.is_subset(&everyone));
    // A castle has more rooms than the first one in sight.
    assert!(seen.len() < everyone.len());
}

#[test]
fn walking_shows_what_comes_into_sight_and_says_when_it_goes() {
    // Reveal the level for a route, and walk until something new is seen.
    let mut test = TestClient::new();
    test.send(&[CASTLE]);
    let everyone = mob_ids(&mut test);
    let mob = test.world.populations.0.values().next().unwrap().mobs[0].clone();
    let answers = test.send(&[ClientMessage::DevTeleport {
        x: mob.position.0,
        y: mob.position.1,
    }]);
    // Standing on it, it is in sight: a teleport starts afresh.
    assert!(seen_ids(&answers).contains(&mob.id));
    assert!(seen_ids(&answers).is_subset(&everyone));
    // Nothing has changed: nothing more is said.
    let answers = test.send(&[]);
    assert!(seen_ids(&answers).is_empty());
}

#[test]
fn mobs_keep_their_ids_after_a_restart() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    test.send(&[CASTLE]);
    let before = mob_ids(&mut test);
    let world = test.leave();
    world.storage.finish_writing();
    drop(world);
    let mut test = TestClient::of(open());
    assert_eq!(mob_ids(&mut test), before);
}

#[test]
fn a_forgotten_site_gets_new_mobs() {
    let mut test = TestClient::new();
    test.send(&[CASTLE]);
    let before = mob_ids(&mut test);
    test.send(&[ClientMessage::DevForgetMapOnServer]);
    assert!(mob_ids(&mut test).is_disjoint(&before));
}

/// Two clients of one world, through the real tick.
struct Two {
    connections: crate::Connections,
    world: ServerWorld,
    ends: [shared::protocol::ClientLink; 2],
}

impl Two {
    fn new() -> Self {
        let mut connections = crate::Connections::default();
        let ends = [1, 2].map(|n| {
            let (end, link) = local_pair();
            connections.add_playing(link, crate::CharacterId(Id::for_test(n)));
            end.send(&HELLO);
            end
        });
        let mut two = Self {
            connections,
            world: ServerWorld::new(WorldSeed::new(1)),
            ends,
        };
        two.tick();
        // Each arrives through their own World Gate: bring them together.
        for end in &two.ends {
            end.send(&ClientMessage::DevTeleport { x: 0, y: 0 });
        }
        two.tick();
        two
    }

    fn tick(&mut self) -> [Vec<ServerMessage>; 2] {
        crate::connections::tests::tick_both(&mut self.connections, &mut self.world);
        [0, 1].map(|n| std::iter::from_fn(|| self.ends[n].receive().ok().flatten()).collect())
    }
}

#[test]
fn players_on_the_world_map_see_each_other_and_their_moves() {
    let mut two = Two::new();
    // Both start at the same place: already told by the first tick.
    two.ends[0].send(&ClientMessage::DevTeleport { x: 3, y: 0 });
    let [_, second] = two.tick();
    assert!(
        second
            .iter()
            .any(|answer| matches!(answer, ServerMessage::EntityMoved { x: 3, y: 0, .. }))
    );
}

#[test]
fn a_player_going_into_a_site_goes_out_of_sight_and_is_seen_inside() {
    let mut two = Two::new();
    two.ends[0].send(&CASTLE);
    let [_, second] = two.tick();
    assert!(
        second
            .iter()
            .any(|answer| matches!(answer, ServerMessage::EntityGone { .. }))
    );
    two.ends[1].send(&CASTLE);
    let [first, second] = two.tick();
    // In the same place, they see each other.
    assert!(
            second
                .iter()
                .any(|answer| matches!(answer, ServerMessage::EntitySeen(look) if look.kind == CreatureKind::Human))
        );
    assert!(
            first
                .iter()
                .any(|answer| matches!(answer, ServerMessage::EntitySeen(look) if look.kind == CreatureKind::Human))
        );
}

/// A client in a site, and a mob on its level made this kind, `apart` cells east
/// of the player in the same room, so that they can see each other.
fn beside_a_mob(kind: CreatureKind, apart: i64) -> TestClient {
    let mut test = TestClient::new();
    test.send(&[CASTLE]);
    let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(apart);
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    let population = test.world.populations.0.values_mut().next().unwrap();
    let mob = &mut population.mobs[0];
    mob.kind = kind;
    mob.position = (x + apart, y);
    mob.home = (x + apart, y);
    test
}

/// Lets this many ticks go by, with mobs acting.
fn run(test: &mut TestClient, ticks: u64) {
    for _ in 0..ticks {
        test.world.action_clock.advance(1);
        run_mobs(std::slice::from_mut(&mut test.client), &mut test.world);
    }
}

fn the_mob(test: &TestClient) -> Mob {
    test.world.populations.0.values().next().unwrap().mobs[0].clone()
}

fn player(test: &TestClient) -> (i64, i64) {
    test.client.visit.as_ref().unwrap().position()
}

#[test]
fn mobs_at_rest_need_no_time_until_they_notice_someone() {
    let mut test = TestClient::new();
    test.send(&[CASTLE]);
    // Kinds that ignore the player, wherever they are.
    let population = test.world.populations.0.values_mut().next().unwrap();
    for mob in &mut population.mobs {
        mob.kind = CreatureKind::Dwarf;
    }
    run(&mut test, 20);
    let population = test.world.populations.0.values().next().unwrap();
    assert!(population.due.is_empty());
    assert!(population.is_settled());
}

#[test]
fn something_unfriendly_runs_at_a_player_and_stops_beside_them() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    run(&mut test, 40);
    assert!(cells_apart(the_mob(&test).position, player(&test)) <= ATTACK_GAP);
    assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));
}

#[test]
fn mobs_notice_only_as_far_as_their_kind_sees() {
    let sees = CreatureKind::Kobold.senses().sight_range();
    let mut test = beside_a_mob(CreatureKind::Kobold, sees + 1);
    run(&mut test, 5);
    assert!(matches!(the_mob(&test).doing, Doing::Idle));
    let mut test = beside_a_mob(CreatureKind::Kobold, sees);
    run(&mut test, 5);
    assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));
}

#[test]
fn a_territorial_mob_stands_between_a_stranger_and_its_home() {
    use super::territory::TERRITORY;
    let mut test = beside_a_mob(CreatureKind::Lizardfolk, TERRITORY + 4);
    let home = the_mob(&test).home;
    run(&mut test, 60);
    let mob = the_mob(&test);
    assert!(
        matches!(mob.doing, Doing::Guarding { .. }),
        "{:?}",
        mob.doing
    );
    // At the edge of its territory, towards the player, and neutral still.
    assert_eq!(mob.position, (home.0 - TERRITORY, home.1));
    let kind = test.client.inventory.creature;
    assert_eq!(
        mob.feeling(kind, test.client.character_id.as_ref()),
        Feeling::Neutral
    );
}

#[test]
fn coming_into_its_territory_angers_it_and_it_attacks() {
    let mut test = beside_a_mob(CreatureKind::Lizardfolk, 4);
    run(&mut test, 40);
    let mob = the_mob(&test);
    let kind = test.client.inventory.creature;
    assert_eq!(
        mob.feeling(kind, test.client.character_id.as_ref()),
        Feeling::Unfriendly
    );
    assert!(cells_apart(mob.position, player(&test)) <= ATTACK_GAP);
}

#[test]
fn an_angry_territorial_mob_only_chases_near_its_home() {
    use super::territory::{LEASH, territorial_doing};
    let test = beside_a_mob(CreatureKind::Lizardfolk, 4);
    let mob = the_mob(&test);
    let spot = |x| Spot {
        id: Id::for_test(9),
        character: None,
        kind: CreatureKind::Human,
        position: (mob.home.0 - x, mob.home.1),
        scent: 100,
    };
    let near = territorial_doing(&mob, &spot(LEASH), Feeling::Unfriendly);
    assert!(matches!(near, Some(Doing::Going { .. })));
    let far = territorial_doing(&mob, &spot(LEASH + 1), Feeling::Unfriendly);
    assert!(matches!(far, Some(Doing::Guarding { .. })));
    // Friends are left to its kind's usual welcome.
    assert_eq!(territorial_doing(&mob, &spot(1), Feeling::Friendly), None);
}

#[test]
fn guards_stand_at_the_edge_or_nearer_home() {
    use super::territory::{TERRITORY, guard_spot};
    assert_eq!(guard_spot((0, 0), (20, 0)), (TERRITORY, 0));
    assert_eq!(guard_spot((0, 0), (20, 20)), (TERRITORY, TERRITORY));
    assert_eq!(guard_spot((0, 0), (3, 0)), (2, 0));
    assert_eq!(guard_spot((5, 5), (5, 5)), (5, 5));
}

#[test]
fn something_timid_runs_away() {
    let mut test = beside_a_mob(CreatureKind::Chicken, 2);
    let started = the_mob(&test).position;
    run(&mut test, 10);
    let gap = cells_apart(the_mob(&test).position, player(&test));
    assert!(gap > cells_apart(started, player(&test)));
}

#[test]
fn a_friendly_one_comes_near_and_keeps_its_distance() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    test.send(&[ClientMessage::DevSetCreature {
        kind: CreatureKind::Kobold,
        how: shared::protocol::KindChange::Permanent,
    }]);
    run(&mut test, 40);
    assert_eq!(
        cells_apart(the_mob(&test).position, player(&test)),
        FRIENDLY_GAP
    );
}

#[test]
fn with_nobody_left_a_mob_settles_goes_home_and_the_level_can_go() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    run(&mut test, 10);
    let home = the_mob(&test).home;
    assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));

    // The player leaves for the world map, and the level keeps going.
    test.send(&[ClientMessage::DevBackToStart]);
    let population = test.world.populations.0.values().next().unwrap();
    assert!(!population.is_settled());

    run(&mut test, HOME_WITHIN + 200);
    assert_eq!(the_mob(&test).doing, Doing::Idle);
    assert_eq!(the_mob(&test).position, home);
    let population = test.world.populations.0.values().next().unwrap();
    assert!(population.is_settled());
    assert!(population.due.is_empty());
}

#[test]
fn nothing_moves_while_the_action_clock_stands_still() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    for _ in 0..50 {
        run_mobs(std::slice::from_mut(&mut test.client), &mut test.world);
    }
    // It may take the one step it had the energy for, and no more.
    let moved = cells_apart(the_mob(&test).position, the_mob(&test).home);
    assert!(moved <= 1, "{moved}");
}

#[test]
fn the_token_says_how_the_mob_feels_about_this_player() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    // Told afresh: going away and back would do the same.
    test.client.seen_entities.clear();
    let answers = test.send(&[]);
    let feeling = answers.iter().find_map(|answer| match answer {
        ServerMessage::EntitySeen(look) if look.kind == CreatureKind::Kobold => Some(look.feeling),
        _ => None,
    });
    assert_eq!(feeling, Some(Feeling::Unfriendly));
}

/// `beside_a_mob`, with the player on the level's first stairs and the mob after
/// them, having seen.
fn being_chased_at_stairs(kind: CreatureKind) -> TestClient {
    let mut test = beside_a_mob(kind, 3);
    let visit = test.client.visit.as_ref().unwrap();
    let stairs = visit.first_stairs();
    test.send(&[ClientMessage::DevTeleport {
        x: stairs.0,
        y: stairs.1,
    }]);
    // Beside the stairs, so it sees them taken.
    let population = test.world.populations.0.values_mut().next().unwrap();
    let mob = &mut population.mobs[0];
    mob.position = stairs;
    mob.home = stairs;
    run(&mut test, 2);
    test
}

/// Which level each mob with this id is on.
fn levels_with(test: &TestClient, id: Id) -> Vec<u32> {
    let levels = test.world.populations.0.iter();
    levels
        .filter(|(_, population)| population.mobs.iter().any(|mob| mob.id == id))
        .map(|(&(_, level), _)| level)
        .collect()
}

#[test]
fn a_mob_that_sees_stairs_taken_follows_and_later_goes_home() {
    let mut test = being_chased_at_stairs(CreatureKind::Kobold);
    let mob = the_mob(&test).id;
    assert_eq!(levels_with(&test, mob), [0]);
    test.send(&[ClientMessage::UseExit]);
    let upstairs = test.client.visit.as_ref().unwrap().index();
    run(&mut test, 60);
    assert_eq!(levels_with(&test, mob), [upstairs]);

    // The player leaves. It loses them, wanders, and takes the stairs home;
    // its home level stays in use meanwhile.
    test.send(&[ClientMessage::DevBackToStart]);
    let home_level = test.world.populations.0.keys().find(|(_, l)| *l == 0);
    assert!(home_level.is_some());
    crate::entities::sync(std::slice::from_mut(&mut test.client), &mut test.world, &[]);
    run(&mut test, HOME_WITHIN + 400);
    assert_eq!(levels_with(&test, mob), [0]);
    let home = test.world.populations.0.iter().find(|((_, l), _)| *l == 0);
    let (_, home) = home.unwrap();
    assert_eq!(home.away, 0);
    assert!(home.is_settled());
}

#[test]
fn a_kind_that_cant_use_stairs_stays_behind() {
    let mut test = being_chased_at_stairs(CreatureKind::Giant);
    // (A giant could, so this makes one that can't by its kind.)
    let (after, last_seen) = (test.client.entity_id(), player(&test));
    let population = test.world.populations.0.values_mut().next().unwrap();
    population.mobs[0].kind = CreatureKind::Horse;
    let mob = population.mobs[0].id;
    population.mobs[0].doing = Doing::Going {
        after,
        last_seen,
        gap: ATTACK_GAP,
    };
    test.send(&[ClientMessage::UseExit]);
    run(&mut test, 60);
    assert_eq!(levels_with(&test, mob), [0]);
}

#[test]
fn a_mob_part_way_through_something_carries_on_after_a_restart() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    test.send(&[CASTLE]);
    let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(3);
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    let population = test.world.populations.0.values_mut().next().unwrap();
    let spot = (x + 2, y);
    let settling = Doing::Settling { steps_left: 4 };
    population.mobs[0].position = spot;
    population.mobs[0].doing = settling;
    test.world.save_mobs();
    let world = test.leave();
    world.storage.finish_writing();
    drop(world);

    let test = TestClient::of(open());
    let carried_on = the_mob(&test);
    assert_eq!(carried_on.position, spot);
    assert_eq!(carried_on.doing, settling);
    let population = test.world.populations.0.values().next().unwrap();
    assert!(!population.due.is_empty());
}

/// `beside_a_mob`, with something to give it.
fn with_a_gift(kind: CreatureKind) -> (TestClient, Id) {
    let mut test = beside_a_mob(kind, 2);
    let answers = test.send(&[ClientMessage::DevGiveItem {
        kind: shared::item::ItemKind::Coin,
        size: None,
        magical: false,
        quality: Default::default(),
        set: false,
        favour: None,
        seed: Some(1),
        count: 1,
        identical: true,
        request: shared::item_form::RollRequest::NONE,
    }]);
    let gift = answers.iter().rev().find_map(|answer| match answer {
        ServerMessage::Inventory(carried) => Some(carried.items[0].item.id),
        _ => None,
    });
    (test, gift.expect("told of it"))
}

#[test]
fn a_beggar_given_something_remembers_and_warms_to_the_giver() {
    let (mut test, coin) = with_a_gift(CreatureKind::Beggar);
    let beggar = the_mob(&test).id;
    run(&mut test, 3);
    // Neutral, and nothing to do about it, to begin with.
    assert_eq!(the_mob(&test).doing, Doing::Idle);

    let answers = test.send(&[ClientMessage::GiveItem {
        item: coin,
        to: beggar,
    }]);
    assert!(test.client.inventory.find(coin).is_none());
    // The token changes colour.
    let told = answers.iter().find_map(|answer| match answer {
        ServerMessage::EntitySeen(look) if look.id == beggar => Some(look.feeling),
        _ => None,
    });
    assert_eq!(told, Some(Feeling::Friendly));
    // And it comes along.
    run(&mut test, 3);
    assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));
}

#[test]
fn a_mob_is_dressed_once_and_keeps_its_things_after_a_restart() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    test.send(&[CASTLE]);
    let belongings = |test: &mut TestClient| -> Vec<(Id, Inventory)> {
        let (&(site, level), mobs) = test.world.populations.0.iter().next().unwrap();
        let ids: Vec<Id> = mobs.mobs.iter().map(|mob| mob.id).collect();
        let saved = &test.world.site(site).levels[&level];
        let of = |id: Id| (id, saved.records[&id].gear.clone().unwrap());
        ids.into_iter().map(of).collect()
    };
    let before = belongings(&mut test);
    let wearers = before
        .iter()
        .filter(|(_, carrying)| !carrying.worn.is_empty());
    assert!(wearers.count() > 0, "someone is dressed: {before:?}");
    let armed = before.iter().any(|(_, carrying)| {
        let carried = carrying.items.iter().map(|placed| &placed.item);
        carried
            .chain(&carrying.worn)
            .any(|item| item.made_at.is_some())
    });
    assert!(armed);
    let world = test.leave();
    world.storage.finish_writing();
    drop(world);

    let mut test = TestClient::of(open());
    assert_eq!(belongings(&mut test), before);
}

#[test]
fn what_a_mob_remembers_and_was_given_comes_back_after_a_restart() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    test.send(&[CASTLE]);
    let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(3);
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    let population = test.world.populations.0.values_mut().next().unwrap();
    population.mobs[0].position = (x + 1, y);
    let mob = population.mobs[0].id;
    let answers = test.send(&[ClientMessage::DevGiveItem {
        kind: shared::item::ItemKind::Coin,
        size: None,
        magical: false,
        quality: Default::default(),
        set: false,
        favour: None,
        seed: Some(1),
        count: 1,
        identical: true,
        request: shared::item_form::RollRequest::NONE,
    }]);
    let coin = answers.iter().rev().find_map(|answer| match answer {
        ServerMessage::Inventory(carried) => Some(carried.items[0].item.id),
        _ => None,
    });
    test.send(&[ClientMessage::GiveItem {
        item: coin.unwrap(),
        to: mob,
    }]);
    let world = test.leave();
    world.storage.finish_writing();
    drop(world);

    let mut test = TestClient::of(open());
    let tester = test.client.character_id.unwrap();
    assert_eq!(the_mob(&test).id, mob);
    assert_eq!(the_mob(&test).memory.warmth_for(&tester), 30);
    let id = SiteId::Dev {
        kind: PoiKind::Castle,
        seed: 7,
        ruin: 0,
    };
    let record = &test.world.site(id).levels[&0].records[&mob];
    assert_eq!(record.carried.len(), 1);
}

fn dev_castle() -> SiteId {
    SiteId::Dev {
        kind: PoiKind::Castle,
        seed: 7,
        ruin: 0,
    }
}

/// Stands the player in a room with the level's first mob beside them, carrying
/// something: the mob's id and where it is.
fn next_to_a_laden_mob(test: &mut TestClient) -> (Id, (i64, i64)) {
    test.send(&[CASTLE]);
    let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(3);
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    let population = test.world.populations.0.values_mut().next().unwrap();
    population.mobs[0].position = (x + 1, y);
    let mob = population.mobs[0].id;
    let site = test.world.site_mut(dev_castle());
    let record = site.levels.get_mut(&0).unwrap().records.get_mut(&mob);
    let record = record.unwrap();
    let mut gear = Inventory::default();
    gear.give(
        shared::item::Item::new(Id::for_test(50), ItemKind::Sword),
        shared::item::StackLimits::NONE,
    )
    .unwrap();
    record.gear = Some(gear);
    let given = shared::item::Item::new(Id::for_test(51), ItemKind::Coin);
    record.carried.push(given);
    // So that the player has been told of it.
    test.send(&[]);
    (mob, (x + 1, y))
}

#[test]
fn a_killed_mob_leaves_all_it_had_and_goes_for_good() {
    let mut test = TestClient::new();
    let (mob, at) = next_to_a_laden_mob(&mut test);
    let answers = test.send(&[ClientMessage::DevKillMob { mob }]);
    assert!(
        answers.contains(&ServerMessage::EntityGone { id: mob }),
        "{answers:?}"
    );
    assert!(answers.iter().any(|answer| matches!(
        answer,
        ServerMessage::GroundSeen { x, y, .. } if (*x, *y) == at
    )));

    let level = &test.world.site(dev_castle()).levels[&0];
    let left: Vec<Id> = level.ground[&at].iter().map(|item| item.id).collect();
    assert_eq!(left, [Id::for_test(50), Id::for_test(51)]);
    assert!(!level.records.contains_key(&mob));
    assert!(!level.mobs.values().any(|id| *id == mob));

    // Not when the level is made up again.
    test.world.populations.forget_all();
    test.send(&[CASTLE]);
    let population = test.world.populations.0.values().next().unwrap();
    assert!(!population.mobs.is_empty());
    assert!(population.mobs.iter().all(|one| one.id != mob));
    let level = &test.world.site(dev_castle()).levels[&0];
    assert!(!level.mobs.values().any(|id| *id == mob));
}

#[test]
fn a_mob_stays_dead_after_a_restart() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    let (mob, at) = next_to_a_laden_mob(&mut test);
    test.send(&[ClientMessage::DevKillMob { mob }]);
    let world = test.leave();
    world.storage.finish_writing();
    drop(world);

    let mut test = TestClient::of(open());
    test.send(&[]);
    let mobs = test.world.populations.0.values().next().unwrap();
    assert!(mobs.mobs.iter().all(|one| one.id != mob));
    let level = &test.world.site(dev_castle()).levels[&0];
    assert_eq!(level.ground[&at].len(), 2);
}

#[test]
fn killing_needs_dev_tools_and_sight() {
    let mut test = TestClient::new();
    let (mob, _) = next_to_a_laden_mob(&mut test);
    test.dev_tools = false;
    test.send(&[ClientMessage::DevKillMob { mob }]);
    assert_eq!(the_mob(&test).id, mob, "not allowed");
    test.dev_tools = true;

    let far = test.client.visit.as_mut().unwrap().walkable_out_of_sight();
    let population = test.world.populations.0.values_mut().next().unwrap();
    population.mobs[0].position = far;
    test.send(&[ClientMessage::DevKillMob { mob }]);
    assert_eq!(the_mob(&test).id, mob, "out of sight");
}

#[test]
fn a_mob_killed_away_from_home_is_not_waited_for() {
    let mut test = TestClient::new();
    let (mob, at) = next_to_a_laden_mob(&mut test);
    let population = test.world.populations.0.values_mut().next().unwrap();
    population.away = 1;
    population.mobs[0].way_back.push(Hop {
        stairs: at,
        to_level: 0,
        arrives: at,
    });
    test.send(&[ClientMessage::DevKillMob { mob }]);
    assert_eq!(test.world.populations.0.values().next().unwrap().away, 0);
}

#[test]
fn nothing_can_be_given_from_too_far_or_that_isnt_carried() {
    let (mut test, coin) = with_a_gift(CreatureKind::Beggar);
    let beggar = the_mob(&test).id;
    // Not carried.
    test.send(&[ClientMessage::GiveItem {
        item: beggar,
        to: beggar,
    }]);
    assert_eq!(the_mob(&test).memory, Memory::default());
    // Too far.
    let population = test.world.populations.0.values_mut().next().unwrap();
    population.mobs[0].position.0 += GIVE_REACH + 1;
    test.send(&[ClientMessage::GiveItem {
        item: coin,
        to: beggar,
    }]);
    assert!(test.client.inventory.find(coin).is_some());
    assert_eq!(the_mob(&test).memory, Memory::default());
}

#[test]
fn mobs_never_step_onto_each_other_or_a_player() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    // Two more running at the player from the same place.
    let population = test.world.populations.0.values_mut().next().unwrap();
    let spot = population.mobs[0].position;
    for (index, mob) in population.mobs.iter_mut().enumerate().take(3).skip(1) {
        mob.kind = CreatureKind::Kobold;
        mob.position = (spot.0, spot.1 + index as i64 - 1);
        mob.home = mob.position;
    }
    for _ in 0..40 {
        run(&mut test, 1);
        let population = test.world.populations.0.values().next().unwrap();
        let mut places: Vec<_> = population.mobs.iter().map(|mob| mob.position).collect();
        places.push(player(&test));
        let all = places.len();
        places.sort();
        places.dedup();
        assert_eq!(places.len(), all, "two on one cell");
    }
}

#[test]
fn a_player_swaps_with_someone_friendly_but_not_with_an_enemy() {
    let east = (1, 0);
    let mut test = beside_a_mob(CreatureKind::Horse, 1);
    let from = player(&test);
    test.steps(&[east]);
    assert_eq!(player(&test), (from.0 + 1, from.1));
    assert_eq!(the_mob(&test).position, from);

    let mut test = beside_a_mob(CreatureKind::Kobold, 1);
    let from = player(&test);
    let answers = test.steps(&[east]);
    assert_eq!(player(&test), from);
    assert!(
        answers
            .iter()
            .any(|answer| matches!(answer, ServerMessage::Position { .. }))
    );
    // Given enough, it stops being an enemy and moves over.
    let mob = the_mob(&test).id;
    let tester = test.client.character_id.unwrap();
    let population = test.world.populations.0.values_mut().next().unwrap();
    let event = Event::Gave(shared::item::ItemKind::Coin);
    population.mobs[0].memory.remember(tester, event);
    assert_eq!(the_mob(&test).id, mob);
    test.steps(&[east]);
    assert_eq!(player(&test), (from.0 + 1, from.1));
}

#[test]
fn a_mob_on_the_cell_someone_arrives_on_moves_aside() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    let spot = the_mob(&test).position;
    test.send(&[ClientMessage::DevTeleport {
        x: spot.0,
        y: spot.1,
    }]);
    assert_eq!(player(&test), spot);
    assert_ne!(the_mob(&test).position, spot);
}

#[test]
fn in_god_mode_mobs_take_no_notice() {
    for kind in [
        CreatureKind::Kobold,
        CreatureKind::Chicken,
        CreatureKind::Beggar,
    ] {
        let mut test = beside_a_mob(kind, 3);
        test.send(&[ClientMessage::DevGodMode(true)]);
        let started = the_mob(&test).position;
        run(&mut test, 40);
        assert_eq!(the_mob(&test).doing, Doing::Idle);
        assert_eq!(the_mob(&test).position, started);
    }
}

#[test]
fn a_chase_ends_when_god_mode_is_turned_on() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 3);
    run(&mut test, 40);
    assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));
    test.send(&[ClientMessage::DevGodMode(true)]);
    run(&mut test, 40);
    assert!(!matches!(the_mob(&test).doing, Doing::Going { .. }));
    assert_ne!(the_mob(&test).position, player(&test));
}

#[test]
fn in_god_mode_a_player_walks_through_an_enemy() {
    let east = (1, 0);
    let mut test = beside_a_mob(CreatureKind::Kobold, 1);
    test.send(&[ClientMessage::DevGodMode(true)]);
    let from = player(&test);
    test.steps(&[east]);
    assert_eq!(player(&test), (from.0 + 1, from.1));
    assert_eq!(the_mob(&test).position, from);

    // Turned off, it is in the way again.
    test.send(&[ClientMessage::DevGodMode(false)]);
    let from = player(&test);
    test.steps(&[(-1, 0)]);
    assert_eq!(player(&test), from);
}

#[test]
fn god_mode_needs_dev_tools() {
    let mut test = TestClient::new();
    test.dev_tools = false;
    test.send(&[ClientMessage::DevGodMode(true)]);
    assert!(!test.client.god_mode);
}

#[test]
fn detecting_shows_mobs_out_of_sight() {
    let mut test = TestClient::new();
    let answers = test.send(&[CASTLE]);
    let everyone = mob_ids(&mut test);
    assert!(seen_ids(&answers).len() < everyone.len());
    let answers = test.send(&[ClientMessage::DevDetectMonsters(true)]);
    let found = seen_ids(&answers);
    assert!(!found.is_empty());
    assert_eq!(test.client.seen_entities.len(), everyone.len());
    // Turned off, those out of sight go.
    let answers = test.send(&[ClientMessage::DevDetectMonsters(false)]);
    assert!(
        answers
            .iter()
            .any(|answer| matches!(answer, ServerMessage::EntityGone { .. }))
    );
}

/// A kobold `apart` cells east of the player, tracking someone else whose trail
/// runs west from it to the player's cell, laid at ticks 10 onwards.
fn on_a_trail(apart: i64) -> (TestClient, Id, super::Scents) {
    use super::scent::Went;
    let test = beside_a_mob(CreatureKind::Kobold, apart);
    let start = the_mob(&test).position;
    let who = Id::for_test(77);
    let mut scents = super::Scents::default();
    for step in 0..apart {
        let cell = (start.0 - step, start.1);
        scents.lay(who, cell, Went::Towards(-1, 0), 100, 10 + step as u64);
    }
    (test, who, scents)
}

fn tracking_after(who: Id) -> Doing {
    Doing::Tracking {
        after: who,
        gap: ATTACK_GAP,
        since: 0,
    }
}

#[test]
fn a_nose_follows_a_trail_out_of_sight() {
    use super::tracking::track;
    let (test, who, scents) = on_a_trail(6);
    let mut mob = the_mob(&test);
    mob.doing = tracking_after(who);
    let level = &test.world.populations.0.values().next().unwrap().level;
    let taken = Taken::new();
    for _ in 0..6 {
        track(&mut mob, level, &[], &scents, &taken, 100);
    }
    assert_eq!(mob.position, player(&test));
    // At the end of the trail it gives up.
    track(&mut mob, level, &[], &scents, &taken, 100);
    assert!(
        matches!(mob.doing, Doing::Settling { .. }),
        "{:?}",
        mob.doing
    );
}

#[test]
fn a_trail_too_old_for_the_nose_is_lost() {
    use super::tracking::track;
    use shared::mob::smells_for;
    let (test, who, scents) = on_a_trail(4);
    let mut mob = the_mob(&test);
    mob.doing = tracking_after(who);
    let start = mob.position;
    let level = &test.world.populations.0.values().next().unwrap().level;
    let keen = smells_for(CreatureKind::Kobold.senses().smell);
    // Every mark is older than the nose can smell.
    track(&mut mob, level, &[], &scents, &Taken::new(), 20 + keen);
    assert_eq!(mob.position, start);
    assert!(matches!(mob.doing, Doing::Settling { .. }));
}

#[test]
fn a_trail_up_the_stairs_is_followed_by_those_that_take_stairs() {
    use super::scent::Went;
    use super::tracking::track;
    let test = beside_a_mob(CreatureKind::Kobold, 2);
    let mut mob = the_mob(&test);
    let who = Id::for_test(77);
    let mut scents = super::Scents::default();
    let went = Went::Stairs {
        to_level: 1,
        arrives: (3, 3),
    };
    scents.lay(who, mob.position, went, 100, 5);
    mob.doing = tracking_after(who);
    let level = &test.world.populations.0.values().next().unwrap().level;
    track(&mut mob, level, &[], &scents, &Taken::new(), 10);
    assert!(
        matches!(mob.doing, Doing::Following { to_level: 1, .. }),
        "{:?}",
        mob.doing
    );
}

#[test]
fn seeing_them_again_ends_the_tracking() {
    use super::tracking::track;
    let (test, who, scents) = on_a_trail(3);
    let mut mob = the_mob(&test);
    mob.doing = tracking_after(who);
    let spot = Spot {
        id: who,
        character: None,
        kind: CreatureKind::Human,
        position: player(&test),
        scent: 100,
    };
    let level = &test.world.populations.0.values().next().unwrap().level;
    track(&mut mob, level, &[spot], &scents, &Taken::new(), 20);
    assert!(matches!(mob.doing, Doing::Going { .. }));
}

#[test]
fn players_and_mobs_leave_scent_where_they_step_from() {
    let mut test = beside_a_mob(CreatureKind::Kobold, 6);
    let from = player(&test);
    run(&mut test, 1);
    test.send(&[ClientMessage::DevTeleport {
        x: from.0 + 1,
        y: from.1,
    }]);
    run(&mut test, 1);
    let population = test.world.populations.0.values().next().unwrap();
    let me = test.client.entity_id();
    let mark = population
        .scents
        .smelt(from, me, shared::creature::Acuity::Keen, 2);
    assert!(mark.is_some());
    // The kobold runs at the player, leaving its own.
    run(&mut test, 20);
    let population = test.world.populations.0.values().next().unwrap();
    let kobold = the_mob(&test);
    assert!(!population.scents.is_empty());
    assert_ne!(kobold.position, kobold.home);
    let left = population
        .scents
        .smelt(kobold.home, kobold.id, shared::creature::Acuity::Keen, 30);
    assert!(left.is_some());
}

#[test]
fn old_marks_are_swept_away() {
    use super::scent::Went;
    use shared::mob::SCENT_KEPT_FOR;
    let mut scents = super::Scents::default();
    scents.lay(Id::for_test(1), (0, 0), Went::Towards(1, 0), 200, 0);
    scents.sweep(SCENT_KEPT_FOR - 1);
    assert!(!scents.is_empty());
    scents.sweep(SCENT_KEPT_FOR);
    assert!(scents.is_empty());
}

#[test]
fn something_with_a_nose_tracks_someone_it_loses() {
    use super::tracking::start_tracking;
    let test = beside_a_mob(CreatureKind::Kobold, 2);
    let doing = start_tracking(&the_mob(&test), Id::for_test(1), ATTACK_GAP);
    assert!(matches!(doing, Some(Doing::Tracking { since: 0, .. })));
}
