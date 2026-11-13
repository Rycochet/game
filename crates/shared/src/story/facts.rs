//! The values stories are told with, each a `lang::Selector` worded by the
//! language files (`<fact>-<key>`).

mod events;
mod people;
mod reason;
mod things;

pub use events::{Errand, Occasion, Stake, TimeAgo};
pub use people::{Passerby, Relation, State, Trait};
pub use reason::{Goal, Reason};
pub use things::{BodyPart, CountBand, ItemCategory, Remains, RewardPart, Season, Weather};
