//! The debug API: Bevy's remote protocol (JSON-RPC over HTTP, on localhost
//! only), for looking at and driving a running game from outside, for example
//! `cargo xtask api server.state`. Its code is only built with the
//! `debug-api` feature, which release builds refuse, and it only starts when
//! `HOW_DEBUG_API` is set. How to add a method: `docs/context/debug-api.md`.
//!
//! The native client uses this module too, for its own methods, as it already
//! depends on the server crate.

use bevy::prelude::*;

#[cfg(feature = "debug-api")]
mod methods;
#[cfg(feature = "debug-api")]
mod remote;

#[cfg(feature = "debug-api")]
pub(crate) use methods::add_server_methods;
#[cfg(feature = "debug-api")]
pub use remote::{AddDebugMethod, BrpError, BrpResult, Value, json, params, refused};

// The debug API is for development only: no release may ever have it.
#[cfg(all(feature = "debug-api", not(debug_assertions)))]
compile_error!(
    "the `debug-api` feature is for debug builds only, and must never be in a release build"
);

/// `true` for the usual port, or a port number.
pub const VARIABLE: &str = "HOW_DEBUG_API";

/// The usual port: Bevy's own default for its remote protocol.
pub const DEFAULT_PORT: u16 = 15702;

/// Starts the debug API for an app, if `HOW_DEBUG_API` asks for it. The
/// client and a standalone server listen on its port, the native client's
/// embedded server on the next one (`port_offset` 1). A port already in use
/// (another copy of the game, say) is logged, and the app starts without it.
pub struct DebugApiPlugin {
    pub port_offset: u16,
}

/// The port `HOW_DEBUG_API` asks for, if any: a number is that port, and
/// anything that turns an option on is the usual one.
pub fn port_from(value: Option<&str>) -> Option<u16> {
    let value = value?.trim();
    if let Ok(port) = value.parse::<u16>() {
        // `0` and `1` turn options off and on, and aren't ports anyone
        // would choose.
        return match port {
            0 => None,
            1 => Some(DEFAULT_PORT),
            port => Some(port),
        };
    }
    crate::environment::option_value(Some(value), false).then_some(DEFAULT_PORT)
}

/// The port this program's environment asks for, if any.
fn asked_for() -> Option<u16> {
    port_from(std::env::var(VARIABLE).ok().as_deref())
}

#[cfg(not(feature = "debug-api"))]
impl Plugin for DebugApiPlugin {
    fn build(&self, _app: &mut App) {
        if asked_for().is_some() {
            warn!(
                "{VARIABLE} is set, but this build has no debug API: build it with `--features debug-api`"
            );
        }
    }
}

#[cfg(feature = "debug-api")]
impl Plugin for DebugApiPlugin {
    fn build(&self, app: &mut App) {
        if let Some(port) = asked_for() {
            remote::start(app, port.saturating_add(self.port_offset));
        }
    }

    fn finish(&self, app: &mut App) {
        remote::register_methods(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_variable_turns_it_on_or_names_a_port() {
        assert_eq!(port_from(None), None);
        assert_eq!(port_from(Some("true")), Some(DEFAULT_PORT));
        assert_eq!(port_from(Some("YES")), Some(DEFAULT_PORT));
        assert_eq!(port_from(Some("1")), Some(DEFAULT_PORT));
        assert_eq!(port_from(Some("15800")), Some(15800));
        assert_eq!(port_from(Some("false")), None);
        assert_eq!(port_from(Some("0")), None);
        assert_eq!(port_from(Some("nonsense")), None);
    }
}
