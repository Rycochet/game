//! Writing out the log's list for the planet shown: each group's heading and
//! quests (its giver's marker, then its name), then a line for each giver of
//! dailies; and the quest chosen in full beside it (`detail`).

use bevy::prelude::*;
use shared::lang::{Args, Lang};
use shared::protocol::{DailyDone, LogEntry};

use super::detail::show_detail;
use super::entries::grouped;
use super::state::LogPanel;
use crate::common::{CurrentWorld, local_date_time};
use crate::dom::Dom;
use crate::quests::{MarkLook, QuestBook, or_partly, plain};

/// A log's ids: `quest_log/<name>`.
pub(super) fn id(name: &str) -> String {
    format!("quest_log/{name}")
}

/// A copy's: `quest_log/<name>.<at>`.
pub(super) fn id_at(name: &str, at: usize) -> String {
    format!("quest_log/{name}.{at}")
}

/// What the planet shown has: its quests and dailies, if told yet.
pub(super) fn shown<'a>(
    panel: &'a LogPanel,
    book: &'a QuestBook,
) -> Option<(Vec<&'a LogEntry>, &'a [DailyDone])> {
    match panel.planet {
        None => Some((book.entries.values().collect(), &book.dailies)),
        Some(world) => {
            let told = panel.others.get(&world)?;
            Some((told.quests.iter().collect(), &told.dailies))
        }
    }
}

pub(super) fn show_page(
    panel: &mut LogPanel,
    book: &QuestBook,
    lang: &Lang,
    world: Option<&CurrentWorld>,
    on_world_map: bool,
    dom: &mut Dom,
) {
    dom.empty(&id("list"));
    panel.rows.clear();
    let Some((entries, dailies)) = shown(panel, book) else {
        dom.set_flag(&id("detail"), "hidden", true);
        dom.set_text(&id("choose"), plain(lang, "log-asking"));
        dom.set_flag(&id("choose"), "hidden", false);
        return;
    };
    let mut headings = 0;
    let mut lines = 0;
    let mut rows = Vec::new();
    for (group, quests) in grouped(entries.iter().copied()) {
        heading(&mut headings, plain(lang, group.heading()), dom);
        for entry in quests {
            let at = rows.len();
            let look = MarkLook::of_state(entry.kind, entry.state);
            // `mark-yellow` and so on, each its own template.
            let template = look.class().replace("mark-", "row_");
            dom.copy_template(&id(&template), &id("list"), at);
            dom.set_text(&id_at("mark", at), format!(" {} ", look.glyph()));
            dom.set_text(&id_at("quest", at), or_partly(entry.story.title(lang)));
            dom.set_flag(
                &id_at("quest", at),
                "selected",
                panel.chosen == Some(entry.id),
            );
            rows.push(entry.id);
        }
    }
    if !dailies.is_empty() {
        heading(&mut headings, plain(lang, "log-dailies"), dom);
        for daily in dailies {
            dom.copy_template(&id("line"), &id("list"), lines);
            dom.set_text(&id_at("line", lines), daily_line(daily, lang));
            lines += 1;
        }
    }
    if headings == 0 {
        heading(&mut headings, plain(lang, "log-none"), dom);
    }
    let chosen = panel
        .chosen
        .and_then(|chosen| entries.iter().find(|e| e.id == chosen));
    let chosen = chosen.copied();
    dom.set_flag(&id("detail"), "hidden", chosen.is_none());
    dom.set_flag(&id("choose"), "hidden", chosen.is_some());
    dom.set_text(&id("choose"), plain(lang, "log-choose"));
    let here = panel.planet.is_none();
    let followed = book.followed();
    let targets = match chosen {
        Some(entry) => {
            let place = here.then_some(world).flatten();
            show_detail(
                entry,
                place,
                on_world_map,
                followed,
                panel.abandoning,
                lang,
                dom,
            )
        }
        None => Vec::new(),
    };
    panel.rows = rows;
    panel.targets = targets;
}

/// A heading in the list (or a line saying it is empty).
fn heading(count: &mut usize, words: String, dom: &mut Dom) {
    dom.copy_template(&id("heading"), &id("list"), *count);
    dom.set_text(&id_at("heading", *count), words);
    *count += 1;
}

/// "Mara Tull of Ashby: 3 done, last 9 Nov 2026, 14:05".
fn daily_line(daily: &DailyDone, lang: &Lang) -> String {
    let giver = (daily.giver.as_ref()).map(|person| or_partly(person.full_name(lang)));
    let args = Args::new()
        .name("giver", giver.unwrap_or_else(|| daily.home.clone()))
        .name("town", daily.home.clone())
        .count("count", i64::from(daily.count))
        .text(
            "when",
            local_date_time(u64::try_from(daily.last).unwrap_or(0)),
        );
    or_partly(lang.render("log-daily", &args))
}
