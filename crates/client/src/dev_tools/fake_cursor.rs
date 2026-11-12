//! A pointer of dev tools' own, in place of the mouse's (`HOW_DEBUG_CURSOR`,
//! the debug API's `client.cursor` and fake mouse). It goes into
//! `PointerPosition`, which the whole game reads, so the game takes it for
//! the mouse; the real pointer on the screen never moves.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use std::env;

use super::parse_pos;
use crate::common::{PointerOverride, PointerPosition, UiPointer, in_interface};

/// Fake input that has gone quiet this long gives the pointer back to the
/// real mouse, so a test left half done never leaves the game unusable.
const HAND_BACK_SECONDS: f32 = 60.0;

/// Where dev tools hold the pointer, while they do.
#[derive(Resource, Default)]
pub(super) struct FakeCursor {
    /// In logical pixels; `None`: the real mouse.
    pub(super) at: Option<Vec2>,
    /// When fake input last used it (real seconds), for handing it back.
    /// `None`: held until let go (`client.cursor`, `HOW_DEBUG_CURSOR`).
    pub(super) used_at: Option<f32>,
}

impl FakeCursor {
    /// Held by fake input, which uses it now.
    #[cfg(feature = "debug-api")]
    pub(super) fn used(&mut self, at: Vec2, now: f32) {
        self.at = Some(at);
        self.used_at = Some(now);
    }

    /// Back to the real mouse.
    pub(super) fn let_go(&mut self) {
        *self = Self::default();
    }
}

pub(super) fn add(app: &mut App) {
    let at = env::var("HOW_DEBUG_CURSOR")
        .ok()
        .and_then(|pos| parse_pos(&pos))
        .map(|(x, y)| Vec2::new(x as f32, y as f32));
    app.insert_resource(FakeCursor { at, used_at: None })
        .add_systems(PreUpdate, point_there.in_set(PointerOverride));
}

/// Puts the pointer where the fake cursor is, unless a finger is on the
/// screen (or just lifted), and hands it back once fake input goes quiet.
pub(super) fn point_there(
    mut cursor: ResMut<FakeCursor>,
    touches: Res<Touches>,
    time: Res<Time<Real>>,
    ui_scale: Res<UiScale>,
    mut pointer: ResMut<PointerPosition>,
    mut ui_pointer: ResMut<UiPointer>,
) {
    if cursor
        .used_at
        .is_some_and(|used| time.elapsed_secs() - used > HAND_BACK_SECONDS)
    {
        cursor.let_go();
    }
    let Some(at) = cursor.at else {
        return;
    };
    if touches.iter().next().is_some() || touches.iter_just_released().next().is_some() {
        return;
    }
    pointer.0 = Some(at);
    ui_pointer.0 = Some(in_interface(at, &ui_scale));
}

#[cfg(test)]
mod tests;
