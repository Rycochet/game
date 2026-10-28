//! The slice view: the original stylised side-on view of the surface around
//! the player, drawn on the CPU into an image. See
//! `docs/context/client/slice_view.md`.

mod render;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::common::{CurrentWorld, GameClock, MapView, ViewStats, Weather};
use crate::player::{MapPosition, Player};
use crate::surface_style::SurfaceStyle;

/// On-screen size of each map pixel.
const PIXEL_SCALE: f32 = 2.0;

pub struct SliceViewPlugin;

impl Plugin for SliceViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MapView::Slices), spawn_view)
            .add_systems(OnExit(MapView::Slices), |mut commands: Commands| {
                commands.remove_resource::<MapImage>();
            })
            .add_systems(Update, redraw.run_if(in_state(MapView::Slices)));
    }
}

#[derive(Resource)]
struct MapImage(Handle<Image>);

fn spawn_view(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: render::WIDTH as u32,
            height: render::HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // Keep the chunky pixels sharp when scaled up.
    image.sampler = ImageSampler::nearest();
    let handle = images.add(image);

    commands.spawn((Camera2d, DespawnOnExit(MapView::Slices)));
    commands.spawn((
        Sprite {
            image: handle.clone(),
            custom_size: Some(Vec2::new(
                render::WIDTH as f32 * PIXEL_SCALE,
                render::HEIGHT as f32 * PIXEL_SCALE,
            )),
            ..default()
        },
        DespawnOnExit(MapView::Slices),
    ));
    commands.insert_resource(MapImage(handle));
}

/// Redraws only when something that affects the picture has changed.
#[allow(clippy::too_many_arguments)]
fn redraw(
    player: Single<Ref<MapPosition>, With<Player>>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    style: Res<SurfaceStyle>,
    world: Res<CurrentWorld>,
    map: Res<MapImage>,
    mut images: ResMut<Assets<Image>>,
    mut stats: ResMut<ViewStats>,
) {
    let changed =
        player.is_changed() || clock.is_changed() || weather.is_changed() || style.is_changed();
    if !(changed || world.is_changed() || map.is_added()) {
        return;
    }
    let Some(mut image) = images.get_mut(&map.0) else {
        return;
    };
    let Some(pixels) = image.data.as_mut() else {
        return;
    };

    let start = Instant::now();
    render::render(
        &world.surface,
        &style,
        world.params.water_level(clock.hour as f64),
        weather.snow_line,
        player.x,
        player.y,
        pixels,
    );
    let ms = start.elapsed().as_secs_f32() * 1000.0;
    stats.summary = format!("Slice view: redraw {ms:.1} ms");
}
