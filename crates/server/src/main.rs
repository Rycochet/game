//! The standalone Hall of Worlds server: a headless Bevy app (no window)
//! running `ServerPlugin` at a fixed tick rate, with clients connecting over
//! WebSockets (`NetworkPlugin`), which also serves the web client.
//!
//! `hall-of-worlds-server --health` asks a server already running here (on
//! `HOW_ADDRESS`) whether it is well, and exits with 0 if it is: Docker's
//! HEALTHCHECK.
//!
//! Set by environment variables:
//!
//! - `HOW_ADDRESS`: the plain port (default `0.0.0.0:24690`), which only answers
//!   that connections must be secure, unless `HOW_BEHIND_PROXY` is on.
//! - `HOW_TLS_ADDRESS`: the secure port for HTTPS and `wss://` (default
//!   `0.0.0.0:24691`).
//! - `HOW_BEHIND_PROXY`: `true` also serves everything on the plain port, for a
//!   reverse proxy to reach. The secure port stays open.
//! - `HOW_DISABLE_SSL`: `true` doesn't open the secure port or set up a
//!   certificate. It needs `HOW_BEHIND_PROXY`, or the server refuses to start.
//! - `HOW_TLS_CERTIFICATE` and `HOW_TLS_KEY`: the paths of your own PEM
//!   certificate and private key. Without them, a self-signed certificate is
//!   made and kept in the data folder.
//! - `HOW_LETS_ENCRYPT_DOMAIN`: get a certificate for this domain from Let's
//!   Encrypt, which accepts its terms. The domain's public port 443 must reach
//!   the secure port. A self-signed certificate is used until it comes.
//! - `HOW_LETS_ENCRYPT_EMAIL`: where Let's Encrypt can write to you (optional).
//! - `HOW_DEBUG_LETS_ENCRYPT_STAGING`: `true` uses Let's Encrypt's test service,
//!   whose certificates nobody trusts, to try things without its rate limits.
//! - `HOW_UPNP`: `true` asks the router to forward the secure port (443 with
//!   Let's Encrypt) while the server runs. Ignored, with a warning, behind a
//!   reverse proxy.
//! - `HOW_TLS_NAMES`: names and addresses a made certificate covers as well as
//!   this machine, separated by commas.
//! - `HOW_WEB`: the web client's folder (`trunk build`'s `dist`) to
//!   serve; without it, only the game's WebSockets are served. The last few
//!   builds served are kept in the data folder's `web-history`, for deltas.
//! - `HOW_DATA`: where to save (default: the user's data
//!   folder).
//! - `HOW_GUESTS`: `true` (the default) lets clients play without an account.
//!   `false` stops the server from starting, until there is another way in.
//! - `HOW_DEBUG_TOOLS`: `true` accepts dev tools messages from anyone connected.
//! - `HOW_SERVER_NAME`: what the server calls itself (default `Hall of Worlds`).
//! - `HOW_PLAYER_LIMIT`: the most players at once (default 0, no limit).
//! - `HOW_IDLE_MINUTES`: minutes a player may do nothing before being
//!   disconnected (default 15, 0 for never).
//! - `HOW_STACK_GRAMS_PER_CELL`: the most a stack of items may weigh for each cell
//!   it takes up, in grams (default 2000).
//! - `HOW_STACK_MAX`: the most items in one stack (default 99).
//! - `LOG_LEVEL`: how much to log (see `environment`); `debug` adds every
//!   connection's every round trip, for looking into network trouble.

use std::env;
use std::net::SocketAddr;
use std::process;

use server::{
    DEFAULT_WORLD_SEED, IDLE_MINUTES, NetworkPlugin, Security, ServerName, ServerOptions,
    UpnpChoice, UpnpPlugin, WebClient, check_health, debug_api::DebugApiPlugin,
    default_data_folder, headless_app, log_plugin, option_on,
};
use shared::world::WorldSeed;

const DEFAULT_ADDRESS: &str = "0.0.0.0:24690";

fn main() {
    let address: SocketAddr = env::var("HOW_ADDRESS")
        .unwrap_or_else(|_| DEFAULT_ADDRESS.into())
        .parse()
        .expect("HOW_ADDRESS is an address and port, such as 0.0.0.0:24690");
    if env::args().nth(1).as_deref() == Some("--health") {
        match check_health(address) {
            Ok(()) => process::exit(0),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
    }
    let data = default_data_folder();
    // Past builds of the web client are kept beside the saves, to send
    // browsers that have one only what changed.
    let web = env::var_os("HOW_WEB").map(|files| WebClient {
        files: files.into(),
        history: Some(data.join("web-history")),
    });
    let security = Security::from_environment(&data).unwrap_or_else(|error| {
        // Logging hasn't started yet.
        eprintln!("{error}");
        process::exit(1);
    });
    let upnp = UpnpPlugin::from_environment(&security);
    let guests = option_on("HOW_GUESTS", true);
    if !guests {
        eprintln!("HOW_GUESTS is false, which leaves no way to log in to this server yet");
        process::exit(1);
    }
    let dev_tools = option_on("HOW_DEBUG_TOOLS", false);
    let player_limit = match env::var("HOW_PLAYER_LIMIT") {
        Ok(limit) => limit.trim().parse().unwrap_or_else(|_| {
            eprintln!("HOW_PLAYER_LIMIT is a number of players, 0 for no limit, not {limit:?}");
            process::exit(1);
        }),
        Err(_) => 0,
    };
    let idle_minutes = match env::var("HOW_IDLE_MINUTES") {
        Ok(minutes) => minutes.trim().parse().unwrap_or_else(|_| {
            eprintln!("HOW_IDLE_MINUTES is a number of minutes, 0 for never, not {minutes:?}");
            process::exit(1);
        }),
        Err(_) => IDLE_MINUTES,
    };
    let mut app = headless_app(
        WorldSeed::new(DEFAULT_WORLD_SEED),
        ServerOptions {
            dev_tools,
            exit_when_alone: false,
            pausable: false,
            guests,
            player_limit,
            idle_minutes,
            ..Default::default()
        },
        Some(data.clone()),
    );
    if let Ok(name) = env::var("HOW_SERVER_NAME") {
        app.insert_resource(ServerName(name));
    }
    app.add_plugins((
        log_plugin(),
        NetworkPlugin {
            address,
            web,
            security,
        },
        // After the log plugin, so what it says is logged.
        DebugApiPlugin { port_offset: 0 },
    ));
    match upnp {
        UpnpChoice::Forward(plugin) => {
            app.add_plugins(plugin);
        }
        UpnpChoice::BehindProxy => bevy::log::warn!(
            "HOW_UPNP is ignored behind a reverse proxy (HOW_BEHIND_PROXY): forward the proxy's port instead"
        ),
        UpnpChoice::Off => {}
    }
    bevy::log::info!("Saving in {}", data.display());
    if dev_tools {
        bevy::log::warn!("Dev tools are on: anyone connected can teleport and change the world");
    }
    app.run();
}
