//! Prints a few towns' quests as a player would read them, with what the
//! server keeps beside them:
//! `cargo test -p shared print_quests -- --ignored --nocapture`.

use super::test_world::{TestWorld, buildings};
use super::*;
use crate::lang::{Lang, Language};

/// One quest as text: its id and needs, its story, its steps and reward.
pub(super) fn print(lang: &Lang, quest: &Quest) -> String {
    let story = &quest.story;
    let giver = story.cast.giver().person.full_name(lang).unwrap();
    let mut text = format!(
        "== {} [{:?} {:?}] from {giver}, building {}\n",
        story.title(lang).unwrap(),
        quest.kind,
        quest.id.slot,
        quest.building,
    );
    for requirement in &quest.requirements {
        text += &format!("needs {requirement:?}\n");
    }
    for paragraph in story.paragraphs_text(lang).unwrap() {
        text += &format!("{paragraph}\n\n");
    }
    for (index, step) in quest.steps.iter().enumerate() {
        text += &format!(
            "- {}   ({:?}, after {:?}, ways {:?})\n",
            story.step_text(lang, index, 0).unwrap(),
            step.target,
            step.after,
            step.alternatives.iter().map(|a| a.way).collect::<Vec<_>>(),
        );
    }
    text += &format!("- {}\n", story.return_text(lang).unwrap());
    text += &format!("reward {:?}, deeds {:?}\n", quest.reward, quest.deeds);
    text += &format!("> {}\n", story.ending_text(lang, &quest.ending).unwrap());
    text
}

#[test]
#[ignore = "prints quests to read"]
fn print_quests() {
    let world = TestWorld::new();
    let lang = Lang::new(Language::English);
    for town in world.towns(12).into_iter().take(3) {
        let name = (world.view().name_of)(&town).unwrap_or_default();
        println!("######## {name} ({:?} at {:?})\n", town.kind, town.chunk);
        let buildings = buildings(&town);
        for quest in town_quests(&world.view(), &town, &buildings) {
            println!("{}", print(&lang, &quest));
        }
        for building in daily_givers(&town, &buildings) {
            let quest = daily_quest(&world.view(), &town, &buildings, building, 20_000).unwrap();
            println!("{}", print(&lang, &quest));
        }
    }
}
