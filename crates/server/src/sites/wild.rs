//! Players in the wilderness (`shared::wilderness`): exploring it from the
//! world map, the window of blocks a visit's level is (moved along as they
//! walk), its pois' cutouts and stairs into their sites, and leaving.
//!
//! Cells keep where they are round the planet (`0..width`) except in a window
//! lying across the seam, whose cells beyond it carry on past `width` or below
//! 0; stepping over the seam puts the player back in `0..width`, as a new map.
//! What is remembered is kept by where it is round the planet.

mod arrival;
mod chunks;
mod exits;
mod explore;
mod in_poi;
mod window;

#[cfg(test)]
mod arrival_tests;
#[cfg(test)]
mod tests;

pub(super) use chunks::ensure_chunk;
pub(crate) use exits::leave_wilderness;
pub(super) use exits::{back_to_wilderness, into_site};
pub(super) use explore::enter_wilderness;
pub(crate) use explore::{explore, follow};
pub(crate) use window::area_level;
