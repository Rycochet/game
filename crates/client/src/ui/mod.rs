//! On-screen interface (HUD, menus). See `docs/context/client/ui.md`.

use bevy::prelude::*;

use crate::common::{GameClock, ViewStats, Weather};
use crate::demo::{AutoWalk, TemperatureCycle};
use crate::player::{MapPosition, Player};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_status_text)
            .add_systems(Update, update_status_text);
    }
}

#[derive(Component)]
struct StatusText;

fn spawn_status_text(mut commands: Commands) {
    commands.spawn((
        StatusText,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

#[allow(clippy::too_many_arguments)]
fn update_status_text(
    player: Single<&MapPosition, With<Player>>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    walk: Res<AutoWalk>,
    cycle: Res<TemperatureCycle>,
    stats: Res<ViewStats>,
    mut text: Single<&mut Text, With<StatusText>>,
) {
    let on_off = |on: bool| if on { "on" } else { "off" };
    let cycle = match *cycle {
        TemperatureCycle::Off => "off",
        TemperatureCycle::Cooling => "cooling",
        TemperatureCycle::Warming => "warming",
    };
    text.0 = format!(
        "Arrow keys / WASD or click to move  M: world map / dungeon\n\
         Right-drag: turn / tilt map  Wheel: zoom  Home: reset view\n\
         G: walk ({})  T: temperature cycle ({cycle}, snow line {:.0})\n\
         K / C / X: token shape / colour / completed\n\
         Position {}, {}  Hour {}\n{}",
        on_off(walk.0),
        weather.snow_line,
        player.x,
        player.y,
        clock.hour,
        stats.summary
    );
}
