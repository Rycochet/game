//! Which way creatures face, and what they can see by it: a cone ahead and a
//! small circle all round (owner, 2026-11-12, reversing "nothing in the game
//! faces a way"). Integers only, so the client and the server agree. See
//! `docs/context/shared/facing.md`.

mod cone;
mod field;
mod heading;

pub use cone::Cone;
pub use field::{Field, NEAR, observation_skill};
pub use heading::{EIGHTH, Heading, TURN, UNIT};

#[cfg(test)]
mod tests;
