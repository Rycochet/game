//! Drawing the minimap or the full map when what it shows changes.

use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;

use super::canvas::Canvas;
use super::frame::FrameCache;
use super::grid::Looks;
use super::lines::MapRivers;
use super::overlay::{MarksLayer, Under};
use super::projection::Projection;
use super::redraw::{FILLING_SECONDS, KeptLand, Redraw};
use super::sprites::Sprites;
use super::world::WorldTiles;
use super::{
    DrawnFrom, FULL_MAP_BUDGET_MS, ICON_PIXELS, MINIMAP_BUDGET_MS, MapPictures, REDRAW_SECONDS,
    Room, Sources, full_map_area, full_map_compass, minimap_across, minimap_border, minimap_marks,
    minimap_picture_size, prepare_stamps, under_the_sky, units_per_pixel,
};
use crate::common::GameView;
use crate::widgets::PARCHMENT;

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_maps(
    sources: Sources,
    pictures: Res<MapPictures>,
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
    mut tiles: ResMut<WorldTiles>,
    mut rivers: ResMut<MapRivers>,
    mut sprites: ResMut<Sprites>,
    mut layer: ResMut<MarksLayer>,
    mut last: Local<Option<DrawnFrom>>,
    mut pending: Local<bool>,
    (mut since, mut since_filled): (Local<f32>, Local<f32>),
    (mut kept, mut frame): (Local<KeptLand>, Local<FrameCache>),
) {
    let Sources {
        view,
        window,
        ui_scale,
        player,
        camera,
        world,
        dungeon,
        levels,
        first_floor: _,
        style,
        sky: sky_now,
        pois,
        asset_server,
        full,
    } = sources;
    let view = *view.get();
    let window = Room::of(&window, &ui_scale);
    // The pictures' pixels to an interface pixel, for their sizes; what is drawn in
    // them follows the screen (`dots`), as the maps do, not the interface.
    let scale = window.scale_factor();
    let dots = window.screen_factor();
    let on_other_floor = levels.showing().is_some();
    let spot = levels.spot(&dungeon, (player.x, player.y));
    // Which way the player's floor is from the one shown.
    let player_is_up = dungeon
        .info
        .as_ref()
        .zip(levels.showing())
        .is_some_and(|(info, shown)| {
            (info.storey(info.index) > info.storey(shown)) == info.building
        });
    let players_level = &*dungeon;
    let dungeon = levels.shown(players_level);

    // The turn: the full map always has north at the top, like a sheet of
    // paper, while the minimap turns with the camera.
    let (right, down) = camera.ground_axes();
    let (grid_right, grid_down) = if full.open {
        Projection::NORTH_UP
    } else {
        (
            (f64::from(right.x), f64::from(right.y)),
            (f64::from(down.x), f64::from(down.y)),
        )
    };

    // (Their elements are sized by `place_maps`.)
    let across = if full.open {
        full_map_area(&window)
    } else {
        Vec2::splat(minimap_picture_size(&window))
    };
    let size = (
        ((across.x * scale).round() as u32).max(1),
        ((across.y * scale).round() as u32).max(1),
    );

    let centre = full.centre.filter(|_| full.open).unwrap_or(spot);
    let zoom = if full.open { full.zoom } else { 1.0 };
    let units = units_per_pixel(view, &window, &camera) / f64::from(scale * zoom);
    let now = DrawnFrom {
        view,
        floor: levels.showing(),
        open: full.open,
        centre,
        turn: (grid_right.0 as f32, grid_right.1 as f32),
        scale: units,
        size,
    };
    *since += time.delta_secs();
    // Only what is remembered in sites and the minimap's sky change without
    // telling: the full map of the world map changes only with the world
    // (chunks and roads arriving), so it isn't redrawn on the clock.
    let on_the_clock = view != GameView::WorldMap || !full.open;
    let world_changed = world.as_ref().is_some_and(|world| world.is_changed());
    let due = on_the_clock && *since >= REDRAW_SECONDS;
    let why = Redraw::of(last.as_ref(), &now, (*pending, world_changed, due));
    if why == Redraw::No {
        return;
    }

    let projection = Projection {
        centre,
        middle: (size.0 as f32 / 2.0, size.1 as f32 / 2.0),
        units_per_pixel: units,
        right: grid_right,
        down: grid_down,
    };
    // Round the circle, or to the corners of the full map.
    let clip = if full.open {
        f32::MAX
    } else {
        minimap_across(&window) * scale / 2.0
    };
    let reach = if full.open {
        (size.0 as f32).hypot(size.1 as f32) / 2.0
    } else {
        clip
    } + ICON_PIXELS * zoom * dots;
    let grid_box = projection.grid_box(reach);

    // What can be got ready before the picture is taken out of the assets.
    let looks = (view != GameView::WorldMap).then(|| {
        let cell = |at: f64| (at + 0.5).floor() as i64;
        Looks::of(
            dungeon,
            (cell(grid_box.0.0), cell(grid_box.0.1)),
            (cell(grid_box.1.0), cell(grid_box.1.1)),
        )
    });
    let (stamps, mut busy) = prepare_stamps(
        looks.as_ref(),
        world.as_deref(),
        &pois,
        &projection,
        grid_box,
        (zoom, dots),
        (&mut sprites, &asset_server, &images),
    );
    if let (None, Some(world)) = (&looks, world.as_deref()) {
        let budget = if full.open {
            FULL_MAP_BUDGET_MS
        } else {
            MINIMAP_BUDGET_MS
        };
        // The tide and weather don't change the maps; only the colours they're
        // drawn in do.
        if style.is_changed() {
            tiles.clear();
        }
        busy |= tiles.fill(world, &style, grid_box, units, budget);
        busy |= rivers.fill(world, grid_box, budget);
    }
    *pending = busy;
    // Still filling in: painted now and then, not every frame.
    *since_filled += time.delta_secs();
    if why == Redraw::Filling && *since_filled < FILLING_SECONDS {
        return;
    }
    *since_filled = 0.0;
    *since = 0.0;
    *last = Some(now.clone());

    let Some(mut image) = images.get_mut(if full.open {
        &pictures.full
    } else {
        &pictures.minimap
    }) else {
        return;
    };
    if image.width() != size.0 || image.height() != size.1 {
        image.resize(Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        });
    }
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let mut canvas = Canvas {
        pixels: data,
        width: size.0,
        height: size.1,
    };
    // On the clock with nothing else changed, the minimap's land is as it was:
    // only its frame and sky are painted again. (In sites, what is remembered
    // may have changed.)
    let reuse = why == Redraw::Clock && view == GameView::WorldMap && !full.open;
    if !(reuse && kept.restore(&now, canvas.pixels)) {
        let [red, green, blue] = PARCHMENT;
        canvas.fill([red, green, blue, 255]);
        match (&looks, world.as_deref()) {
            (Some(looks), _) => super::grid::draw(&mut canvas, &projection, looks, PARCHMENT, clip),
            (None, Some(world)) => {
                let width = world.surface.width();
                tiles.draw(&mut canvas, &projection, width, clip, PARCHMENT);
                // Thicker when zoomed in, but less than the zoom.
                rivers.draw(&mut canvas, &projection, world, zoom.sqrt() * dots, clip);
            }
            (None, None) => {}
        }
        for (path, part, pixels, at, opacity) in &stamps {
            if let Some(sprite) = sprites.get(path, *part, *pixels) {
                canvas.sprite(sprite, *at, *opacity);
            }
        }
        if let Some(looks) = &looks {
            super::site_icons::draw(&mut canvas, looks, dungeon, &projection, zoom * dots, dots);
        }
        if full.open {
            kept.forget();
        } else {
            kept.keep(now, canvas.pixels);
        }
    }

    // The Hall of Worlds has no north to show (owner).
    let in_hall = players_level
        .info
        .as_ref()
        .is_some_and(|info| matches!(info.kind, shared::protocol::LevelKind::Hall { .. }));
    if full.open && !in_hall {
        full_map_compass(&mut canvas, &window, size, (dots, scale));
    } else if !full.open {
        let sky = under_the_sky(view, players_level).then_some(&*sky_now);
        let turn = (grid_right, grid_down);
        minimap_marks(
            &mut canvas,
            (&window, &mut frame),
            &projection,
            clip,
            turn,
            sky,
            !in_hall,
        );
    }
    layer.0 = Some(Under {
        projection,
        size,
        full_map: full.open,
        inside: (!full.open).then(|| (projection.middle, clip - minimap_border(&window))),
        mark: zoom * dots,
        other_floor: on_other_floor.then_some(player_is_up),
        wrap: (view == GameView::WorldMap)
            .then(|| world.as_deref().map(|world| world.surface.width()))
            .flatten(),
    });
}
