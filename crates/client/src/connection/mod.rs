//! The client's connection to the server. See
//! `docs/context/client/connection.md`.
//!
//! Native clients start an embedded server for single player (on its own
//! thread, with its own Bevy world) and talk to it through an in-process
//! `Link`, using the same messages a network connection carries; the web
//! client connects to the server it was loaded from over a WebSocket
//! (`socket`). A lost connection is tried again every few seconds
//! (`ServerStatus` says how it is going). The embedded server saves in its own
//! data folder; on quitting, the client hangs up and waits for it to save.
//!
//! It sends the player's steps (`player::Stepped`); the server checks them,
//! sends what comes into sight and says where it has the player, and the
//! player is moved back if a step was refused (`prediction`). Other modules
//! send with `ToServer` and read the server's map messages as `FromServer`.

mod guest;
mod ping;
mod prediction;
mod probe;
mod socket;
#[cfg(not(target_arch = "wasm32"))]
mod trust;

use bevy::prelude::*;
use prediction::Prediction;
use socket::Target;

use crate::common::{ChosenServer, CurrentWorld, Session};
use crate::player::PlayerSystems;

// Split into files (`cargo xtask lint` keeps files small).
mod characters;
mod clock;
mod connect;
mod difficulty;
#[cfg(not(target_arch = "wasm32"))]
mod hosting;
mod log_in;
mod log_out;
mod messages;
mod receive;
mod remember;
mod saved_login;
mod send;

use characters::{answer_characters, choose_character, show_playable};
use clock::{apply_pong, run_clock, send_local_pings, stop_game_time};
#[cfg(target_arch = "wasm32")]
use connect::pump_socket;
use connect::{Connecting, connect, give_up, hang_up};
use log_in::{answer_login, choose_login, show_account};
use log_out::{back_at_start, log_out, now_playing, start_playing};
pub use messages::{FromServer, ServerConnection, ToServer};
pub use probe::{ProbeResult, ServerProbe};
use receive::receive;
use remember::{forget_when_unchosen, remember_server};
use send::{count_map_changes, hang_up_on_exit, send, send_steps};
#[cfg(not(target_arch = "wasm32"))]
pub use trust::server_key;

pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToServer>()
            .add_message::<FromServer>()
            .init_resource::<Prediction>()
            .insert_resource(Connecting {
                target: Target::choose(&ChosenServer::default(), true),
                environment: true,
                guest: None,
                methods: Vec::new(),
                tried_session: false,
                offered: false,
                retry: None,
            })
            .add_systems(
                PreUpdate,
                (
                    log_out.run_if(back_at_start),
                    start_playing.run_if(now_playing),
                    connect
                        .run_if(not(resource_exists::<ServerConnection>))
                        .run_if(|session: Res<Session>| session.is_playing()),
                    receive.run_if(resource_exists::<ServerConnection>),
                    answer_login.run_if(resource_exists::<ServerConnection>),
                    answer_characters.run_if(resource_exists::<ServerConnection>),
                    stop_game_time,
                )
                    .chain(),
            )
            .add_systems(Update, send_steps.after(PlayerSystems::Travel))
            .add_systems(
                Update,
                (
                    remember_server,
                    forget_when_unchosen,
                    difficulty::keep_difficulty,
                    choose_login,
                    show_account,
                    choose_character,
                    show_playable,
                ),
            )
            .add_systems(
                Update,
                (
                    (run_clock, apply_pong).chain(),
                    send_local_pings.run_if(resource_exists::<CurrentWorld>),
                ),
            )
            // After this frame's steps, which were still on the last map.
            .add_systems(PostUpdate, count_map_changes)
            // Last, so everything written this frame goes, in order.
            .add_systems(
                Last,
                (send, hang_up_on_exit)
                    .chain()
                    .run_if(resource_exists::<ServerConnection>),
            );
        #[cfg(target_arch = "wasm32")]
        app.add_systems(Last, pump_socket.after(send));
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, (hosting::ask_to_host, difficulty::ask_difficulties));
    }
}
