//! The Quest Log's button, just right of the Character button at the bottom
//! left (the same size, a list on it): pressed or tapped, it does what L
//! does. Hidden while connecting, as the Character button is.

use bevy::prelude::*;

use super::icon::list_icon;
use crate::common::GameView;
use crate::dom::{Pictures, Template, bottom_strip, document};
use crate::html;
use crate::widgets::{ButtonTooltip, UiId};

/// 8 from the window's bottom, and 6 right of the Character button (8 from
/// the left, 34 across). No form round it, as for the Character button.
pub(super) static BUTTON: Template = html!(
    "quest_log.button",
    r#"
<button id="quest_log" form="hud" left="48" bottom="8" width="34" height="34">
  <img src="picture:list"/>
</button>
"#
);

/// Marks the button.
#[derive(Component)]
pub(super) struct LogButton;

pub(super) fn spawn_button(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Pictures>,
) {
    pictures.name("list", images.add(list_icon()));
    // Above the maps, as the other corner buttons are.
    let button = (document(&BUTTON), GlobalZIndex(15));
    commands.spawn(button).insert(bottom_strip());
}

/// Marks the button once it is built, with its tip.
pub(super) fn bind_button(new: Query<(Entity, &UiId), Added<UiId>>, mut commands: Commands) {
    for (entity, id) in &new {
        if id.parts() == ("hud", "quest_log") {
            let tip = ButtonTooltip("Quest Log".into());
            commands.entity(entity).insert((LogButton, tip));
        }
    }
}

/// The button is hidden while connecting.
pub(super) fn show_button(
    view: Res<State<GameView>>,
    button: Option<Single<&mut Visibility, With<LogButton>>>,
) {
    // (Not there until its markup is built.)
    let Some(mut button) = button else {
        return;
    };
    button.set_if_neq(if *view.get() == GameView::Connecting {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
}
