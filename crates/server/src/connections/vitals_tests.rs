//! Health and Mana through a connected client: told on joining, changed by dev
//! tools, saved.

use shared::protocol::{ClientMessage, DEV_BONUS_TICKS, DevVitals, ServerMessage};
use shared::vitals::{ATTRIBUTE_PER_POINT, CALM_AFTER, VitalNow};
use shared::world::WorldSeed;

use super::tests::TestClient;
use crate::ServerWorld;
use crate::storage::{Storage, tests::TestFolder};

/// The last `Vitals` among these answers: Health, then Mana.
fn vitals(answers: &[ServerMessage]) -> (VitalNow, VitalNow) {
    answers
        .iter()
        .rev()
        .find_map(|message| match message {
            ServerMessage::Vitals {
                who: None,
                health,
                mana,
            } => Some((*health, *mana)),
            _ => None,
        })
        .expect("told its vitals")
}

fn dev(what: DevVitals) -> ClientMessage {
    ClientMessage::DevVitals(what)
}

/// The most of each the test character has, from its attributes.
fn most(test: &TestClient) -> (u32, u32) {
    let attributes = &test.client.sheet.attributes;
    (
        u32::from(attributes.toughness) / ATTRIBUTE_PER_POINT,
        u32::from(attributes.will) / ATTRIBUTE_PER_POINT,
    )
}

#[test]
fn told_full_on_joining() {
    let test = TestClient::new();
    let (health, mana) = vitals(&test.welcome);
    let (h, m) = most(&test);
    assert_eq!(
        (health.value.shown(), health.most, health.refills_in),
        (h, h, 0)
    );
    assert_eq!((mana.value.shown(), mana.most), (m, m));
}

#[test]
fn hurt_and_spent_then_filled() {
    let mut test = TestClient::new();
    let (h, m) = most(&test);
    let (health, mana) = vitals(&test.send(&[dev(DevVitals::Hurt(7))]));
    assert_eq!(health.value.shown(), h - 7);
    assert_eq!(u64::from(health.refills_in), CALM_AFTER);
    assert_eq!(mana.value.shown(), m);
    let (_, mana) = vitals(&test.send(&[dev(DevVitals::SpendMana(4))]));
    assert_eq!(mana.value.shown(), m - 4);
    let (health, mana) = vitals(&test.send(&[dev(DevVitals::Fill)]));
    assert_eq!((health.value.shown(), mana.value.shown()), (h, m));
}

#[test]
fn a_bonus_raises_both_until_it_ends() {
    let mut test = TestClient::new();
    let (h, m) = most(&test);
    let (health, mana) = vitals(&test.send(&[dev(DevVitals::Bonus)]));
    assert_eq!((health.most, mana.most), (h + 5, m + 5));
    assert_eq!(health.value.shown(), h + 5);
    test.world.action_clock.advance(DEV_BONUS_TICKS);
    let now = test.world.action_clock.ticks();
    assert!(test.client.keep_vitals(now));
    let told: Vec<_> = std::iter::from_fn(|| test.end.receive().unwrap()).collect();
    let (health, _) = vitals(&told);
    assert_eq!((health.value.shown(), health.most), (h, h));
    assert!(test.client.sheet.vitals.changes.is_empty());
}

#[test]
fn not_told_again_while_nothing_changes() {
    let mut test = TestClient::new();
    let now = test.world.action_clock.ticks();
    assert!(test.client.keep_vitals(now));
    assert!(test.end.receive().unwrap().is_none());
}

#[test]
fn kept_in_the_save() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(open());
    test.send(&[dev(DevVitals::Hurt(9))]);
    assert_eq!(test.client.sheet.vitals.health.practice, 9_000);
    test.leave();
    let again = TestClient::of(open());
    assert_eq!(again.client.sheet.vitals.health.practice, 9_000);
    let (h, _) = most(&again);
    assert_eq!(vitals(&again.welcome).0.value.shown(), h - 9);
}
