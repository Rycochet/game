//! How big stacks of items may get on this server, from `HOW_STACK_GRAMS_PER_CELL`
//! and `HOW_STACK_MAX`. Clients are told in `Welcome`.

use std::env;

use bevy::log::warn;
use shared::item::StackLimits;

/// The limits the environment asks for, or the usual ones.
pub(crate) fn from_environment() -> StackLimits {
    from_values(
        env::var("HOW_STACK_GRAMS_PER_CELL").ok().as_deref(),
        env::var("HOW_STACK_MAX").ok().as_deref(),
    )
}

/// The limits these values ask for. Anything that isn't a whole number of at least
/// 1 is warned about, and leaves the usual limit.
fn from_values(grams_per_cell: Option<&str>, most: Option<&str>) -> StackLimits {
    let usual = StackLimits::default();
    StackLimits {
        grams_per_cell: read("HOW_STACK_GRAMS_PER_CELL", grams_per_cell)
            .unwrap_or(usual.grams_per_cell),
        most: read("HOW_STACK_MAX", most).unwrap_or(usual.most),
    }
}

fn read(variable: &str, value: Option<&str>) -> Option<u32> {
    let value = value?;
    match value.trim().parse() {
        Ok(number) if number >= 1 => Some(number),
        _ => {
            warn!("{variable} is a whole number of at least 1, not {value:?}: using the usual");
            None
        }
    }
}

#[cfg(test)]
mod tests;
