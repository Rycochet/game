//! Demo modes toggled by keys, for showing off and testing the views. See
//! `docs/context/client/demo.md`.
//!
//! - G: walk east one step per frame, until pressed again.
//! - T: temperature cycle: the snow line falls from normal to 0 (everything
//!   frozen) and rises back, as in the prototype.
//! - K / C: cycle the player's token shape / attitude colour; X toggles
//!   its "completed" state; Z cycles its icon size.
//! - B: cycle what the current level's stairs are built as (only what is
//!   drawn; the next level goes back to its own).
//! - F: cycle what is growing in the current level's fields (only what is drawn).
//! - H: move what grows in herb patches on a stage (only what is drawn).
//! - P: to the plain with every kind of poi (`shared::showcase`; dev tools).
//! - Esc: back to the starting point on the world map, out of any site
//!   (dev tools: only the embedded server allows it).

use bevy::prelude::*;

use shared::movement::Step;
use shared::protocol::ClientMessage;

use crate::common::{CurrentDungeon, GameView, HerbGrowthShift, NORMAL_SNOW_LINE, Token, Weather};
use crate::connection::ToServer;
use crate::player::{Player, PlayerSystems, TakeStep};

/// How fast the snow line moves during the temperature cycle, in height units
/// per second (the prototype moved 1 unit every 166 ms).
const SNOW_LINE_SPEED: f64 = 6.0;

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutoWalk>()
            .init_resource::<TemperatureCycle>()
            .add_systems(
                Update,
                (
                    toggle_modes,
                    temperature_cycle,
                    cycle_token,
                    cycle_stairs,
                    cycle_fields,
                    cycle_herbs,
                    back_to_start,
                    to_showcase,
                ),
            )
            .add_systems(
                Update,
                // Only on the world map: in a site it would stop at a wall.
                auto_walk
                    .run_if(in_state(GameView::WorldMap))
                    .in_set(PlayerSystems::Input),
            );
    }
}

/// While on, the player walks east one step per frame.
#[derive(Resource, Default)]
pub struct AutoWalk(pub bool);

/// Where the temperature demo is in its cycle.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemperatureCycle {
    #[default]
    Off,
    Cooling,
    Warming,
}

fn toggle_modes(
    keys: Res<ButtonInput<KeyCode>>,
    mut walk: ResMut<AutoWalk>,
    mut cycle: ResMut<TemperatureCycle>,
) {
    if keys.just_pressed(KeyCode::KeyG) {
        walk.0 = !walk.0;
    }
    if keys.just_pressed(KeyCode::KeyT) {
        // Starts a cycle; pressing again during one skips to warming back up.
        *cycle = match *cycle {
            TemperatureCycle::Off => TemperatureCycle::Cooling,
            _ => TemperatureCycle::Warming,
        };
    }
}

fn auto_walk(walk: Res<AutoWalk>, mut steps: MessageWriter<TakeStep>) {
    if walk.0 {
        steps.write(TakeStep(Step::new(1, 0).expect("a step")));
    }
}

/// Esc: asks the server to put the player back where new characters start.
fn back_to_start(keys: Res<ButtonInput<KeyCode>>, mut to_server: MessageWriter<ToServer>) {
    if keys.just_pressed(KeyCode::Escape) {
        to_server.write(ToServer(ClientMessage::DevBackToStart));
    }
}

/// P: asks the server to put the player on the plain with every kind of poi.
fn to_showcase(keys: Res<ButtonInput<KeyCode>>, mut to_server: MessageWriter<ToServer>) {
    if shared::showcase::ENABLED && keys.just_pressed(KeyCode::KeyP) {
        to_server.write(ToServer(ClientMessage::DevToShowcase));
    }
}

/// K and C cycle the player's token through its shapes and attitudes, and X
/// toggles it completed, to show them all.
fn cycle_token(keys: Res<ButtonInput<KeyCode>>, mut token: Single<&mut Token, With<Player>>) {
    if keys.just_pressed(KeyCode::KeyK) {
        token.shape = token.shape.next();
    }
    if keys.just_pressed(KeyCode::KeyC) {
        token.attitude = token.attitude.next();
    }
    if keys.just_pressed(KeyCode::KeyX) {
        token.completed = !token.completed;
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        token.size = token.size.next();
    }
}

/// B moves every staircase on the level to the next style, to show them all.
fn cycle_stairs(keys: Res<ButtonInput<KeyCode>>, mut dungeon: ResMut<CurrentDungeon>) {
    if keys.just_pressed(KeyCode::KeyB) {
        for stairs in &mut dungeon.stairs {
            stairs.style = stairs.style.next();
        }
    }
}

/// F moves every field on the level to its next stage, to show them all (so
/// pasture is drawn as crops, which alone go through every stage).
fn cycle_fields(keys: Res<ButtonInput<KeyCode>>, mut dungeon: ResMut<CurrentDungeon>) {
    if keys.just_pressed(KeyCode::KeyF) {
        for field in &mut dungeon.fields {
            field.livestock = false;
            field.variety = field.variety.wrapping_add(1);
        }
    }
}

/// H moves herb patches on a stage, without waiting two game days.
fn cycle_herbs(keys: Res<ButtonInput<KeyCode>>, mut shift: ResMut<HerbGrowthShift>) {
    if keys.just_pressed(KeyCode::KeyH) {
        shift.0 = shift.0.wrapping_add(1);
    }
}

/// Moves the snow line by asking the server, whose word `Weather` holds, as it
/// decides where ice can be walked on (dev tools: only the embedded server
/// allows it). `line` keeps the fractions between whole heights.
fn temperature_cycle(
    time: Res<Time>,
    mut cycle: ResMut<TemperatureCycle>,
    weather: Res<Weather>,
    mut line: Local<f64>,
    mut to_server: MessageWriter<ToServer>,
) {
    let step = SNOW_LINE_SPEED * time.delta_secs_f64();
    match *cycle {
        TemperatureCycle::Off => {
            *line = weather.snow_line;
            return;
        }
        TemperatureCycle::Cooling => {
            *line = (*line - step).max(0.0);
            if *line == 0.0 {
                *cycle = TemperatureCycle::Warming;
            }
        }
        TemperatureCycle::Warming => {
            *line = (*line + step).min(NORMAL_SNOW_LINE);
            if *line == NORMAL_SNOW_LINE {
                *cycle = TemperatureCycle::Off;
            }
        }
    }
    let whole = line.round();
    if whole != weather.snow_line {
        to_server.write(ToServer(ClientMessage::DevSnowLine(whole as u32)));
    }
}
