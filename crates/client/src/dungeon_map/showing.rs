//! Showing and hiding the scene, and tokens on cells no one sees.

use bevy::prelude::*;
use shared::sheet::Feeling;

use super::{CELL_SIZE, DungeonScene, Shown, TOKEN_AMBIENT_BRIGHTNESS};
use crate::common::MapPosition;
use crate::common::{DetectMonsters, Other};
use crate::view3d::{TokenView, ViewGrid};

/// Becoming the current view: shows the scene, sets the dim fill light for
/// tokens, and tells `view3d` the grid's size, with the floor at 0.
pub(super) fn show_scene(
    mut commands: Commands,
    mut scene: Query<&mut Visibility, With<DungeonScene>>,
) {
    for mut visibility in &mut scene {
        *visibility = Visibility::Inherited;
    }
    commands.insert_resource(GlobalAmbientLight {
        brightness: TOKEN_AMBIENT_BRIGHTNESS,
        ..default()
    });
    commands.insert_resource(ViewGrid {
        cell_size: CELL_SIZE,
        floor: 0.0,
    });
}

pub(super) fn hide_scene(
    mut scene: Query<&mut Visibility, With<DungeonScene>>,
    mut tokens: Query<&mut Visibility, (With<TokenView>, Without<DungeonScene>)>,
) {
    for mut visibility in &mut scene {
        *visibility = Visibility::Hidden;
    }
    // Other views show every token they don't hide themselves.
    for mut visibility in &mut tokens {
        visibility.set_if_neq(Visibility::Inherited);
    }
}

/// Tokens are shown only on cells someone sees.
pub(super) fn hide_unseen_tokens(
    shown: Res<Shown>,
    positions: Query<&MapPosition>,
    others: Query<&Other>,
    detect: Res<DetectMonsters>,
    mut tokens: Query<(&TokenView, &mut Visibility)>,
) {
    let Some(sight) = &shown.sight else {
        return;
    };
    for (view, mut visibility) in &mut tokens {
        let detected = detect.0
            && others
                .get(view.owner)
                .is_ok_and(|other| other.feeling != Feeling::None);
        let seen = detected
            || positions
                .get(view.owner)
                .is_ok_and(|position| sight.is_seen(position.x, position.y));
        visibility.set_if_neq(if seen {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
