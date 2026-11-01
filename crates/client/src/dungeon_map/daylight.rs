//! The light from outside: what the suns and moons give the site's place on the
//! world map, on everything out in the open, and spilling in from there through
//! openings and down stairs. Worked out once a game hour.

use std::collections::BTreeSet;

use bevy::prelude::*;
use shared::celestial::{brightest, flat_light};
use shared::dungeon::{Cell, Level, line_is_clear};
use shared::protocol::SeenStairs;
use shared::rng::Seed;
use shared::site::BuildingLook;

use super::mesh::{LightGrid, with_ambient};
use super::town::window_faces;
use super::{CropPart, HerbPart, Surface, SurfaceMaterials, TOKEN_AMBIENT_BRIGHTNESS};
use crate::common::{
    CurrentDungeon, CurrentWorld, GameClock, MapPosition, ServerClock, WorldReturn,
};
use crate::player::Player;

/// How much of the light clouds take away, as `SHADOW` in `terrain.wesl`.
const CLOUD_SHADOW: f64 = 0.2;

/// Out in the open, tokens have this ambient light in full light: they are lit
/// by real lights, where tiles have it baked in.
const OPEN_AMBIENT_BRIGHTNESS: f32 = 1500.0;

/// Against the outside light, what spills from an open cell at no distance, and
/// how many cells it reaches.
const SPILL_STRENGTH: f32 = 0.5;
const SPILL_REACH: f32 = 4.0;
/// The same down stairs open to the sky, where it falls from above.
const STAIRS_REACH: f32 = 3.0;

/// How far a light low in the sky stretches spilt light along its way, and
/// shortens it the other way: at most this much either way.
const STRETCH: f32 = 1.0;
const LEAST_STRETCH: f32 = 0.5;

/// A town's buildings put their lights on once the suns and moons give flat
/// ground less than this, each up to this many game minutes after that, and off
/// as long after it comes back.
const LIGHTS_ON_BELOW: f32 = 0.3;
const LIGHTS_SPREAD_MINUTES: u64 = 60;
/// What a lit window gives, and how many cells it reaches.
pub(super) const WINDOW_LIGHT: Vec3 = Vec3::new(1.0, 0.85, 0.6);
const WINDOW_REACH: f32 = 3.0;

/// The buildings of the town with their lights on, by `id`. Only for looks, so
/// only the client knows; clients agree as far as their clocks do.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub(super) struct TownLights(pub BTreeSet<u16>);

/// How many minutes after the light changes a building's lights follow. From
/// the world and place, so they differ between towns.
fn lights_delay(world: Seed, place: (i64, i64), building: u16) -> u64 {
    world.at(&[place.0, place.1, i64::from(building)]) % LIGHTS_SPREAD_MINUTES
}

/// Whether the light was low enough for lights at a time.
fn light_is_low(world: &CurrentWorld, minutes: f64) -> bool {
    let bodies = world
        .celestial
        .at(minutes / shared::world::MINUTES_PER_HOUR);
    let flat = flat_light(&bodies);
    flat.into_iter().fold(0.0, f64::max) < f64::from(LIGHTS_ON_BELOW)
}

/// Each game minute, works out which of a town's buildings have their lights on.
pub(super) fn update_town_lights(
    world: Option<Res<CurrentWorld>>,
    clock: Res<ServerClock>,
    place: Res<WorldReturn>,
    dungeon: Res<CurrentDungeon>,
    mut lights: ResMut<TownLights>,
    mut last_minute: Local<Option<i64>>,
) {
    let Some(world) = world else {
        return;
    };
    let minute = clock.minutes as i64;
    if *last_minute == Some(minute) && !dungeon.is_changed() && !world.is_changed() {
        return;
    }
    *last_minute = Some(minute);
    let on: BTreeSet<u16> = dungeon
        .buildings
        .iter()
        .filter(|building| {
            let delay = lights_delay(world.params.seed, (place.0.x, place.0.y), building.id);
            light_is_low(&world, minute as f64 - delay as f64)
        })
        .map(|building| building.id)
        .collect();
    if lights.0 != on {
        lights.0 = on;
    }
}

/// The light outside at the site's place.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct OutsideLight {
    /// On open ground, linear RGB from 0 (only the dull ambient light) to 1.
    pub light: Vec3,
    /// The way the brightest light is going across the grid, longer the lower
    /// it is: 1 on the horizon, 0 straight overhead.
    pub travel: Vec2,
}

impl OutsideLight {
    /// How much to multiply what is out in the open and unlit by.
    pub fn lit(&self) -> Vec3 {
        with_ambient(self.light)
    }
}

/// Keeps the outside light up to date, for a new hour or anything changing the
/// world (the debug menu's panel), or arriving somewhere else.
pub(super) fn update_outside(
    world: Option<Res<CurrentWorld>>,
    clock: Res<GameClock>,
    place: Res<WorldReturn>,
    mut outside: ResMut<OutsideLight>,
) {
    let Some(world) = world else {
        return;
    };
    if !(world.is_changed() || clock.is_changed() || place.is_changed()) {
        return;
    }
    let hours = f64::from(clock.hour);
    let bodies = world.celestial.at(hours);
    let cover = world.weather.sample(place.0.x, place.0.y, hours).cover;
    let flat = flat_light(&bodies).map(|channel| (channel * (1.0 - CLOUD_SHADOW * cover)) as f32);
    let [east, _, south] = bodies[brightest(&bodies)].direction;
    outside.set_if_neq(OutsideLight {
        light: Vec3::from_array(flat).min(Vec3::ONE),
        travel: -Vec2::new(east as f32, south as f32),
    });
}

/// What isn't baked into meshes' colours follows the light: remembered open
/// ground, which would otherwise be brighter than what is in sight at night, and
/// crops and plants.
#[allow(clippy::type_complexity)]
pub(super) fn tint_outside(
    outside: Res<OutsideLight>,
    surfaces: Res<SurfaceMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    plants: Query<&MeshMaterial3d<StandardMaterial>, Or<(With<CropPart>, With<HerbPart>)>>,
) {
    if !outside.is_changed() {
        return;
    }
    let lit = outside.lit();
    let tint = |material: &mut StandardMaterial, by: Vec3| {
        let alpha = material.base_color.alpha();
        material.base_color = Color::linear_rgba(by.x, by.y, by.z, alpha);
    };
    for surface in [Surface::Ground, Surface::Town] {
        if let Some(mut material) = materials.get_mut(&surfaces.memory[surface as usize]) {
            tint(&mut material, Vec3::splat(lit.max_element()));
        }
    }
    for plant in &plants {
        if let Some(mut material) = materials.get_mut(&plant.0) {
            tint(&mut material, lit);
        }
    }
}

/// Tokens have more ambient light with the player out in the open or in a town,
/// following the light there, and the usual dim fill otherwise.
pub(super) fn token_ambient(
    outside: Res<OutsideLight>,
    dungeon: Res<CurrentDungeon>,
    player: Single<&MapPosition, With<Player>>,
    ambient: Option<ResMut<GlobalAmbientLight>>,
) {
    let Some(mut ambient) = ambient else {
        return;
    };
    let open = dungeon.is_town() || dungeon.level.cell(player.x, player.y).is_outside();
    let strongest = outside.light.max_element();
    let (brightness, tint) = if open && strongest > 0.0 {
        let extra = (OPEN_AMBIENT_BRIGHTNESS - TOKEN_AMBIENT_BRIGHTNESS) * strongest;
        (TOKEN_AMBIENT_BRIGHTNESS + extra, outside.light / strongest)
    } else {
        (TOKEN_AMBIENT_BRIGHTNESS, Vec3::ONE)
    };
    let color = Color::linear_rgb(tint.x, tint.y, tint.z);
    if ambient.brightness != brightness || ambient.color != color {
        ambient.brightness = brightness;
        ambient.color = color;
    }
}

/// How much of the outside light spills to each cell, kept until the level or
/// the light changes: it costs too much to work out each time the camera moves.
#[derive(Default)]
pub(super) struct Spill {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    amount: Vec<f32>,
}

/// Whether a cell has the sky over it. `open_air` says water and lava do too,
/// in places that are all in the open apart from their buildings; underground,
/// they don't.
fn under_sky(cell: Cell, open_air: bool) -> bool {
    cell.is_outside() || (open_air && cell.is_liquid())
}

impl Spill {
    /// Works it out for a level: from open cells beside anything open that isn't
    /// under the sky, stretched by `travel`, and down stairs open to the sky.
    /// Each cell takes the strongest reaching it, so more of them are no brighter.
    pub fn work_out(level: &Level, stairs: &[SeenStairs], travel: Vec2, open_air: bool) -> Self {
        let mut spill = Self::none(level);
        let (min_x, min_y, width, height) = level.bounds();
        let takes = |cell: Cell| !under_sky(cell, open_air);
        let sheltered =
            |x: i64, y: i64| level.is_open(x, y) && !under_sky(level.cell(x, y), open_air);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                let from_open_cell = under_sky(level.cell(x, y), open_air)
                    && (y - 1..=y + 1).any(|ny| (x - 1..=x + 1).any(|nx| sheltered(nx, ny)));
                if from_open_cell {
                    let stretch = |way: Vec2| (1.0 + STRETCH * way.dot(travel)).max(LEAST_STRETCH);
                    spill.add(level, (x, y), SPILL_REACH, 1.0 + STRETCH, takes, stretch);
                }
            }
        }
        for stair in stairs.iter().filter(|stair| stair.open_to_sky) {
            spill.add(level, (stair.x, stair.y), STAIRS_REACH, 1.0, takes, |_| 1.0);
        }
        spill
    }

    /// What the windows of the buildings in `on` give, the same way, from the
    /// cell in front of each, out in the open as much as anywhere.
    pub fn from_windows(level: &Level, buildings: &[BuildingLook], on: &BTreeSet<u16>) -> Self {
        let mut spill = Self::none(level);
        for building in buildings
            .iter()
            .filter(|building| on.contains(&building.id))
        {
            for rect in &building.rects {
                for y in rect.y..rect.y + rect.height {
                    for x in rect.x..rect.x + rect.width {
                        if level.cell(x, y) != Cell::BlindWindow {
                            continue;
                        }
                        if let Some((dx, dy)) = window_faces(level, (x, y)) {
                            let front = (x + dx, y + dy);
                            spill.add(level, front, WINDOW_REACH, 1.0, |_| true, |_| 1.0);
                        }
                    }
                }
            }
        }
        spill
    }

    /// Nothing anywhere on a level.
    fn none(level: &Level) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        Self {
            min_x,
            min_y,
            width,
            height,
            amount: vec![0.0; (width * height) as usize],
        }
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        ((0..self.width).contains(&col) && (0..self.height).contains(&row))
            .then_some((row * self.width + col) as usize)
    }

    /// Spills from one cell, as far as `reach` cells times `stretch` of the way
    /// to a cell (at most `longest`), stopped by walls, on the cells that `takes`
    /// it.
    fn add(
        &mut self,
        level: &Level,
        from: (i64, i64),
        reach: f32,
        longest: f32,
        takes: impl Fn(Cell) -> bool,
        stretch: impl Fn(Vec2) -> f32,
    ) {
        let far = (reach * longest).ceil() as i64;
        for y in from.1 - far..=from.1 + far {
            for x in from.0 - far..=from.0 + far {
                let Some(i) = self.index(x, y) else {
                    continue;
                };
                if !level.is_open(x, y) || !takes(level.cell(x, y)) {
                    continue;
                }
                let way = Vec2::new((x - from.0) as f32, (y - from.1) as f32);
                let distance = way.length() / stretch(way.normalize_or_zero());
                let fade = 1.0 - (distance / (reach + 1.0)).powi(2);
                let amount = SPILL_STRENGTH * fade;
                // The line last: it costs the most.
                if amount > self.amount[i] && line_is_clear(level, from, (x, y)) {
                    self.amount[i] = amount;
                }
            }
        }
    }

    pub fn at(&self, x: i64, y: i64) -> f32 {
        self.index(x, y).map_or(0.0, |i| self.amount[i])
    }
}

/// The light from outside on every cell: all of it under the sky, or in a town
/// (out in the open all over), and what spills in elsewhere.
pub(super) fn light_from_outside(
    level: &Level,
    spill: &Spill,
    light: Vec3,
    town: bool,
    open_air: bool,
) -> LightGrid {
    if town {
        return LightGrid::filled(level, light);
    }
    let mut grid = LightGrid::new(level);
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if under_sky(level.cell(x, y), open_air) {
                grid.set(x, y, light);
            } else {
                grid.set(x, y, light * spill.at(x, y));
            }
        }
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::site::{StairsDirection, StairsStyle};

    /// `#` wall, `.` floor, `,` ground, `W` window, `+` door.
    fn level(rows: &[&str]) -> Level {
        let mut level = Level::new(0, 0, rows[0].len() as i64, rows.len() as i64);
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let cell = match c {
                    '#' => Cell::Wall,
                    '.' => Cell::Floor,
                    ',' => Cell::Ground,
                    'W' => Cell::Window,
                    '+' => Cell::Door,
                    _ => panic!("{c}"),
                };
                level.set(x as i64, y as i64, cell);
            }
        }
        level
    }

    const ROOM: [&str; 5] = [
        ",,###########",
        ",,#.........#",
        ",,W.........#",
        ",,#.........#",
        ",,###########",
    ];

    fn spilling(level: &Level, travel: Vec2) -> Spill {
        Spill::work_out(level, &[], travel, false)
    }

    #[test]
    fn light_comes_in_through_a_window_and_fades() {
        let level = level(&ROOM);
        let spill = spilling(&level, Vec2::ZERO);
        let along: Vec<f32> = (2..9).map(|x| spill.at(x, 2)).collect();
        assert!(along[0] > 0.4, "{along:?}");
        assert!(along.windows(2).all(|pair| pair[0] >= pair[1]), "{along:?}");
        assert_eq!(spill.at(8, 2), 0.0);
    }

    #[test]
    fn more_open_cells_are_no_brighter() {
        let wide = level(&[
            ",,,,,,,,,",
            ",,,,,,,,,",
            "####W####",
            "#.......#",
            "#.......#",
            "#########",
        ]);
        // From the window's own column, whatever else reaches it, no more than one.
        let amount = spilling(&wide, Vec2::ZERO).at(4, 3);
        assert!(amount <= SPILL_STRENGTH, "{amount}");
    }

    #[test]
    fn walls_stop_it() {
        let shut = level(&[",,#.....", ",,#.....", ",,#....."]);
        let spill = spilling(&shut, Vec2::ZERO);
        assert_eq!(spill.at(3, 1), 0.0);
    }

    #[test]
    fn a_low_light_stretches_it_along_its_way() {
        let level = level(&ROOM);
        let going_in = spilling(&level, Vec2::X);
        let going_out = spilling(&level, Vec2::NEG_X);
        let flat = spilling(&level, Vec2::ZERO);
        assert!(going_in.at(7, 2) > flat.at(7, 2));
        assert!(going_in.at(5, 2) > going_out.at(5, 2));
    }

    #[test]
    fn stairs_open_to_the_sky_light_their_surroundings() {
        let cave = level(&["#####", "#...#", "#...#", "#...#", "#####"]);
        let stair = |open_to_sky| SeenStairs {
            open_to_sky,
            x: 2,
            y: 2,
            direction: StairsDirection::Up,
            descends: (0, 1),
            style: StairsStyle::Spiral,
        };
        let lit = Spill::work_out(&cave, &[stair(true)], Vec2::ZERO, false);
        assert_eq!(lit.at(2, 2), SPILL_STRENGTH);
        assert!(lit.at(1, 1) > 0.0);
        let shut = Spill::work_out(&cave, &[stair(false)], Vec2::ZERO, false);
        assert_eq!(shut.at(2, 2), 0.0);
    }

    #[test]
    fn covered_ground_has_only_what_spills_under_it() {
        let hut = level(&[",,,,,,,,", ",,,,,,,,", ",,,,,,,,"]);
        let mut hut = hut;
        for y in 0..3 {
            for x in 2..8 {
                hut.set(x, y, Cell::CoveredGround);
            }
        }
        let spill = spilling(&hut, Vec2::ZERO);
        let light = Vec3::ONE;
        let grid = light_from_outside(&hut, &spill, light, false, false);
        assert_eq!(grid.at(0, 1), light);
        let edge = grid.at(2, 1).x;
        assert!(edge > 0.0 && edge < 1.0);
        assert!(grid.at(7, 1).x < edge);
    }

    #[test]
    fn water_is_only_under_the_sky_in_open_air_places() {
        let pool = level(&["...", "...", "..."]);
        let mut pool = pool;
        pool.set(1, 1, Cell::Water);
        let spill = Spill::default();
        let inside = light_from_outside(&pool, &spill, Vec3::ONE, false, false);
        assert_eq!(inside.at(1, 1), Vec3::ZERO);
        let open = light_from_outside(&pool, &spill, Vec3::ONE, false, true);
        assert_eq!(open.at(1, 1), Vec3::ONE);
    }

    fn building(id: u16, window: (i64, i64)) -> BuildingLook {
        use shared::site::{BuildingKind, RoofKind, WallKind};
        BuildingLook {
            id,
            kind: BuildingKind::House,
            walls: WallKind::ALL[0],
            roof: RoofKind::ALL[0],
            rects: vec![shared::site::Rect {
                x: window.0,
                y: window.1,
                width: 1,
                height: 1,
            }],
            door: (0, 0),
            faces: (0, 1),
        }
    }

    #[test]
    fn a_lit_window_lights_the_cells_before_it_and_only_if_on() {
        let mut street = level(&[
            ",,,,,,,,,",
            ",,,,,,,,,",
            ",,,,,,,,,",
            ",,,,,,,,,",
            "#########",
        ]);
        street.set(4, 4, Cell::BlindWindow);
        let buildings = [building(7, (4, 4))];
        let on = BTreeSet::from([7]);
        let glow = Spill::from_windows(&street, &buildings, &on);
        assert_eq!(glow.at(4, 3), SPILL_STRENGTH);
        assert!(glow.at(4, 1) > 0.0);
        assert_eq!(glow.at(8, 0), 0.0);
        let dark = Spill::from_windows(&street, &buildings, &BTreeSet::new());
        assert_eq!(dark.at(4, 3), 0.0);
    }

    #[test]
    fn lights_come_on_over_about_an_hour() {
        let seed = Seed::new(5);
        let delays: BTreeSet<u64> = (0..100).map(|id| lights_delay(seed, (3, 4), id)).collect();
        assert!(delays.iter().all(|&delay| delay < LIGHTS_SPREAD_MINUTES));
        assert!(delays.len() > 30, "{delays:?}");
        // The same for everyone, and different in another town.
        assert_eq!(lights_delay(seed, (3, 4), 9), lights_delay(seed, (3, 4), 9));
        let elsewhere =
            (0..100).filter(|&id| lights_delay(seed, (3, 4), id) != lights_delay(seed, (9, 4), id));
        assert!(elsewhere.count() > 50);
    }

    #[test]
    fn a_town_has_it_all_over() {
        let town = level(&["#.,"]);
        let grid = light_from_outside(&town, &Spill::default(), Vec3::splat(0.5), true, false);
        assert_eq!(grid.at(0, 0), Vec3::splat(0.5));
        assert_eq!(grid.at(1, 0), Vec3::splat(0.5));
    }
}
