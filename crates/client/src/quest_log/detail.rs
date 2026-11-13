//! One quest in full on the log's right: its name, who gave it and where,
//! where it stands, its story, its steps with counts, its reward, the places
//! it sends you to (buttons showing them on the map), and Track and Abandon
//! for one under way on this planet.

use shared::lang::{Args, Lang};
use shared::poi::ChunkPos;
use shared::protocol::{LogEntry, QuestState};
use shared::quest::QuestId;

use super::page::{id, id_at};
use crate::common::{CurrentWorld, poi_name};
use crate::dom::Dom;
use crate::quests::{or_partly, plain, reward_line, step_lines};

/// Writes `entry` out. `world` is the planet's when it is the one the player
/// is on (for its places and buttons). The places' chunks, by button.
pub(super) fn show_detail(
    entry: &LogEntry,
    world: Option<&CurrentWorld>,
    on_world_map: bool,
    followed: Option<QuestId>,
    abandoning: bool,
    lang: &Lang,
    dom: &mut Dom,
) -> Vec<ChunkPos> {
    dom.set_text(&id("title"), or_partly(entry.story.title(lang)));
    dom.set_text(&id("giver"), giver_line(entry, lang));
    dom.set_text(&id("state"), plain(lang, state_message(entry.state)));
    dom.empty(&id("paras"));
    let paras = entry.story.paragraphs_text(lang);
    let paras = paras.unwrap_or_else(|error| vec![or_partly(Err(error))]);
    for (at, para) in paras.into_iter().enumerate() {
        dom.copy_template(&id("para"), &id("paras"), at);
        dom.set_text(&id_at("para", at), para);
    }
    let mut steps = step_lines(entry, lang);
    let active = matches!(entry.state, QuestState::Taken | QuestState::Ready);
    if active {
        steps.push(or_partly(entry.story.return_text(lang)));
    }
    dom.set_text(&id("steps_head"), plain(lang, "talk-steps"));
    dom.empty(&id("steps"));
    for (at, step) in steps.into_iter().enumerate() {
        dom.copy_template(&id("step"), &id("steps"), at);
        dom.set_text(&id_at("step", at), step);
    }
    dom.set_text(&id("reward"), reward_line(lang, &entry.reward));
    let targets = match (world, active) {
        (Some(world), true) => targets(entry, world),
        _ => Vec::new(),
    };
    dom.set_flag(&id("targets_head"), "hidden", targets.is_empty());
    dom.set_text(&id("targets_head"), plain(lang, "log-targets"));
    dom.empty(&id("targets"));
    for (at, (_, name)) in targets.iter().enumerate() {
        dom.copy_template(&id("target"), &id("targets"), at);
        dom.set_text(&id_at("target", at), name.clone());
        // The full map shows the world map's places only from there.
        dom.set_flag(&id_at("target", at), "disabled", !on_world_map);
    }
    let ours = world.is_some() && active;
    dom.set_flag(&id("track"), "hidden", !ours);
    dom.set_flag(&id("track"), "selected", followed == Some(entry.id));
    dom.set_flag(&id("abandon"), "hidden", !ours);
    let abandon = if abandoning {
        "Really abandon?"
    } else {
        "Abandon"
    };
    dom.set_text(&id("abandon"), abandon);
    targets.into_iter().map(|(chunk, _)| chunk).collect()
}

/// "From Mara Tull of Ashby".
fn giver_line(entry: &LogEntry, lang: &Lang) -> String {
    let giver = or_partly(entry.story.cast.giver().person.full_name(lang));
    let args = Args::new()
        .name("giver", giver)
        .name("town", entry.story.facts.home.clone());
    or_partly(lang.render("log-giver", &args))
}

fn state_message(state: QuestState) -> &'static str {
    match state {
        QuestState::Taken => "log-state-taken",
        QuestState::Ready => "log-state-ready",
        QuestState::Done => "log-state-done",
        QuestState::Failed => "log-state-failed",
        QuestState::Abandoned => "log-state-abandoned",
    }
}

/// The places the quest's steps not yet done send you to, each once, by the
/// names the player knows them by (accepting a quest makes its places known
/// and named).
fn targets(entry: &LogEntry, world: &CurrentWorld) -> Vec<(ChunkPos, String)> {
    let done = crate::quests::done_steps(entry);
    let mut found: Vec<(ChunkPos, String)> = Vec::new();
    for (step, _) in entry.steps.iter().zip(&done).filter(|(_, done)| !**done) {
        let chunk = step.target.chunk();
        if found.iter().any(|(known, _)| *known == chunk) {
            continue;
        }
        if let Some(poi) = world.pois.poi_in(chunk) {
            found.push((chunk, poi_name(&poi, world.pois.name_of(chunk))));
        }
    }
    found
}
