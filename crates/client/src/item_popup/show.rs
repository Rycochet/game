//! What an item's popup shows: its name in its quality's colour, anything
//! about where it is, a line for each bonus, and its weight at the bottom.

use bevy::prelude::*;
use shared::id::Id;
use shared::lang::Lang;

use super::open::{FORM, ItemPopups};
use super::words::bonus_lines;
use crate::common::{NO_QUALITY_NAME, quality_colour};
use crate::dom::{Dom, Template};
use crate::html;
use crate::widgets::{Pointed, TextSize, UiId, weight_words};

/// The popup: `name` is filled with the name in its colour once built; the
/// rest are set as it opens.
pub(super) static ITEM_POPUP: Template = html!(
    "item_popup",
    r#"
<form id="item_popup">
  <popup id="box" hover>
    <canvas id="name" margin-right="20"/>
    <p id="notes" hidden>.</p>
    <p id="bonuses" hidden>.</p>
    <p id="weight">.</p>
  </popup>
</form>
"#
);

/// Places the popup about `of` at `at` and writes what it says.
pub(super) fn fill(dom: &mut Dom, of: Id, pointed: &Pointed, at: Vec2, lang: &Lang) {
    let form = UiId::of(FORM, of, "box").form();
    let id = |name: &str| format!("{form}/{name}");
    dom.set(&id("box"), "left", &at.x.to_string());
    dom.set(&id("box"), "top", &at.y.to_string());
    let notes = &pointed.notes;
    dom.set_flag(&id("notes"), "hidden", notes.is_empty());
    dom.set_text(&id("notes"), notes.clone());
    let lines = bonus_lines(&pointed.item, lang);
    dom.set_flag(&id("bonuses"), "hidden", lines.is_empty());
    dom.set_text(&id("bonuses"), lines.join("\n"));
    let weight = weight_words(pointed.item.total_weight(), None);
    dom.set_text(&id("weight"), weight);
}

/// Writes each new popup's name, by its top, in its quality's colour, as
/// Give Item's lists do.
pub(super) fn bind_names(
    new: Query<(Entity, &UiId), Added<UiId>>,
    popups: Res<ItemPopups>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        if id.parts() != (FORM, "name") {
            continue;
        }
        let open = popups.open.iter().find(|open| Some(open.of) == id.which());
        let Some(open) = open else {
            continue;
        };
        let top = open.item.top();
        let colour = top.quality.map_or(NO_QUALITY_NAME, quality_colour);
        let words = (Text::new(top.name()), TextSize::Medium.font());
        commands
            .entity(entity)
            .with_child((words, TextColor(colour)));
    }
}
