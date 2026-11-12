//! Sample stories printed as text, as a player would read them (run with
//! `cargo test -p shared sample_stories -- --nocapture` to read them), and a
//! golden of which beats one request tells, so a change in choosing shows.

use super::plot_tests::request;
use super::*;
use crate::lang::{Lang, Language};

/// The whole of a story as text: title, paragraphs, steps, ending.
fn print(lang: &Lang, telling: &Telling) -> String {
    let story = &telling.story;
    let mut text = format!("== {} ({:?})\n", story.title(lang).unwrap(), story.reason);
    for paragraph in story.paragraphs_text(lang).unwrap() {
        text += &format!("{paragraph}\n\n");
    }
    for index in 0..story.facts.asks.len() {
        text += &format!("- {}\n", story.step_text(lang, index, 2).unwrap());
    }
    text += &format!("- {}\n", story.return_text(lang).unwrap());
    text += &format!("> {}\n", story.ending_text(lang, &telling.ending).unwrap());
    text
}

#[test]
fn sample_stories() {
    let lang = Lang::new(Language::English);
    let mut seen = std::collections::BTreeSet::new();
    // The first story told for each reason, and a later step of it.
    for seed in 0..200 {
        let goal = Goal::ALL[seed as usize % Goal::ALL.len()];
        let mut first = request(seed, goal);
        first.continues = true;
        let telling = tell(&first);
        if !seen.insert(telling.story.reason) {
            continue;
        }
        println!("{}", print(&lang, &telling));
        let mut later = first.clone();
        later.text_seed = first.text_seed.derive("step 2");
        later.reason = Some(telling.story.reason);
        later.later_step = true;
        later.continues = false;
        later.carried =
            (telling.story).carry(&[Role::Giver, Role::Relative, Role::Rival], true, true);
        println!("{}", print(&lang, &tell(&later)));
    }
    assert_eq!(seen.len(), Reason::ALL.len());
    // A short (daily) one and a dark one.
    let mut daily = request(7, Goal::Gather);
    daily.length = Length::Short;
    println!("{}", print(&lang, &tell(&daily)));
    let mut dark = request(9, Goal::Find);
    dark.dark = true;
    println!("{}", print(&lang, &tell(&dark)));
}

#[test]
fn golden_beats() {
    let telling = tell(&request(12, Goal::Find));
    let story = &telling.story;
    assert_eq!(story.reason, GOLDEN_REASON);
    assert_eq!(story.paragraphs, GOLDEN_PARAGRAPHS);
}

const GOLDEN_REASON: Reason = Reason::Rumour;
const GOLDEN_PARAGRAPHS: &[&[Beat]] = &[
    &[Beat::RumourHeard],
    &[Beat::WrongPlaceSearched, Beat::FirstTryFailed],
    &[Beat::Retrieve, Beat::Warning, Beat::Reward],
];
