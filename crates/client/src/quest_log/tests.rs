use shared::hall::GateCode;
use shared::protocol::{PlanetQuests, QuestState};
use shared::world::WorldSeed;

use super::entries::{Group, grouped};
use super::planets::choices;
use super::runes::picture;
use crate::quests::tests::entry;

#[test]
fn quests_are_grouped_under_way_then_done_then_failed_newest_first() {
    let mut first = entry(QuestState::Done, vec![1, 1]);
    first.at = 5;
    let mut second = entry(QuestState::Done, vec![1, 1]);
    second.at = 9;
    let failed = entry(QuestState::Abandoned, vec![0, 0]);
    let taken = entry(QuestState::Ready, vec![1, 1]);
    let all = [first.clone(), failed, second.clone(), taken];
    let groups = grouped(all.iter());
    let order: Vec<Group> = groups.iter().map(|(group, _)| *group).collect();
    assert_eq!(order, Group::ALL);
    let done: Vec<i64> = groups[1].1.iter().map(|entry| entry.at).collect();
    assert_eq!(done, vec![second.at, first.at]);
    assert!(grouped([].iter()).is_empty(), "no groups without quests");
}

#[test]
fn the_planet_the_player_is_on_comes_first() {
    let planet = |seed: u64, here: bool| PlanetQuests {
        world: WorldSeed::new(seed).id(),
        code: GateCode::of_planet(WorldSeed::new(seed)),
        here,
        taken: false,
        ready: false,
    };
    let told = [planet(4, false), planet(5, true), planet(6, false)];
    let order = choices(&told);
    assert_eq!(order, vec![None, Some(told[0].world), Some(told[2].world)]);
    assert_eq!(choices(&[]), vec![None], "here even before the server says");
}

#[test]
fn a_planets_runes_are_drawn() {
    let code = GateCode::of_planet(WorldSeed::new(12345)).unwrap();
    let image = picture(code);
    let size = image.size();
    assert!(size.x > 4 * size.y, "a long strip of runes: {size}");
    let data = image.data.unwrap();
    assert!(data.chunks(4).any(|pixel| pixel[3] == 255), "some ink");
}

#[test]
fn the_button_is_good_markup() {
    super::button::BUTTON.assert_good();
}
