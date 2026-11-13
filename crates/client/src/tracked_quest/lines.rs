//! What the box says of a quest: its objectives that can be done now (steps
//! not done whose earlier steps are), each with its count, and whether only
//! handing it in is left.

use shared::lang::Lang;
use shared::protocol::{LogEntry, QuestState};

use crate::quests::{done_steps, or_partly};

/// The steps to do now, as words with their counts ("Slay kobolds 2/6").
pub(super) fn objectives(entry: &LogEntry, lang: &Lang) -> Vec<String> {
    let done = done_steps(entry);
    (entry.steps.iter().enumerate())
        .filter(|(i, step)| !done[*i] && step.is_open(&done))
        .map(|(i, _)| {
            let count = entry.counts.get(i).copied().unwrap_or(0);
            or_partly(entry.story.step_text(lang, i, count))
        })
        .collect()
}

/// "Return to ...", once every step is done.
pub(super) fn return_line(entry: &LogEntry, lang: &Lang) -> Option<String> {
    (entry.state == QuestState::Ready).then(|| or_partly(entry.story.return_text(lang)))
}
