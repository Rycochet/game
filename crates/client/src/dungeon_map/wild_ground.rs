//! The wilderness's ground in the colours of the land under it: the world map's
//! colour at each point (biome, snow, shore), blended across the cells between
//! points, so a walk from forest into grassland changes colour as it goes.
//! The ground's picture is painted the land's colour where the player is
//! (`pictures::colour_ground`), and each cell's corners are tinted by how its
//! own colour differs from that, so the banks of water (drawn with the same
//! picture) match the land round them.

use bevy::prelude::*;
use shared::surface::SurfaceSample;
use shared::wilderness::SCALE;

use crate::common::{CurrentDungeon, CurrentWorld, GameClock, MapPosition, WorldReturn};
use crate::surface_style::SurfaceStyle;

/// Grass is the land's colour, a little darker.
const GRASS_SHADE: f32 = 0.85;

/// The most a tint brightens the picture by, each way (a light snowfield
/// beside the dark forest the picture is painted for).
const MOST_TINT: f32 = 6.0;

/// The land's colour at the points round the wilderness's window, or nothing
/// anywhere else.
#[derive(Resource, Default)]
pub struct GroundTint(Option<Tints>);

/// Colours by point, linear.
pub struct Tints {
    min_px: i64,
    min_py: i64,
    columns: i64,
    colours: Vec<Vec3>,
    /// What they were worked out for: the window, the water level and the
    /// place the ground's picture is painted for.
    made_for: ((i64, i64, i64, i64), u64, (i64, i64)),
}

impl GroundTint {
    pub fn tints(&self) -> Option<&Tints> {
        self.0.as_ref()
    }
}

impl Tints {
    /// The tint of a cell's ground (grass a little darker), blended between
    /// the four points round its middle.
    pub fn at(&self, x: i64, y: i64, grass: bool) -> Vec3 {
        let (px, py) = (x.div_euclid(SCALE), y.div_euclid(SCALE));
        let tx = (x.rem_euclid(SCALE) as f32 + 0.5) / SCALE as f32;
        let ty = (y.rem_euclid(SCALE) as f32 + 0.5) / SCALE as f32;
        let colour = |px: i64, py: i64| {
            let index = (py - self.min_py) * self.columns + px - self.min_px;
            self.colours
                .get(index as usize)
                .copied()
                .unwrap_or(Vec3::ONE)
        };
        let north = colour(px, py).lerp(colour(px + 1, py), tx);
        let south = colour(px, py + 1).lerp(colour(px + 1, py + 1), tx);
        let tint = north.lerp(south, ty);
        if grass { tint * GRASS_SHADE } else { tint }
    }
}

/// Works the colours out again when the window moves or the tide turns.
pub(super) fn work_out_tint(
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    style: Res<SurfaceStyle>,
    clock: Res<GameClock>,
    world_return: Res<WorldReturn>,
    mut tint: ResMut<GroundTint>,
) {
    let Some(world) = world.filter(|_| dungeon.is_wilderness()) else {
        if tint.0.is_some() {
            tint.0 = None;
        }
        return;
    };
    let bounds = dungeon.level.bounds();
    let water = world.water_level(&clock);
    let place = &world_return.0;
    let made_for = (bounds, water.to_bits(), (place.x, place.y));
    if tint.0.as_ref().is_some_and(|t| t.made_for == made_for) {
        return;
    }
    let (min_x, min_y, width, height) = bounds;
    let min_px = min_x.div_euclid(SCALE);
    let min_py = min_y.div_euclid(SCALE);
    let columns = (min_x + width - 1).div_euclid(SCALE) + 2 - min_px;
    let rows = (min_y + height - 1).div_euclid(SCALE) + 2 - min_py;
    let planet = world.surface.width();
    let base = linear(land_colour(&world, &style, water, *place));
    let mut colours = Vec::with_capacity((columns * rows) as usize);
    for py in min_py..min_py + rows {
        for px in min_px..min_px + columns {
            let point = MapPosition {
                x: px.rem_euclid(planet),
                y: py,
            };
            let colour = linear(land_colour(&world, &style, water, point));
            colours.push((colour / base.max(Vec3::splat(0.01))).min(Vec3::splat(MOST_TINT)));
        }
    }
    tint.0 = Some(Tints {
        min_px,
        min_py,
        columns,
        colours,
        made_for,
    });
}

/// The land's colour at a world map point as the wilderness shows it: as on
/// the world map, but a point under the sea counts as just out of it (its
/// shore), as the wilderness's own water is drawn as water.
pub fn land_colour(
    world: &CurrentWorld,
    style: &SurfaceStyle,
    water: f64,
    MapPosition { x, y }: MapPosition,
) -> [u8; 4] {
    // No jitter: it only roughens the map's colour bands.
    let sample = SurfaceSample {
        jitter: 0.0,
        ..world.surface.sample(x, y)
    };
    let climate = world.climate.sample(x, y, sample.height);
    let water = water.min(sample.height - SHORE_STEP);
    style.ground_colour(sample, climate, water, world.surface.floor())
}

/// How far under a point the water is put, to colour it as just out of it.
const SHORE_STEP: f64 = 0.01;

fn linear([r, g, b, _]: [u8; 4]) -> Vec3 {
    let linear = Color::srgb_u8(r, g, b).to_linear();
    Vec3::new(linear.red, linear.green, linear.blue)
}
