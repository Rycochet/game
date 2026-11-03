//! Spawning an open list.

use bevy::prelude::*;

use super::super::fullscreen::layer_z;
use super::super::text_box::{TextBox, spawn_text_box};
use super::super::{PANEL_EDGE, Pressable, panel, stops_presses};
use super::{
    Dropdown, FONT, GAP, LEAST_WIDTH, LIST_LAYER, List, ListBacking, ListRows, ROW, ROWS,
    SEARCH_HEIGHT, list_corner,
};

pub(super) fn spawn_list(
    commands: &mut Commands,
    dropdown: Entity,
    of: &Dropdown,
    place: Rect,
    window: Vec2,
) -> List {
    let width = place.width().max(LEAST_WIDTH);
    let search_room = if of.searchable {
        SEARCH_HEIGHT + GAP
    } else {
        0.0
    };
    let rows = of.options.len().clamp(1, ROWS) as f32;
    let height = search_room + rows * (ROW + GAP) + 2.0 * PANEL_EDGE + GAP;
    let corner = list_corner(place, height, width, window);
    let backing = commands
        .spawn((
            ListBacking,
            Pressable,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            GlobalZIndex(layer_z(LIST_LAYER)),
        ))
        .id();
    let holder = commands.spawn(panel()).id();
    commands.entity(holder).insert((
        stops_presses(holder),
        Node {
            position_type: PositionType::Absolute,
            left: px(corner.x),
            top: px(corner.y),
            width: px(width),
            border: UiRect::all(px(PANEL_EDGE)),
            padding: UiRect::all(px(GAP)),
            row_gap: px(GAP),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ));
    commands.entity(backing).add_child(holder);
    let mut search = None;
    commands.entity(holder).with_children(|inside| {
        if of.searchable {
            let text_box = TextBox {
                prefix: "Search: ",
                ..default()
            };
            let node = Node {
                height: px(SEARCH_HEIGHT),
                min_height: px(SEARCH_HEIGHT),
                ..default()
            };
            search = Some(spawn_text_box(inside, text_box, FONT, node).id());
        }
        inside.spawn((
            ListRows,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(GAP),
                ..default()
            },
        ));
    });
    List {
        dropdown,
        backing,
        search,
        filter: String::new(),
        highlight: None,
        first: 0,
        dragged: 0.0,
        drawn: None,
    }
}
