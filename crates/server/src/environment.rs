//! What both native programs read from the environment: the log level, a
//! refusal to run on the old variable names, yes-or-no options and secrets.

use std::{env, fs};

use bevy::log::{DEFAULT_FILTER, Level, LogPlugin, info, warn};
use shared::difficulty::Difficulties;

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

/// Logs turned off. The operating system's certificate check (used in reaching
/// Let's Encrypt) logs an error for every certificate it can't vouch for, which
/// says nothing useful here.
const SILENCED: &str = "rustls_platform_verifier=off";

/// The values that turn on an option that is off by default.
const TRUE_VALUES: [&str; 4] = ["1", "on", "true", "yes"];

/// The values that turn off an option that is on by default.
const FALSE_VALUES: [&str; 4] = ["0", "off", "no", "false"];

/// Whether a yes-or-no option is on. Only the values that change it count, in
/// any case; anything else, or not being set, leaves `default`.
pub fn option_on(variable: &str, default: bool) -> bool {
    option_value(env::var(variable).ok().as_deref(), default)
}

pub(crate) fn option_value(value: Option<&str>, default: bool) -> bool {
    let Some(value) = value else {
        return default;
    };
    let changes_it = if default { FALSE_VALUES } else { TRUE_VALUES };
    let changed = changes_it
        .iter()
        .any(|word| word.eq_ignore_ascii_case(value));
    default != changed
}

/// A secret (`HOW_..._SECRET`): the contents of the file `<name>_FILE` names,
/// if that is set, which wins over `<name>` itself (for Docker's secrets and the
/// like). `None` if neither is set; blank counts as not set, and the spaces and
/// line ends round a value are left out. An error if the file can't be read.
pub(crate) fn secret(name: &str) -> Result<Option<String>, String> {
    secret_value(name, |variable| env::var(variable).ok())
}

/// `secret`, with the variables from `get`.
pub(crate) fn secret_value(
    name: &str,
    get: impl Fn(&str) -> Option<String>,
) -> Result<Option<String>, String> {
    let set = |variable: &str| get(variable).filter(|value| !value.trim().is_empty());
    let file = format!("{name}_FILE");
    let Some(path) = set(&file) else {
        return Ok(set(name).map(|value| value.trim().to_owned()));
    };
    if set(name).is_some() {
        info!("{file} and {name} are both set: the file is used");
    }
    let path = path.trim();
    let text =
        fs::read_to_string(path).map_err(|error| format!("{file}: can't read {path} ({error})"))?;
    Ok(Some(text.trim().to_owned()).filter(|text| !text.is_empty()))
}

/// `HOW_UPNP`, if set: whether to ask the router to forward the port. A
/// standalone server reads it alone (off if unset); in a native client it wins
/// over Server Options' switch.
pub fn upnp_from_environment() -> Option<bool> {
    upnp_value(env::var("HOW_UPNP").ok().as_deref())
}

pub(crate) fn upnp_value(value: Option<&str>) -> Option<bool> {
    value.map(|value| option_value(Some(value), false))
}

/// `HOW_DIFFICULTIES`, if set to anything but blank: the difficulties allowed,
/// as names separated by commas. Unknown names are warned about (so read it
/// once logging has started) and left out; with none known, all are allowed.
/// A standalone server allows all without it; in a native client it wins over
/// Server Options' tickboxes.
pub fn difficulties_from_environment() -> Option<Difficulties> {
    difficulties_value(env::var("HOW_DIFFICULTIES").ok().as_deref())
}

pub(crate) fn difficulties_value(value: Option<&str>) -> Option<Difficulties> {
    let value = value.filter(|value| !value.trim().is_empty())?;
    let (allowed, unknown) = Difficulties::parse(value);
    for name in unknown {
        warn!("HOW_DIFFICULTIES: {name:?} isn't a difficulty, so it is ignored");
    }
    Some(allowed)
}

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

#[cfg(test)]
mod tests;
