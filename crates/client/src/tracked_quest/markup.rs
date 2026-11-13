//! The tracked quest's box as markup: dark like the map's hover box, in the
//! window's top right corner under the minimap (`show` keeps its place). Its
//! heading and "Return to" line are copies of a template per marker colour,
//! as writing takes its class's colour only when made.

use bevy::prelude::*;

use crate::dom::{Click, Template, document, top_strip};
use crate::quests::{OpenQuestLog, QuestBook};
use crate::widgets::UiId;

pub(super) static TRACKED: Template = crate::html!(
    "tracked",
    r#"
<form id="tracked">
  <div id="box" top="0" right="8" hidden>
    <span class="tip">
      <div id="lines" class="tight"/>
    </span>
  </div>
  <template id="head_yellow"><span id="head" class="joined"><span id="mark" class="mark-yellow"/><p id="name"/></span></template>
  <template id="head_blue"><span id="head" class="joined"><span id="mark" class="mark-blue"/><p id="name"/></span></template>
  <template id="head_orange"><span id="head" class="joined"><span id="mark" class="mark-orange"/><p id="name"/></span></template>
  <template id="head_grey"><span id="head" class="joined"><span id="mark" class="mark-grey"/><p id="name"/></span></template>
  <template id="line"><p id="line"/></template>
  <template id="return_yellow"><span id="return" class="joined"><p id="to"/><span id="ready" class="mark-yellow"/></span></template>
  <template id="return_blue"><span id="return" class="joined"><p id="to"/><span id="ready" class="mark-blue"/></span></template>
  <template id="return_orange"><span id="return" class="joined"><p id="to"/><span id="ready" class="mark-orange"/></span></template>
</form>
"#
);

/// The box's document, over the game's view like the minimap.
pub(super) fn spawn_box(mut commands: Commands) {
    let over = (document(&TRACKED), GlobalZIndex(10));
    commands.spawn(over).insert(top_strip());
}

/// A click or tap anywhere on the box, once built, opens the Quest Log on
/// the quest followed.
pub(super) fn bind_box(new: Query<(Entity, &UiId), Added<UiId>>, mut commands: Commands) {
    for (entity, id) in &new {
        if id.parts() == ("tracked", "box") {
            commands.entity(entity).observe(open_log);
        }
    }
}

fn open_log(mut click: On<Click>, book: Res<QuestBook>, mut open: MessageWriter<OpenQuestLog>) {
    click.propagate(false);
    if let Some(id) = book.followed() {
        open.write(OpenQuestLog(id));
    }
}
