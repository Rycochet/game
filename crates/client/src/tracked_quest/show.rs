//! Writing the box out when the quest followed, its progress, the words or
//! the minimap change; and keeping it under the minimap.

use bevy::prelude::*;
use shared::protocol::LogEntry;

use super::lines::{objectives, return_line};
use crate::common::{GameView, Wording};
use crate::dom::Dom;
use crate::minimap::MinimapArea;
use crate::quests::{MarkLook, QuestBook, or_partly};

/// The gap between the minimap's picture and the box.
const GAP: f32 = 4.0;

/// What the box was last written from.
#[derive(Clone, PartialEq)]
pub(super) struct Shown {
    entry: Option<LogEntry>,
    area: MinimapArea,
    playing: bool,
}

pub(super) fn show_tracked(
    book: Res<QuestBook>,
    wording: Res<Wording>,
    area: Res<MinimapArea>,
    view: Res<State<GameView>>,
    mut dom: Dom,
    mut was: Local<Option<Shown>>,
) {
    if !(book.is_changed() || wording.is_changed() || area.is_changed() || view.is_changed())
        && was.is_some()
    {
        return;
    }
    let entry = book.followed().and_then(|id| book.entries.get(&id));
    let now = Shown {
        entry: entry.cloned(),
        area: *area,
        playing: *view.get() != GameView::Connecting,
    };
    if was.as_ref() == Some(&now) {
        return;
    }
    let shows = now.playing && now.area.shown && now.entry.is_some();
    dom.set_flag("tracked/box", "hidden", !shows);
    if was.as_ref().is_none_or(|was| was.area != now.area) {
        dom.set("tracked/box", "top", &(now.area.bottom + GAP).to_string());
        dom.set("tracked/box", "width", &now.area.across.to_string());
    }
    if let Some(entry) = &now.entry {
        write_out(entry, &wording.0, &mut dom);
    }
    *was = Some(now);
}

/// The marker and name, each objective, and "Return to ..." with "?".
fn write_out(entry: &LogEntry, lang: &shared::lang::Lang, dom: &mut Dom) {
    dom.empty("tracked/lines");
    let look = MarkLook::of_state(entry.kind, entry.state);
    let head = look.class().replace("mark-", "tracked/head_");
    dom.copy_template(&head, "tracked/lines", 0);
    dom.set_text("tracked/mark.0", format!(" {} ", look.glyph()));
    dom.set_text("tracked/name.0", or_partly(entry.story.title(lang)));
    let lines = objectives(entry, lang);
    for (at, line) in lines.iter().enumerate() {
        dom.copy_template("tracked/line", "tracked/lines", at);
        dom.set_text(&format!("tracked/line.{at}"), line.clone());
    }
    if let Some(words) = return_line(entry, lang) {
        let ready = MarkLook::new(entry.kind, true);
        let template = ready.class().replace("mark-", "tracked/return_");
        dom.copy_template(&template, "tracked/lines", 0);
        dom.set_text("tracked/to.0", words);
        dom.set_text("tracked/ready.0", format!(" {} ", ready.glyph()));
    }
}
