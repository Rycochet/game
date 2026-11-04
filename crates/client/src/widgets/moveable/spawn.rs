//! A moveable panel and its parts, its title bar's stripes, and stopping presses.

use bevy::prelude::*;

use super::super::{PANEL_EDGE, RaisePanel, SHADED_EDGE, panel, panel_title, spawn_close_button};
use super::{
    CLOSE_MARGIN, CONTENT_GAP, FIRST_PANEL_Z, MIN_STRIPES_WIDTH, PADDING, STRIPE_GAP,
    STRIPE_THICKNESS, STRIPES, TITLE_GAP, TOP_PADDING,
};

/// A panel that can be moved. `at` is where its top-left corner is wanted, in
/// logical pixels; it is kept on the screen.
#[derive(Component, Clone, Copy, Debug)]
pub(in crate::widgets) struct MoveablePanel {
    pub(super) at: Vec2,
    /// The window's size when last placed, to keep it in the same part of the
    /// window when that changes.
    pub(super) window: Option<Vec2>,
}

/// What a click on this node does.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::widgets) enum PanelPart {
    /// Only stops the click going through the panel.
    Body(Entity),
    TitleBar(Entity),
    Close(Entity),
    /// Stops it reaching widgets under it, but leaves it for its owner to read
    /// from the mouse and fingers.
    LeftForOwner(Entity),
}

/// A moveable panel with this title, at this place. Returns the panel, and the
/// entity to put its contents in; the panel fits them. It is despawned when
/// closed.
pub fn spawn_moveable_panel(commands: &mut Commands, title: &str, at: Vec2) -> (Entity, Entity) {
    let root = commands
        .spawn((
            panel(),
            MoveablePanel { at, window: None },
            GlobalZIndex(FIRST_PANEL_Z),
            Visibility::Hidden,
        ))
        .id();
    commands.entity(root).insert((
        PanelPart::Body(root),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(PANEL_EDGE)),
            padding: UiRect {
                top: px(TOP_PADDING),
                ..UiRect::all(px(PADDING))
            },
            row_gap: px(CONTENT_GAP),
            ..default()
        },
    ));
    let content = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(CONTENT_GAP),
            ..default()
        })
        .id();
    let bar = commands
        .spawn((
            PanelPart::TitleBar(root),
            Node {
                align_items: AlignItems::Center,
                column_gap: px(TITLE_GAP),
                ..default()
            },
        ))
        .with_children(|bar| {
            spawn_stripes(bar);
            bar.spawn(panel_title(title));
            spawn_stripes(bar);
            bar.spawn((
                PanelPart::Close(root),
                Node {
                    padding: UiRect::all(px(CLOSE_MARGIN)),
                    // Still on it for fingers, but not making the bar taller.
                    margin: UiRect::vertical(px(-CLOSE_MARGIN)),
                    ..default()
                },
            ))
            .with_children(|holder| {
                spawn_close_button(holder, default());
            });
        })
        .id();
    commands.entity(root).add_children(&[bar, content]);
    // In front of any panels already open.
    commands.write_message(RaisePanel(root));
    (root, content)
}

/// Put this on a node for presses on it to go no further. `entity` is the node.
pub fn stops_presses(entity: Entity) -> impl Bundle {
    PanelPart::Body(entity)
}

/// Lines running from the title out to the edge of the panel.
pub(super) fn spawn_stripes(bar: &mut ChildSpawnerCommands) {
    bar.spawn(Node {
        flex_grow: 1.0,
        min_width: px(MIN_STRIPES_WIDTH),
        flex_direction: FlexDirection::Column,
        row_gap: px(STRIPE_GAP),
        ..default()
    })
    .with_children(|stripes| {
        for _ in 0..STRIPES {
            stripes.spawn((
                Node {
                    height: px(STRIPE_THICKNESS),
                    ..default()
                },
                BackgroundColor(SHADED_EDGE),
            ));
        }
    });
}
