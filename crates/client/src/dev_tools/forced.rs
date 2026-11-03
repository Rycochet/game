//! What dev tools keep as they are while set: stairs' style, fields' stage, the cursor.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use shared::site::StairsStyle;
use std::env;

use super::parse_pos;
use crate::common::CurrentDungeon;

/// What dev tools keep as they are while set, however the game changes them:
/// every stairs' style, every field's stage (`HOW_DEBUG_STAIRS`,
/// `HOW_DEBUG_FIELDS`), and where the mouse cursor is (`HOW_DEBUG_CURSOR`).
/// The debug API sets them too.
#[derive(Resource, Default)]
pub(super) struct Forced {
    pub(super) stairs: Option<StairsStyle>,
    pub(super) fields: Option<u32>,
    /// In logical pixels.
    pub(super) cursor: Option<Vec2>,
}

pub(super) fn apply_forced(
    forced: Res<Forced>,
    mut dungeon: ResMut<CurrentDungeon>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if let Some(style) = forced.stairs
        && dungeon.stairs.iter().any(|stairs| stairs.style != style)
    {
        for stairs in &mut dungeon.stairs {
            stairs.style = style;
        }
    }
    if let Some(variety) = forced.fields
        && dungeon.fields.iter().any(|field| field.variety != variety)
    {
        for field in &mut dungeon.fields {
            field.variety = variety;
        }
    }
    if let Some(at) = forced.cursor
        && window.cursor_position() != Some(at)
    {
        window.set_cursor_position(Some(at));
    }
}

/// `HOW_DEBUG_STAIRS`, `HOW_DEBUG_FIELDS` and `HOW_DEBUG_CURSOR`.
pub(super) fn read(app: &mut App) {
    let mut forced = app.world_mut().resource_mut::<Forced>();
    forced.stairs = env::var("HOW_DEBUG_STAIRS")
        .ok()
        .and_then(|n| n.trim().parse::<usize>().ok())
        .and_then(|n| StairsStyle::ALL.get(n).copied());
    forced.fields = env::var("HOW_DEBUG_FIELDS")
        .ok()
        .and_then(|n| n.trim().parse::<u32>().ok());
    forced.cursor = env::var("HOW_DEBUG_CURSOR")
        .ok()
        .and_then(|pos| parse_pos(&pos))
        .map(|(x, y)| Vec2::new(x as f32, y as f32));
}
