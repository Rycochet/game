//! One quest in full, as words: on offer (its story, what to do, what it pays,
//! Accept and Decline), under way or done (how far along, Complete when every
//! step is done), or just handed in (the giver's last words).

use shared::lang::Lang;
use shared::protocol::{Ahead, OfferState, QuestState};
use shared::story::{RewardPart, Story};

use super::OpenTalk;
use crate::quests::{QuestBook, or_partly, plain, reward_line, step_lines};

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
            reward: reward_line(lang, &entry.reward),
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
            ahead,
        } => {
            let steps =
                (0..story.facts.asks.len()).map(|ask| or_partly(story.step_text(lang, ask, 0)));
            let mut detail = Detail {
                accept: true,
                turn_in: false,
                ..about(story, steps.collect(), pays, lang)
            };
            // Its boss is already dead, or its item found: the player says so
            // (owner).
            match ahead {
                Some(Ahead::Slain) => detail.paragraphs.push(plain(lang, "talk-dealt-with")),
                Some(Ahead::Found) => detail.paragraphs.push(plain(lang, "talk-found")),
                None => {}
            }
            Some(detail)
        }
        OfferState::Taken | OfferState::Ready => {
            let entry = book.entries.get(&offer.id)?;
            Some(Detail {
                accept: false,
                // The log moves on as steps are done; the offer only when
                // the giver is asked again.
                turn_in: entry.state == QuestState::Ready,
                ..about(&entry.story, step_lines(entry, lang), &entry.reward, lang)
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
        reward: reward_line(lang, pays),
        accept: false,
        turn_in: false,
    }
}

fn or_partly_all(written: Result<Vec<String>, shared::lang::RenderError>) -> Vec<String> {
    written.unwrap_or_else(|error| vec![or_partly(Err(error))])
}
