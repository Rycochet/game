//! Travel and energy as the client knows them, and making the player.

use bevy::prelude::*;
use shared::energy::{ACTION_TICKS_PER_SECOND, Energy};
use shared::movement::Step;
use std::collections::VecDeque;

use super::{LIGHT_RADIUS, Player};
use crate::common::{Icon, LightSource, MapPosition, SmoothPosition, Token};

/// Travel state, private to this module.
#[derive(Component, Default)]
pub(super) struct Travel {
    /// The points a journey has still to reach, in order.
    pub(super) route: VecDeque<(i64, i64)>,
    /// Where the journey was asked to go, to find a new route if the tide cuts
    /// this one.
    pub(super) goal: (i64, i64),
    /// Whether the route ends where the journey was asked to go.
    pub(super) route_arrives: bool,
    /// The `MapPosition` this module last set, to notice other changes.
    pub(super) last: MapPosition,
    /// The cell a jump over liquid is landing on. The player is already there
    /// as far as the game goes, and is shown getting there: until they land
    /// they can't take another step or change direction.
    pub(super) jump: Option<MapPosition>,
}

/// The player's energy as far as the client can tell, the same way the server
/// does, so steps aren't shown sooner than it will allow them.
#[derive(Resource, Default)]
pub(super) struct PlayerEnergy {
    pub(super) energy: Energy,
    /// The speed the energy has been going at.
    pub(super) speed: u32,
    /// Steps asked for and waiting for energy, in order.
    pub(super) waiting: VecDeque<Step>,
    /// How long a journey has to stop for, in seconds, for what was spent on other
    /// things: it is paced by distance, not energy.
    pub(super) stopped_for: f64,
}

/// Which tick of action time it is here. Stops when the game does.
pub(super) fn action_tick(time: &Time) -> u64 {
    (time.elapsed_secs_f64() * ACTION_TICKS_PER_SECOND) as u64
}

pub(super) fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        MapPosition::default(),
        SmoothPosition::default(),
        Travel::default(),
        // A white circle (no attitude): drawn as a cylinder on the map.
        Token::default(),
        Icon("monsters/humanarcher.png".into()),
        LightSource {
            radius: LIGHT_RADIUS,
            colour: Color::WHITE,
        },
    ));
}
