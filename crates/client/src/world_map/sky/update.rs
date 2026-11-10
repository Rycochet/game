//! Working out the sky each moment, and passing it to the shaders every frame.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use shared::world::GAME_HOUR_SECONDS;

use super::super::lights::{FLASH_COLOUR, FLASH_LIT};
use super::super::map_sky::MapSky;
use super::super::view_square;
use super::{
    CLOUD_HEIGHT, CloudMaterial, PLAYER_GLOW, PrecipitationMaterial, SKY_INTERVAL, SKY_REACH,
    SKY_SIDE, SKY_SPACING, SkyAssets, SkyParams, TerrainMaterial,
};
use crate::common::{
    CurrentWorld, GraphicsPrefs, Lightning, MapPosition, Preferences, ServerClock, Shadows,
};
use crate::player::Player;
use crate::view3d::{OrbitCamera, ViewCamera};

/// What the Graphics settings show, for the shaders (`SkyParams::shown`).
fn shown(graphics: &GraphicsPrefs) -> Vec4 {
    let on = |yes: bool| if yes { 1.0 } else { 0.0 };
    Vec4::new(
        on(graphics.clouds),
        on(graphics.clouds && graphics.shadows != Shadows::Off),
        graphics.precipitation.share(),
        on(graphics.foliage),
    )
}

/// One moment of the sky, written into a texture.
pub(super) fn work_out_moment(
    world: &CurrentWorld,
    image: &mut Image,
    temperatures: &mut Temperatures,
    origin: (i64, i64),
    hours: f64,
) {
    temperatures.update(world, origin);
    let Some(data) = image.data.as_mut() else {
        return;
    };
    for row in 0..SKY_SIDE {
        for column in 0..SKY_SIDE {
            let weather = world.weather.sample(
                origin.0 + column as i64 * SKY_SPACING,
                origin.1 + row as i64 * SKY_SPACING,
                hours,
            );
            let i = row * SKY_SIDE + column;
            data[i * 4] = (weather.cover * 255.0) as u8;
            data[i * 4 + 1] = (weather.rain * 255.0) as u8;
            data[i * 4 + 2] = temperatures.values[i];
        }
    }
}

/// Temperature at each texture point, kept until the texture moves: it doesn't
/// change with time.
#[derive(Default)]
pub(super) struct Temperatures {
    origin: Option<(i64, i64)>,
    values: Vec<u8>,
}

impl Temperatures {
    fn update(&mut self, world: &CurrentWorld, origin: (i64, i64)) {
        if self.origin == Some(origin) {
            return;
        }
        self.origin = Some(origin);
        self.values.clear();
        let sea_level = world.params.sea_level;
        for row in 0..SKY_SIDE as i64 {
            for column in 0..SKY_SIDE as i64 {
                let (x, y) = (
                    origin.0 + column * SKY_SPACING,
                    origin.1 + row * SKY_SPACING,
                );
                let temperature = world.climate.sample(x, y, sea_level).temperature;
                self.values.push((temperature * 255.0) as u8);
            }
        }
    }
}

/// What the sky was last worked out from.
#[derive(Default)]
pub struct SkyState {
    /// Each moment's texture position, older first.
    origins: [Option<(i64, i64)>; 2],
    /// The clock's time at the newer moment.
    newer_hours: f64,
    temperatures: Temperatures,
    /// The window the cloud sheet was cut to: its turn and size.
    square: Option<(f32, f32)>,
}

/// Every frame: passes the time and window to the shaders, and each
/// `SKY_INTERVAL` works out the next moment of the sky around the player.
#[allow(clippy::too_many_arguments)]
pub fn update(
    time: Res<Time>,
    world: Res<CurrentWorld>,
    clock: Res<ServerClock>,
    sky_now: Res<MapSky>,
    preferences: Res<Preferences>,
    lightning: Res<Lightning>,
    assets: Option<ResMut<SkyAssets>>,
    player: Single<&MapPosition, With<Player>>,
    orbit: Single<&OrbitCamera, With<ViewCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut clouds: ResMut<Assets<CloudMaterial>>,
    mut precipitation: ResMut<Assets<PrecipitationMaterial>>,
    mut state: Local<SkyState>,
) {
    let Some(mut assets) = assets else {
        return;
    };
    let (px, py) = (player.x, player.y);
    let hours = clock.hours();
    let interval = SKY_INTERVAL / GAME_HOUR_SECONDS as f64;

    // The texture snaps to whole spacings, so clouds stay put as the player
    // walks.
    let half_side = (SKY_SIDE as i64 / 2) * SKY_SPACING;
    let origin = (
        px.div_euclid(SKY_SPACING) * SKY_SPACING - half_side,
        py.div_euclid(SKY_SPACING) * SKY_SPACING - half_side,
    );
    let far_off = state.origins.iter().any(|o| {
        o.is_none_or(|o| {
            (o.0 + half_side - px).abs() > SKY_REACH || (o.1 + half_side - py).abs() > SKY_REACH
        })
    });
    // Just started, teleported, or the clock jumped: both moments, now.
    if far_off
        || hours < state.newer_hours - 2.0 * interval
        || hours > state.newer_hours + 2.0 * interval
    {
        let SkyState {
            temperatures,
            origins,
            newer_hours,
            ..
        } = &mut *state;
        for (i, moment) in [hours, hours + interval].into_iter().enumerate() {
            if let Some(mut image) = images.get_mut(&assets.moments[i]) {
                work_out_moment(&world, &mut image, temperatures, origin, moment);
            }
            origins[i] = Some(origin);
        }
        *newer_hours = hours + interval;
    } else if hours >= state.newer_hours {
        // The newer moment becomes the older; the next is worked out into
        // the older's texture.
        assets.moments.swap(0, 1);
        state.origins.swap(0, 1);
        let next = state.newer_hours + interval;
        let SkyState {
            temperatures,
            origins,
            ..
        } = &mut *state;
        if let Some(mut image) = images.get_mut(&assets.moments[1]) {
            work_out_moment(&world, &mut image, temperatures, origin, next);
        }
        origins[1] = Some(origin);
        state.newer_hours = next;
    }

    let corner = |o: Option<(i64, i64)>| {
        let (x, y) = o.unwrap_or_default();
        // Texel centres are half a spacing in from the texture's corner.
        Vec2::new(x as f32, y as f32) - Vec2::splat(SKY_SPACING as f32 / 2.0)
    };
    let graphics = &preferences.graphics;
    let square = view_square(&orbit, graphics.view_distance);
    let sky_now = &sky_now.0;
    let params = SkyParams {
        origin_a: corner(state.origins[0]),
        origin_b: corner(state.origins[1]),
        extent: (SKY_SIDE as i64 * SKY_SPACING) as f32,
        blend: (1.0 - (state.newer_hours - hours) / interval).clamp(0.0, 1.0) as f32,
        time: time.elapsed_secs(),
        cloud_height: CLOUD_HEIGHT,
        half: square.half,
        right: square.right,
        towards: square.towards,
        centre: Vec2::new(px as f32, py as f32),
        // Lightning lights the clouds and rain up too.
        light: sky_now
            .lit()
            .lerp(FLASH_COLOUR * FLASH_LIT, lightning.0)
            .extend(PLAYER_GLOW * (1.0 - sky_now.flat.max_element().min(1.0))),
        shown: shown(graphics),
        still: Vec4::new(
            graphics.least_light(),
            if graphics.reduce_motion {
                0.0
            } else {
                time.elapsed_secs()
            },
            0.0,
            0.0,
        ),
    };
    let [older, newer] = &assets.moments;
    // Writing to these also tells Bevy to pick up changed textures.
    for handle in [&assets.terrain, &assets.rivers] {
        if let Some(mut material) = terrain_materials.get_mut(handle) {
            material.extension.params = params;
            material.extension.sky_a = older.clone();
            material.extension.sky_b = newer.clone();
        }
    }
    if let Some(mut material) = clouds.get_mut(&assets.clouds) {
        material.params = params;
        material.sky_a = older.clone();
        material.sky_b = newer.clone();
    }
    if let Some(mut material) = precipitation.get_mut(&assets.precipitation) {
        material.params = params;
        material.sky_a = older.clone();
        material.sky_b = newer.clone();
    }

    // The cloud sheet: the window's square, straight above it.
    if state.square != Some((orbit.yaw, square.half))
        && let Some(mut mesh) = meshes.get_mut(&assets.cloud_sheet)
    {
        state.square = Some((orbit.yaw, square.half));
        let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .map(|(u, v)| (square.right * u + square.towards * v) * square.half);
        let positions: Vec<[f32; 3]> = corners.iter().map(|c| [c.x, CLOUD_HEIGHT, c.y]).collect();
        *mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 4])
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        // Seen from above whichever way it was wound.
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3, 0, 2, 1, 0, 3, 2]));
    }
}
