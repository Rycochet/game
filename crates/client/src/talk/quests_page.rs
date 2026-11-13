//! Writing out a panel when what it shows changes: its title (who is behind
//! the door), which tab is shown, and the Quests page: the list of what the
//! giver has, or one quest in full with the buttons that suit it.

use bevy::prelude::*;
use shared::lang::Lang;
use shared::protocol::{Offer, OfferState};
use shared::story::Story;

use super::detail::{Detail, detail_of};
use super::words::{giver_title, hint_text, plain};
use super::{OpenTalk, OpenTalks, Tab};
use crate::common::{CurrentDungeon, Wording};
use crate::dom::Dom;
use crate::quests::QuestBook;

pub(super) fn show_pages(
    mut open: ResMut<OpenTalks>,
    book: Res<QuestBook>,
    wording: Res<Wording>,
    dungeon: Res<CurrentDungeon>,
    mut dom: Dom,
) {
    // A quest under way moving on shows at once.
    let changed = book.is_changed();
    for talk in open.0.iter_mut().filter(|talk| talk.dirty || changed) {
        talk.dirty = false;
        let ids = Ids(format!("talk#{}", talk.key()));
        let lang = &wording.0;
        let shop = talk.grid.is_some();
        let tab = if shop { talk.tab } else { Tab::Quests };
        dom.set_flag(&ids.of("tabs"), "hidden", !shop);
        let words = if tab == Tab::Quests {
            "Quests"
        } else {
            "Trade"
        };
        dom.set(&ids.of("tabs"), "value", words);
        dom.set_flag(&ids.of("quests_page"), "hidden", tab != Tab::Quests);
        dom.set_flag(&ids.of("trade_page"), "hidden", tab != Tab::Trade);
        let kind = (dungeon.buildings.iter()).find(|look| look.id == talk.building);
        if let (Some(look), Some(offer)) = (kind, talk.offers.first()) {
            let title = giver_title(lang, &offer.giver, look.kind);
            dom.set(&ids.of("panel"), "title", &title);
        }
        let refused = talk.refused.clone().unwrap_or_default();
        dom.set_flag(&ids.of("refused"), "hidden", refused.is_empty());
        dom.set_text(&ids.of("refused"), refused);
        match detail_of(talk, &book, lang) {
            Some(detail) => show_detail(&ids, &detail, &mut dom),
            None => show_list(&ids, talk, &book, lang, &mut dom),
        }
    }
}

/// A panel's ids: `talk#<key>/<name>`.
pub(super) struct Ids(pub(super) String);

impl Ids {
    pub(super) fn of(&self, name: &str) -> String {
        format!("{}/{name}", self.0)
    }

    pub(super) fn at(&self, name: &str, at: usize) -> String {
        format!("{}/{name}.{at}", self.0)
    }
}

/// Everything the giver has: a button for each quest that can be taken,
/// handed in or is under way, and what they say of those that can't be yet.
fn show_list(ids: &Ids, talk: &OpenTalk, book: &QuestBook, lang: &Lang, dom: &mut Dom) {
    dom.set_flag(&ids.of("detail"), "hidden", true);
    dom.set_flag(&ids.of("list"), "hidden", false);
    dom.empty(&ids.of("list"));
    if talk.offers.is_empty() {
        dom.copy_template(&ids.of("hint"), &ids.of("list"), 0);
        dom.set_text(&ids.at("hint", 0), plain(lang, "talk-nothing"));
    }
    // One row for each offer, in order: a press on row `at` is offer `at`.
    for (at, offer) in talk.offers.iter().enumerate() {
        if let OfferState::Locked(hints) = &offer.state {
            let said: Vec<String> = hints.iter().map(|hint| hint_text(lang, hint)).collect();
            dom.copy_template(&ids.of("hint"), &ids.of("list"), at);
            dom.set_text(&ids.at("hint", at), format!("! {}", said.join(" ")));
            continue;
        }
        let glyph = match offer.state {
            OfferState::Available { .. } => "! ",
            OfferState::Ready => "? ",
            _ => "",
        };
        let title = story_of(offer, book).map(|story| story.title(lang));
        let title = title.map(super::words::or_partly).unwrap_or_default();
        dom.copy_template(&ids.of("offer"), &ids.of("list"), at);
        dom.set_text(&ids.at("offer", at), format!("{glyph}{title}"));
    }
}

/// An offer's story: sent with it while on offer, else the log's.
pub(super) fn story_of<'a>(offer: &'a Offer, book: &'a QuestBook) -> Option<&'a Story> {
    match &offer.state {
        OfferState::Available { story, .. } => Some(story),
        _ => book.entries.get(&offer.id).map(|entry| &*entry.story),
    }
}

/// One quest in full: its name, its story (or the giver's last words), its
/// steps, its reward and its buttons.
fn show_detail(ids: &Ids, detail: &Detail, dom: &mut Dom) {
    dom.set_flag(&ids.of("list"), "hidden", true);
    dom.set_flag(&ids.of("detail"), "hidden", false);
    dom.set_text(&ids.of("title"), detail.title.clone());
    dom.empty(&ids.of("paras"));
    for (at, para) in detail.paragraphs.iter().enumerate() {
        dom.copy_template(&ids.of("para"), &ids.of("paras"), at);
        dom.set_text(&ids.at("para", at), para.clone());
    }
    dom.empty(&ids.of("steps"));
    dom.set_flag(&ids.of("steps_head"), "hidden", detail.steps.is_empty());
    dom.set_text(&ids.of("steps_head"), detail.steps_head.clone());
    for (at, step) in detail.steps.iter().enumerate() {
        dom.copy_template(&ids.of("step"), &ids.of("steps"), at);
        dom.set_text(&ids.at("step", at), step.clone());
    }
    dom.set_text(&ids.of("reward"), detail.reward.clone());
    for (name, shown) in [
        ("accept", detail.accept),
        ("turn_in", detail.turn_in),
        ("decline", detail.accept),
        ("back", !detail.accept),
    ] {
        dom.set_flag(&ids.of(name), "hidden", !shown);
    }
    dom.set_flag(&ids.of("enter"), "hidden", detail.enter.is_none());
    if let Some(words) = &detail.enter {
        dom.set_text(&ids.of("enter"), words.clone());
    }
}
