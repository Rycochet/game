//! A quest's words as the talk panel and the Quest Log both write them: what
//! could be written (missing parts in braces, logged), each step with how far
//! along it is, and the reward.

use shared::lang::{Args, Lang, RenderError};
use shared::protocol::LogEntry;
use shared::story::{RewardPart, reward_text};

use super::done_steps;

/// What could be written of a message: any part missing is left in braces
/// (and logged), rather than nothing shown.
pub fn or_partly(written: Result<String, RenderError>) -> String {
    written.unwrap_or_else(|error| {
        bevy::log::warn!("Words missing in {}: {:?}", error.id, error.problems);
        error.text
    })
}

/// A message with no values.
pub fn plain(lang: &Lang, id: &str) -> String {
    or_partly(lang.render(id, &Args::new()))
}

/// Each step of a quest taken, with how far along it is ("(done)" once done).
pub fn step_lines(entry: &LogEntry, lang: &Lang) -> Vec<String> {
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

/// "Reward: ...".
pub fn reward_line(lang: &Lang, parts: &[RewardPart]) -> String {
    let pays = or_partly(reward_text(lang, parts));
    or_partly(lang.render("talk-reward", &Args::new().text("reward", pays)))
}
