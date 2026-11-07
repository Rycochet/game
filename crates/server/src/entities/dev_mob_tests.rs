//! Create Mob: mobs made to order on the player's level.

use shared::creature::CreatureKind;
use shared::protocol::{ClientMessage, DevMob};
use shared::sheet::Feeling;
use shared::site::{DevArea, DevLayout, DevMobs};

use super::tests::{player, run};
use super::{Doing, Mob};
use crate::connections::tests::TestClient;
use crate::storage::SiteId;

/// A client in an empty 32-cell Dev Area, on its stairs at (1, 16).
fn in_an_area() -> TestClient {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevGenerateArea(DevArea {
        size: 32,
        layout: DevLayout::Empty,
        mobs: DevMobs::None,
        seed: 3,
    })]);
    test
}

fn order(x: i64, y: i64) -> DevMob {
    DevMob {
        kind: CreatureKind::Goblin,
        x,
        y,
        attitude: None,
        gear: true,
        still: false,
        seed: 7,
    }
}

fn mobs(test: &TestClient) -> Vec<Mob> {
    let place = test.client.visit.as_ref().unwrap().population();
    test.world.populations.0[&place].mobs.clone()
}

#[test]
fn a_mob_is_put_where_asked_unless_someone_or_a_wall_is_there() {
    let mut test = in_an_area();
    test.send(&[ClientMessage::DevCreateMob(order(5, 16))]);
    let made = mobs(&test);
    assert_eq!(made.len(), 1);
    assert_eq!(
        (made[0].kind, made[0].position),
        (CreatureKind::Goblin, (5, 16))
    );
    assert!(made[0].is_dev_made());
    // On it again, on the player, into the wall: refused.
    let here = player(&test);
    test.send(&[
        ClientMessage::DevCreateMob(order(5, 16)),
        ClientMessage::DevCreateMob(order(here.0, here.1)),
        ClientMessage::DevCreateMob(order(-1, 16)),
    ]);
    assert_eq!(mobs(&test).len(), 1);
    test.send(&[ClientMessage::DevCreateMob(order(6, 16))]);
    let numbers: Vec<_> = mobs(&test).iter().map(|mob| mob.number).collect();
    assert_ne!(numbers[0], numbers[1]);
}

#[test]
fn gear_can_be_left_off() {
    let mut test = in_an_area();
    let bare = DevMob {
        gear: false,
        ..order(5, 16)
    };
    test.send(&[ClientMessage::DevCreateMob(bare)]);
    assert!(mobs(&test)[0].shown.is_empty());
}

#[test]
fn its_attitude_is_how_it_stands_with_whoever_made_it() {
    let mut test = in_an_area();
    let friendly = DevMob {
        attitude: Some(Feeling::Friendly),
        ..order(5, 16)
    };
    test.send(&[ClientMessage::DevCreateMob(friendly)]);
    let character = test.client.character_id;
    let mob = &mobs(&test)[0];
    assert_eq!(
        mob.feeling(CreatureKind::Human, character.as_ref()),
        Feeling::Friendly
    );
}

#[test]
fn a_still_mob_does_nothing_and_an_angry_one_comes() {
    let mut test = in_an_area();
    let angry = DevMob {
        attitude: Some(Feeling::Unfriendly),
        ..order(5, 16)
    };
    let still = DevMob {
        still: true,
        y: 18,
        ..angry
    };
    test.send(&[
        ClientMessage::DevCreateMob(angry),
        ClientMessage::DevCreateMob(still),
    ]);
    run(&mut test, 60);
    let made = mobs(&test);
    assert_ne!(made[0].position, (5, 16), "the angry one came");
    assert_eq!(made[1].position, (5, 18));
    assert_eq!(made[1].doing, Doing::Idle);
}

#[test]
fn a_made_mob_killed_leaves_nothing_marked_dead() {
    let mut test = in_an_area();
    test.send(&[ClientMessage::DevCreateMob(order(4, 16))]);
    let mob = mobs(&test)[0].id;
    test.send(&[ClientMessage::DevKillMob { mob }]);
    assert!(mobs(&test).is_empty());
    let id = test.client.visit.as_ref().unwrap().population().0;
    assert!(matches!(id, SiteId::DevArea(_)));
    let dead = &test.world.site(id).levels[&0].dead;
    assert!(dead.is_empty());
}

#[test]
fn not_in_the_wilderness() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::Explore]);
    if test.client.visit.is_none() {
        return; // In water here: nothing to try.
    }
    let (x, y) = player(&test);
    let place = test.client.visit.as_ref().unwrap().population();
    let before = test
        .world
        .populations
        .0
        .get(&place)
        .map_or(0, |p| p.mobs.len());
    test.send(&[ClientMessage::DevCreateMob(order(x + 1, y))]);
    let after = test
        .world
        .populations
        .0
        .get(&place)
        .map_or(0, |p| p.mobs.len());
    assert_eq!(before, after);
}
