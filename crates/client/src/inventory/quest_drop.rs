//! Dropping a quest item destroys it (owner), so it is asked first: a
//! requester naming it, Destroy or Keep. Letting go of one over the ground
//! leaves it where it was and asks; Destroy sends the drop, which the server
//! carries out by destroying it (or the quest items in a bag). Giving one to
//! anyone is refused here as it is on the server.

use bevy::prelude::*;
use shared::id::Id;
use shared::protocol::ClientMessage;

use super::Carried;
use crate::connection::ToServer;
use crate::dom::{Dom, Template, UiEvent, UiValue, document};
use crate::html;

/// The quest item waiting on the player's answer, if one is.
#[derive(Resource, Default)]
pub(super) struct QuestDrop {
    pub(super) asking: Option<Id>,
}

static QUEST_DROP: Template = html!(
    "quest_drop",
    r#"
<form id="quest_drop">
  <dialog id="asking" title="Destroy quest item" depth="1" hidden>
    <p id="question" class="medium" width="360"/>
    <footer>
      <button id="destroy" class="danger">Destroy</button>
      <button id="keep">Keep</button>
    </footer>
  </dialog>
</form>
"#
);

pub(super) fn spawn_requester(mut commands: Commands) {
    commands.spawn(document(&QUEST_DROP));
}

/// Up while a drop is asked about, naming what would be destroyed.
pub(super) fn show_requester(drop: Res<QuestDrop>, carried: Res<Carried>, mut dom: Dom) {
    if !drop.is_changed() {
        return;
    }
    let item = drop.asking.and_then(|id| carried.0.find(id));
    if let Some(item) = item {
        let what = if item.is_quest_item() {
            format!("{} belongs to a quest.", item.name())
        } else {
            format!("{} holds something that belongs to a quest.", item.name())
        };
        let question = format!("{what} Dropping it destroys it. Destroy it?");
        dom.set_text("quest_drop/question", question);
    }
    dom.set_flag("quest_drop/asking", "hidden", item.is_none());
}

/// The answer: Destroy drops it (the server destroys it), Keep or a press
/// outside leaves it.
pub(super) fn answer_requester(
    mut events: MessageReader<UiEvent>,
    mut drop: ResMut<QuestDrop>,
    mut carried: ResMut<Carried>,
    mut to_server: MessageWriter<ToServer>,
) {
    for event in events.read() {
        let ("quest_drop", name) = event.id.parts() else {
            continue;
        };
        let Some(item) = drop.asking else {
            continue;
        };
        match (&event.value, name) {
            (UiValue::Pressed, "destroy") => {
                if carried.0.hand_over(item).is_some() {
                    to_server.write(ToServer(ClientMessage::DropItem { item }));
                }
                drop.asking = None;
            }
            (UiValue::Pressed, "keep") | (UiValue::Dismissed, _) => drop.asking = None,
            _ => {}
        }
    }
}

/// Whether `id` is something that can't be given, as it is or holds a quest
/// item.
pub(super) fn holds_quest_item(carried: &Carried, id: Id) -> bool {
    (carried.0.find(id)).is_some_and(shared::item::Item::holds_quest_item)
}
