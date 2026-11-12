//! The loading screen as markup, and filling its places once it is built.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{
    BACKGROUND, BarText, GOLD, LoadingScreen, START_TITLE_TOP, TITLE, TITLE_FONT_BYTES,
    TITLE_SHADOW, Title, title_font_size,
};
use crate::common::{Session, interface_area};
use crate::dom::{Template, document};
use crate::html;
use crate::widgets::{UiId, layer_z};

/// The screen: the title, drawn by code in its own font, and the bar at the
/// bottom. The dark behind them, and its place over everything, are the
/// document's own node (`spawn_screen`).
pub(super) static LOADING: Template = html!(
    "loading",
    r#"
<form id="loading" fills>
  <div id="screen" fills align="center" valign="middle">
    <canvas id="title"/>
    <progress id="bar" class="loading" value="0" max="1"
              left="32" right="32" bottom="32" height="32"/>
  </div>
</form>
"#
);

pub(super) fn spawn_screen(mut commands: Commands) {
    let fills = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    // Above everything but the full-screen layers, so the system menu can be
    // the startup menu over it.
    let over = GlobalZIndex(layer_z(0) - 1);
    let screen = (LoadingScreen, BackgroundColor(BACKGROUND), over);
    commands.spawn((screen, document(&LOADING))).insert(fills);
}

/// Fills the screen's places as soon as it is built, in the same frame: the
/// title in its font, already the size it should be, and the bar's words
/// marked for their colour.
#[allow(clippy::too_many_arguments)]
pub(super) fn bind_screen(
    new: Query<(Entity, &UiId), Added<UiId>>,
    children: Query<&Children>,
    texts: Query<(), With<Text>>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    session: Res<Session>,
    mut fonts: ResMut<Assets<Font>>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        match id.parts() {
            ("loading", "title") => {
                let font = fonts.add(Font::from_bytes(TITLE_FONT_BYTES.to_vec()));
                let at_start = !session.is_playing();
                let size = title_font_size(interface_area(&window, &ui_scale), at_start);
                let font = TextFont {
                    font: font.into(),
                    font_size: size.into(),
                    ..default()
                };
                let shadow = TextShadow {
                    offset: Vec2::splat(3.0),
                    color: TITLE_SHADOW,
                };
                let words = (Text::new(TITLE), font, TextLayout::justify(Justify::Center));
                let title = (Title, words, TextColor(GOLD), shadow, title_node(at_start));
                commands.entity(entity).with_child(title);
            }
            ("loading", "bar") => {
                let words = children.iter_descendants(entity);
                if let Some(words) = words.into_iter().find(|child| texts.contains(*child)) {
                    commands.entity(words).insert(BarText);
                }
            }
            _ => {}
        }
    }
}

/// The title's own node: at the startup menu it keeps to the top of the window.
pub(super) fn title_node(at_start: bool) -> Node {
    let top = match at_start {
        true => Val::Vh(START_TITLE_TOP),
        false => Val::Auto,
    };
    Node {
        margin: UiRect::top(top),
        ..default()
    }
}
