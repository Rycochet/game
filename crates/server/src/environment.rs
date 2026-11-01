//! What both native programs read from the environment before anything else:
//! the log level, and a refusal to run on the old variable names.

use std::env;
use std::process;

use bevy::log::{DEFAULT_FILTER, Level, LogPlugin};

/// How much to log: `error`, `warn`, `info` (the default), `debug` or `trace`.
const LOG_LEVEL_VARIABLE: &str = "LOG_LEVEL";

/// The crates `debug` and `trace` apply to, so other people's don't bury our
/// lines. `RUST_LOG` overrides all this, for filtering any crate.
const OUR_CRATES: [&str; 4] = [
    "hall_of_worlds",
    "hall_of_worlds_server",
    "server",
    "shared",
];

/// Logs turned off. The operating system's certificate check logs an error for
/// every self-signed certificate, which is normal here; `connection::trust`
/// says what matters.
const SILENCED: &str = "rustls_platform_verifier=off";

/// The start of the variable names used before `HOW_`.
const OLD_PREFIX: &str = "HALL_OF_WORLDS_";

/// The logging settings `LOG_LEVEL` asks for.
pub fn log_plugin() -> LogPlugin {
    log_plugin_for(env::var(LOG_LEVEL_VARIABLE).ok().as_deref())
}

fn log_plugin_for(value: Option<&str>) -> LogPlugin {
    let level = value.map_or(Level::INFO, |value| {
        value.parse().unwrap_or_else(|_| {
            // Logging hasn't started yet.
            eprintln!("{LOG_LEVEL_VARIABLE}={value} is not a log level, so using info");
            Level::INFO
        })
    });
    if level == Level::DEBUG || level == Level::TRACE {
        let ours = OUR_CRATES.map(|name| format!("{name}={level}")).join(",");
        LogPlugin {
            level: Level::INFO,
            filter: format!("{DEFAULT_FILTER},{SILENCED},{ours}"),
            ..Default::default()
        }
    } else {
        LogPlugin {
            level,
            filter: format!("{DEFAULT_FILTER},{SILENCED}"),
            ..Default::default()
        }
    }
}

/// Stop if an old variable name is set. Ignoring one could mean quietly saving
/// somewhere else, which looks like lost characters or overwrites real ones.
pub fn refuse_old_variables() {
    let old: Vec<String> = env::vars_os()
        .filter_map(|(name, _)| name.into_string().ok())
        .filter(|name| name.starts_with(OLD_PREFIX))
        .collect();
    if old.is_empty() {
        return;
    }
    for name in &old {
        eprintln!("{name} is no longer read: use {}", replacement(name));
    }
    process::exit(1);
}

/// What replaced an old variable.
fn replacement(old: &str) -> &'static str {
    match old.strip_prefix(OLD_PREFIX) {
        Some("ADDRESS") => "HOW_ADDRESS",
        Some("WEB") => "HOW_WEB",
        Some("SERVER_DATA") => "HOW_DATA",
        Some("SERVER") => "HOW_SERVER",
        Some("DEV_TOOLS") => "HOW_DEBUG_TOOLS",
        Some("LOG_PINGS") => "LOG_LEVEL=debug",
        _ => "the HOW_ names, or remove it",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_or_unknown_levels_log_as_usual() {
        for value in [None, Some("loud"), Some("")] {
            let plugin = log_plugin_for(value);
            assert_eq!(plugin.level, Level::INFO);
            assert_eq!(plugin.filter, format!("{DEFAULT_FILTER},{SILENCED}"));
        }
    }

    #[test]
    fn quieter_levels_apply_to_everything() {
        let plugin = log_plugin_for(Some("WARN"));
        assert_eq!(plugin.level, Level::WARN);
        assert_eq!(plugin.filter, format!("{DEFAULT_FILTER},{SILENCED}"));
    }

    #[test]
    fn louder_levels_apply_only_to_our_crates() {
        let plugin = log_plugin_for(Some("debug"));
        assert_eq!(plugin.level, Level::INFO);
        assert!(plugin.filter.starts_with(DEFAULT_FILTER));
        assert!(plugin.filter.contains("server=DEBUG"));
    }

    #[test]
    fn old_variables_name_their_replacements() {
        assert_eq!(replacement("HALL_OF_WORLDS_SERVER_DATA"), "HOW_DATA");
        assert_eq!(replacement("HALL_OF_WORLDS_LOG_PINGS"), "LOG_LEVEL=debug");
    }
}
