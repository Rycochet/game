//! The standalone Hall of Worlds server: a headless Bevy app (no window)
//! running `ServerPlugin` at a fixed tick rate, with clients connecting over
//! WebSockets (`NetworkPlugin`), which also serves the web client.
//!
//! Set by environment variables:
//!
//! - `HOW_ADDRESS`: where to listen (default `0.0.0.0:8080`).
//! - `HOW_WEB`: the web client's folder (`trunk build`'s `dist`) to
//!   serve; without it, only the game's WebSockets are served.
//! - `HOW_DATA`: where to save (default: the user's data
//!   folder).
//! - `HOW_DEBUG_TOOLS`: `1` accepts dev tools messages from anyone connected.
//! - `LOG_LEVEL`: how much to log (see `environment`); `debug` adds every
//!   connection's every round trip, for looking into network trouble.

use std::env;
use std::net::SocketAddr;

use server::{
    DEFAULT_WORLD_SEED, NetworkPlugin, ServerOptions, default_data_folder, headless_app,
    log_plugin, refuse_old_variables,
};
use shared::world::WorldSeed;

const DEFAULT_ADDRESS: &str = "0.0.0.0:8080";

fn main() {
    refuse_old_variables();
    let address: SocketAddr = env::var("HOW_ADDRESS")
        .unwrap_or_else(|_| DEFAULT_ADDRESS.into())
        .parse()
        .expect("HOW_ADDRESS is an address and port, such as 0.0.0.0:8080");
    let web = env::var_os("HOW_WEB").map(Into::into);
    let data = default_data_folder();
    let dev_tools = env::var("HOW_DEBUG_TOOLS").is_ok_and(|value| value == "1");
    let mut app = headless_app(
        WorldSeed::new(DEFAULT_WORLD_SEED),
        ServerOptions {
            dev_tools,
            exit_when_alone: false,
        },
        Some(data.clone()),
    );
    app.add_plugins((log_plugin(), NetworkPlugin { address, web }));
    bevy::log::info!("Saving in {}", data.display());
    if dev_tools {
        bevy::log::warn!("Dev tools are on: anyone connected can teleport and change the world");
    }
    app.run();
}
