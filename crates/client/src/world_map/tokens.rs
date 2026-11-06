//! Resting tokens on the terrain, and hiding those outside the window.

use bevy::prelude::*;

use super::rivers::KnownRivers;
use super::terrain::{HeightCache, ViewSquare};
use super::view_square;
use crate::common::{CurrentWorld, GameClock, MapPosition, Preferences, SmoothPosition};
use crate::player::Player;
use crate::surface_style::SurfaceStyle;
use crate::view3d::{OrbitCamera, TOKEN_DIAMETER, TOKEN_HALF_THICKNESS, TokenView, ViewCamera};

/// Every frame, after `view3d` has placed the tokens: rests each one with its
/// centre on the drawn ground at its own point (owner), so a poi's flat base
/// meets the ground at the poi. On a slope the uphill edge dips into it.
#[allow(clippy::too_many_arguments)]
pub(super) fn rest_tokens_on_terrain(
    world: Res<CurrentWorld>,
    clock: Res<GameClock>,
    style: Res<SurfaceStyle>,
    known_rivers: Res<KnownRivers>,
    cache: Option<ResMut<HeightCache>>,
    mut views: Query<&mut Transform, With<TokenView>>,
) {
    let Some(mut cache) = cache else {
        return;
    };
    let water_level = world.water_level(&clock);
    for mut transform in &mut views {
        // Where `view3d` put it (one unit a grid point here), gliding or not.
        let (x, z) = (transform.translation.x, transform.translation.z);
        let ground = super::terrain::shown_height_at(
            &mut cache,
            &world.surface,
            &world.climate,
            &world.vegetation,
            &known_rivers,
            &style,
            water_level,
            (f64::from(x), f64::from(z)),
        );
        transform.translation.y = ground + TOKEN_HALF_THICKNESS;
    }
}

/// Every frame: hides tokens that aren't wholly inside the terrain window,
/// so none hang over its edges or float beyond them.
pub(super) fn hide_tokens_outside_window(
    player: Single<&MapPosition, With<Player>>,
    orbit: Single<&OrbitCamera, With<ViewCamera>>,
    positions: Query<&SmoothPosition>,
    mut views: Query<(&TokenView, &mut Visibility)>,
    preferences: Res<Preferences>,
) {
    let square = view_square(&orbit, preferences.video.view_distance);
    let inner = ViewSquare {
        half: square.half - TOKEN_DIAMETER / 2.0,
        ..square
    };
    for (view, mut visibility) in &mut views {
        let Ok(position) = positions.get(view.owner) else {
            continue;
        };
        // Relative to the window's centre, the player's grid point.
        let offset = Vec2::new(
            (position.x - player.x as f64) as f32,
            (position.y - player.y as f64) as f32,
        );
        visibility.set_if_neq(if inner.contains(offset) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
