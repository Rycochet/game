//! The screen, as markup: a full-screen layer with the grid of characters on
//! the left and the chosen one on the right, and above it the requester asking
//! before one is deleted. Built once and shown while choosing.

use bevy::prelude::*;

use super::requester::Answer;
use crate::common::CharacterSelect;
use crate::dom::{Dom, Template, document};
use crate::html;
use crate::widgets::UiId;

/// The form all its widgets' ids are in.
pub(super) const FORM: &str = "character_select";

/// The two columns are as tall as the layer (`valign="stretch"`). The cells
/// of the grid are drawn by code (`cells`), in the canvas.
pub(super) static CHARACTER_SELECT: Template = html!(
    "character_select",
    r#"
<form id="character_select">
  <dialog id="layer" modal depth="0" width="90%" height="90%" hidden>
    <span class="apart" nowrap valign="stretch" fills>
      <div class="rows" fills>
        <h1>Characters</h1>
        <scroll id="scroll" fills>
          <canvas id="grid"/>
        </scroll>
        <span id="change_user_row" hidden>
          <button id="change_user">Change User</button>
        </span>
      </div>
      <div class="rows" width="380">
        <h1 id="name"/>
        <p id="words" class="medium"/>
        <div fills/>
        <p id="created" class="medium"/>
        <span align="right" nowrap>
          <button id="play" class="main">Play</button>
          <button id="delete" class="danger">Delete</button>
          <button id="cancel">Cancel</button>
        </span>
      </div>
    </span>
  </dialog>
  <dialog id="asking" title="Delete character" depth="1" hidden>
    <p id="question" class="medium" width="360"/>
    <footer>
      <button id="really_delete" class="danger">Delete</button>
      <button id="keep">Cancel</button>
    </footer>
  </dialog>
</form>
"#
);

/// Where the cells go, wrapping like words.
#[derive(Component)]
pub(super) struct Grid;

/// What the buttons along the bottom right do, and Change User at the bottom
/// left.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum Button {
    Play,
    Delete,
    Cancel,
    ChangeUser,
}

pub(super) fn spawn_screen(mut commands: Commands) {
    commands.spawn(document(&CHARACTER_SELECT));
}

/// Gives the built parts what the screen's systems know them by, found by
/// their ids.
pub(super) fn bind_screen(mut commands: Commands, new: Query<(Entity, &UiId), Added<UiId>>) {
    for (entity, id) in &new {
        let (FORM, name) = id.parts() else {
            continue;
        };
        let mut bound = commands.entity(entity);
        match name {
            "grid" => bound.with_child((
                Grid,
                Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(12),
                    row_gap: px(12),
                    ..default()
                },
            )),
            "play" => bound.insert(Button::Play),
            "delete" => bound.insert(Button::Delete),
            "cancel" => bound.insert(Button::Cancel),
            "change_user" => bound.insert(Button::ChangeUser),
            // A press outside the requester is No too.
            "asking" | "keep" => bound.insert(Answer::Keep),
            "really_delete" => bound.insert(Answer::Delete),
            _ => continue,
        };
    }
}

/// Shows the screen while choosing.
pub(super) fn show_screen(select: Res<CharacterSelect>, mut dom: Dom) {
    if select.is_changed() {
        dom.set_flag("character_select/layer", "hidden", !select.open);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_screen_is_good_markup() {
        super::CHARACTER_SELECT.assert_good();
    }
}
