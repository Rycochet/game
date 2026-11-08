//! The player-controlled character: keyboard steps and travelling to a
//! clicked point. See `docs/context/client/player.md`.
//!
//! Every step the player takes is announced (`Stepped`), for `connection` to
//! send to the server, which checks it: what is shown here is a prediction
//! the server may put right.

use bevy::prelude::*;

use crate::common::{CancelledEnergy, GameView, ServerStatus, SpentEnergy};

// Split into files (`cargo xtask lint` keeps files small).
mod keys;
mod messages;
mod moving;
mod state;
mod steps;
mod travel;
mod wild_heights;

use keys::move_player;
pub use messages::{Arrived, Player, PlayerSystems, Stepped, TakeStep, TravelTo};
pub use moving::PlayerMoving;
use moving::note_moving;
use state::{PlayerEnergy, Travel, action_tick, spawn_player};
use steps::take_steps;
use travel::{enemy_cells, travel};
use wild_heights::keep_wild_heights;

/// How far the player's light reaches in dungeons, in cells.
const LIGHT_RADIUS: u32 = 6;

/// Travel speed towards a clicked point at `NORMAL_SPEED`, in grid units per
/// second: world map points, or dungeon cells (which are much bigger on screen).
/// What `shared::energy`'s costs come to, so the server has time for every step.
const TRAVEL_SPEED: f64 = 60.0;
const DUNGEON_TRAVEL_SPEED: f64 = shared::energy::SITE_CELLS_PER_SECOND;

/// How many asked-for steps can wait for energy here.
const MOST_STEPS_WAITING: usize = 8;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEnergy>()
            .init_resource::<PlayerMoving>()
            .add_message::<TravelTo>()
            .add_message::<Arrived>()
            .add_message::<TakeStep>()
            .add_message::<Stepped>()
            .add_message::<SpentEnergy>()
            .add_message::<CancelledEnergy>()
            .configure_sets(
                Update,
                (PlayerSystems::Input, PlayerSystems::Travel).chain(),
            )
            .add_systems(Startup, spawn_player)
            // Nowhere to move until the server has said which world this is.
            .add_systems(
                Update,
                (
                    move_player.in_set(PlayerSystems::Input),
                    // After everything in `Input` asked for its steps.
                    (keep_wild_heights, take_steps, travel, note_moving)
                        .chain()
                        .in_set(PlayerSystems::Travel),
                )
                    .run_if(not(in_state(GameView::Connecting)))
                    // Nobody to tell while the connection is down.
                    .run_if(|status: Res<ServerStatus>| status.is_connected())
                    // Nobody to move while a new character is being made.
                    .run_if(|making: Res<crate::common::MakingCharacter>| !making.0),
            );
    }
}
