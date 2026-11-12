//! Travel and energy as the client knows them, and making the player.

use bevy::prelude::*;
use shared::energy::{ACTION_TICKS_PER_SECOND, Energy};
use shared::movement::Step;
use shared::spot::Spot;
use std::collections::VecDeque;

use super::stride::Stride;
use super::{LIGHT_RADIUS, Player};
use crate::common::{Icon, LightSource, MapPosition, SmoothPosition, SpotPosition, Token};

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
    /// A closed door asked open, not yet answered: not asked again meanwhile.
    pub(super) door_asked: Option<(i64, i64)>,
    /// On a level or in the Hall: the spots a journey has still to reach, in
    /// order (each straight on from the last).
    pub(super) spots: VecDeque<Spot>,
    /// The `SpotPosition` this module last set, to notice other changes.
    pub(super) last_spot: Spot,
    /// A move already taken (and sent) to this spot, being shown: a jump, or a
    /// step asked for (`TakeStep`). Nothing else moves the player meanwhile.
    pub(super) landing: Option<Spot>,
    /// The straight stretch walked since the last move was sent.
    pub(super) stride: Stride,
    /// A route to `goal` is to be found on a level (asked for, or the way was
    /// blocked).
    pub(super) asked: bool,
    /// The route was found again because the way was blocked, and nothing
    /// has been walked since: blocked again, the journey ends.
    pub(super) rerouted: bool,
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
        SpotPosition::default(),
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

/// The player's token's base is drawn to their size.
pub(super) fn size_token(
    size: Res<crate::common::PlayerSize>,
    mut token: Single<&mut Token, With<Player>>,
) {
    if token.base != size.0 {
        token.base = size.0;
    }
}
