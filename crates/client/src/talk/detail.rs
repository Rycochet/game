//! One quest in full, as words: on offer (its story, what to do, what it pays,
//! Accept and Decline), under way or done (how far along, Complete when every
//! step is done), or just handed in (the giver's last words).

use shared::lang::{Args, Lang};
use shared::protocol::{LogEntry, OfferState, QuestState};
use shared::story::{RewardPart, Story, reward_text};

use super::OpenTalk;
use super::words::{or_partly, plain};
use crate::quests::{QuestBook, done_steps};

/// What the detail shows, and which buttons.
pub(super) struct Detail {
    pub(super) title: String,
    pub(super) paragraphs: Vec<String>,
    pub(super) steps_head: String,
    pub(super) steps: Vec<String>,
    pub(super) reward: String,
    /// On offer: Accept and Decline (else Back).
    pub(super) accept: bool,
    /// Every step done: Complete.
    pub(super) turn_in: bool,
}

/// The quest a panel shows in full, if it shows one.
pub(super) fn detail_of(talk: &OpenTalk, book: &QuestBook, lang: &Lang) -> Option<Detail> {
    if let Some((id, ending)) = &talk.ended {
        let entry = book.entries.get(id)?;
        return Some(Detail {
            title: or_partly(entry.story.title(lang)),
            paragraphs: vec![or_partly(entry.story.ending_text(lang, ending))],
            steps_head: String::new(),
            steps: Vec::new(),
            reward: reward(lang, &entry.reward),
            accept: false,
            turn_in: false,
        });
    }
    let offer = talk
        .offers
        .iter()
        .find(|offer| Some(offer.id) == talk.chosen)?;
    match &offer.state {
        OfferState::Available {
            story,
            reward: pays,
        } => {
            let steps =
                (0..story.facts.asks.len()).map(|ask| or_partly(story.step_text(lang, ask, 0)));
            Some(Detail {
                accept: true,
                turn_in: false,
                ..about(story, steps.collect(), pays, lang)
            })
        }
        OfferState::Taken | OfferState::Ready => {
            let entry = book.entries.get(&offer.id)?;
            Some(Detail {
                accept: false,
                // The log moves on as steps are done; the offer only when
                // the giver is asked again.
                turn_in: entry.state == QuestState::Ready,
                ..about(&entry.story, under_way(entry, lang), &entry.reward, lang)
            })
        }
        OfferState::Locked(_) => None,
    }
}

/// A quest's name, story, steps (then going back to the giver) and reward.
fn about(story: &Story, mut steps: Vec<String>, pays: &[RewardPart], lang: &Lang) -> Detail {
    steps.push(or_partly(story.return_text(lang)));
    Detail {
        title: or_partly(story.title(lang)),
        paragraphs: or_partly_all(story.paragraphs_text(lang)),
        steps_head: plain(lang, "talk-steps"),
        steps,
        reward: reward(lang, pays),
        accept: false,
        turn_in: false,
    }
}

/// Each step of a quest taken, with how far along it is.
fn under_way(entry: &LogEntry, lang: &Lang) -> Vec<String> {
    let done = done_steps(entry);
    (0..entry.steps.len())
        .map(|i| {
            let count = entry.counts.get(i).copied().unwrap_or(0);
            let step = or_partly(entry.story.step_text(lang, i, count));
            if done[i] {
                let args = Args::new().text("step", step);
                or_partly(lang.render("talk-step-done", &args))
            } else {
                step
            }
        })
        .collect()
}

fn reward(lang: &Lang, parts: &[RewardPart]) -> String {
    let pays = or_partly(reward_text(lang, parts));
    or_partly(lang.render("talk-reward", &Args::new().text("reward", pays)))
}

fn or_partly_all(written: Result<Vec<String>, shared::lang::RenderError>) -> Vec<String> {
    written.unwrap_or_else(|error| vec![or_partly(Err(error))])
}
