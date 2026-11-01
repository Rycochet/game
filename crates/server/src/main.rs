//! The standalone Hall of Worlds server: a headless Bevy app (no window)
//! running `ServerPlugin` at a fixed tick rate, with clients connecting over
//! WebSockets (`NetworkPlugin`), which also serves the web client.
//!
//! Set by environment variables:
//!
//! - `HOW_ADDRESS`: the plain port (default `0.0.0.0:8080`), which only answers
//!   that connections must be secure, unless `HOW_BEHIND_PROXY` is set.
//! - `HOW_TLS_ADDRESS`: the secure port for HTTPS and `wss://` (default
//!   `0.0.0.0:8081`).
//! - `HOW_BEHIND_PROXY`: `true` turns TLS off and serves everything on the plain
//!   port, for a server behind a reverse proxy that looks after TLS.
//! - `HOW_TLS_CERTIFICATE` and `HOW_TLS_KEY`: the paths of your own PEM
//!   certificate and private key. Without them, a self-signed certificate is
//!   made and kept in the data folder.
//! - `HOW_TLS_NAMES`: names and addresses a made certificate covers as well as
//!   this machine, separated by commas.
//! - `HOW_WEB`: the web client's folder (`trunk build`'s `dist`) to
//!   serve; without it, only the game's WebSockets are served.
//! - `HOW_DATA`: where to save (default: the user's data
//!   folder).
//! - `HOW_DEBUG_TOOLS`: `true` accepts dev tools messages from anyone connected.
//! - `LOG_LEVEL`: how much to log (see `environment`); `debug` adds every
//!   connection's every round trip, for looking into network trouble.

use std::env;
use std::net::SocketAddr;
use std::process;

use server::{
    DEFAULT_WORLD_SEED, NetworkPlugin, Security, ServerOptions, default_data_folder, headless_app,
    log_plugin, option_on, refuse_old_variables,
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
    let security = Security::from_environment(&data).unwrap_or_else(|error| {
        // Logging hasn't started yet.
        eprintln!("{error}");
        process::exit(1);
    });
    let dev_tools = option_on("HOW_DEBUG_TOOLS", false);
    let mut app = headless_app(
        WorldSeed::new(DEFAULT_WORLD_SEED),
        ServerOptions {
            dev_tools,
            exit_when_alone: false,
        },
        Some(data.clone()),
    );
    app.add_plugins((
        log_plugin(),
        NetworkPlugin {
            address,
            web,
            security,
        },
    ));
    bevy::log::info!("Saving in {}", data.display());
    if dev_tools {
        bevy::log::warn!("Dev tools are on: anyone connected can teleport and change the world");
    }
    app.run();
}
