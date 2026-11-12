//! A moveable panel and its parts, its title bar's stripes, and stopping presses.

use bevy::prelude::*;

use super::super::{PANEL_EDGE, RaisePanel, SHADED_EDGE, panel, panel_title, spawn_close_button};
use super::{
    CLOSE_MARGIN, CONTENT_GAP, FIRST_PANEL_Z, GRIP_GAP, GRIP_PADDING, GRIP_THICKNESS,
    MIN_STRIPES_WIDTH, PADDING, STRIPE_GAP, STRIPE_THICKNESS, STRIPES, TITLE_GAP, TOP_PADDING,
};

/// A panel that can be moved. `at` is where its top-left corner is wanted, in
/// logical pixels; it is kept on the screen.
#[derive(Component, Clone, Copy, Debug)]
pub struct MoveablePanel {
    pub(super) at: Vec2,
    /// The window's size when last placed, to keep it in the same part of the
    /// window when that changes.
    pub(super) window: Option<Vec2>,
    /// Until first placed: where it is wanted, in place of `at`.
    pub(super) first: Option<PanelPlace>,
    /// A drag has just let go of it: `place_panels` says where it ended up
    /// (`PanelMoved`).
    pub(super) dropped: bool,
}

/// Where a moveable panel first goes, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelPlace {
    /// Its top-left corner here.
    TopLeft(Vec2),
    /// Its bottom-right corner this far left of and above the window's.
    BottomRight(Vec2),
    /// Its top-left corner at this share of the window across and down, as
    /// `PanelMoved` gives it: for a place saved between runs.
    Share(Vec2),
}

/// A moveable panel was dragged and let go: where its top-left corner is now,
/// as a share of the window across and down (`PanelPlace::Share` puts it back).
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct PanelMoved {
    pub panel: Entity,
    pub share: Vec2,
}

impl MoveablePanel {
    fn new(place: PanelPlace) -> Self {
        let at = match place {
            PanelPlace::TopLeft(at) => at,
            PanelPlace::BottomRight(_) | PanelPlace::Share(_) => Vec2::ZERO,
        };
        Self {
            at,
            window: None,
            first: Some(place),
            dropped: false,
        }
    }

    /// Changes where it first goes, until it has been placed: after that it
    /// is where the player left it, and this does nothing.
    pub fn start_at(&mut self, place: PanelPlace) {
        if self.first.is_some_and(|first| first != place) {
            self.first = Some(place);
        }
    }
}

#[cfg(test)]
impl MoveablePanel {
    /// Where it will first go, until it has been placed.
    pub fn starts_at(&self) -> Option<PanelPlace> {
        self.first
    }
}

/// On a moveable panel's title: the panel it names.
#[derive(Component, Clone, Copy, Debug)]
pub struct PanelTitleOf(pub Entity);

/// What a click on this node does.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[require(crate::dom::Solid)]
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
    let root = spawn_root(commands, PanelPlace::TopLeft(at), TOP_PADDING);
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
            bar.spawn((panel_title(title), PanelTitleOf(root)));
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

/// A moveable panel with no title or close button, moved by the narrow grip
/// along its top: thinner lines, nearer the edge. Returns the panel, and the
/// entity to put its contents in; the panel fits them. Hide it with
/// `Display::None` to keep its place.
pub fn spawn_grip_panel(commands: &mut Commands, place: PanelPlace) -> (Entity, Entity) {
    let root = spawn_root(commands, place, 0.0);
    let grip = commands
        .spawn((
            PanelPart::TitleBar(root),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(GRIP_GAP),
                // The room above and below the lines grips too, for fingers.
                padding: UiRect::vertical(px(GRIP_PADDING)),
                ..default()
            },
        ))
        .with_children(|grip| {
            for _ in 0..STRIPES {
                grip.spawn((
                    Node {
                        height: px(GRIP_THICKNESS),
                        ..default()
                    },
                    BackgroundColor(SHADED_EDGE),
                ));
            }
        })
        .id();
    let content = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(CONTENT_GAP),
            ..default()
        })
        .id();
    commands.entity(root).add_children(&[grip, content]);
    commands.write_message(RaisePanel(root));
    (root, content)
}

/// A moveable panel's outside, hidden until placed, with `top` padding.
fn spawn_root(commands: &mut Commands, place: PanelPlace, top: f32) -> Entity {
    let root = commands
        .spawn((
            panel(),
            MoveablePanel::new(place),
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
                top: px(top),
                ..UiRect::all(px(PADDING))
            },
            row_gap: px(CONTENT_GAP),
            ..default()
        },
    ));
    root
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
