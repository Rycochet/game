//! What a mob remembers about characters: how it stands with each (friendly,
//! neutral or unfriendly), what it thinks of them (its opinion, a number), and
//! who has lately come into its territory.
//!
//! Standing and opinion are kept apart (owner, 2026-11-07): you can dislike
//! someone without going for them. Being hit lowers the standing a step at a
//! time, and once someone has hit it, any further blow makes it unfriendly.
//! Gifts only raise the opinion; how opinion moves standing is for later.

use serde::{Deserialize, Serialize};

use crate::item::ItemKind;
use crate::protocol::CharacterId;
use crate::sheet::Feeling;
use crate::vitals::{PER_POINT, Points};

/// How many things a mob remembers: the oldest go first, but never how it
/// stands with someone.
pub(super) const MOST_REMEMBERED: usize = 32;

/// How long, in action ticks, a mob stays angry at someone who came into its
/// territory: 3 minutes.
pub const TRESPASS_FORGOTTEN_AFTER: u64 = 3 * 60 * 20;

/// How much a gift raises its opinion, in thousandths of a Health point.
pub const GIFT_OPINION: i64 = 10 * PER_POINT as i64;

/// The least a first blow lowers its opinion by, however light (a blow that
/// does no harm still counts), in thousandths of a Health point.
pub const FIRST_HARM_OPINION: u32 = 10 * PER_POINT;

/// Something a mob remembers about a character.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// Older saves only: a gift, counted into the opinion when next read.
    Gave(ItemKind),
    /// Came into its territory, at this action tick.
    Trespassed(u64),
    /// How it stands with them and what it thinks of them, once that has
    /// changed. Only one per character, and never forgotten.
    Regard(Regard),
}

/// How a mob stands with a character, and what it thinks of them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Regard {
    /// `None` until something changes it: how its kind feels about theirs.
    pub standing: Option<Feeling>,
    /// What it thinks of them, in thousandths of a Health point. Never sets
    /// the standing by itself.
    pub opinion: i64,
    /// They have hit it: it gives them no further chance.
    pub harmed: bool,
}

/// What a mob remembers, about which characters.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory(Vec<(CharacterId, Event)>);

impl Memory {
    /// Keeps something new, letting go of the oldest thing that isn't a regard
    /// if it remembers too much.
    fn push(&mut self, about: CharacterId, event: Event) {
        if self.0.len() >= MOST_REMEMBERED {
            let oldest = self
                .0
                .iter()
                .position(|(_, old)| !matches!(old, Event::Regard(_)));
            if let Some(oldest) = oldest {
                self.0.remove(oldest);
            }
        }
        self.0.push((about, event));
    }

    /// Its regard for this character, made if new, with any gifts from older
    /// saves counted into it.
    fn regard_mut(&mut self, about: CharacterId) -> &mut Regard {
        let gifts = self
            .0
            .iter()
            .filter(|(who, event)| *who == about && matches!(event, Event::Gave(_)));
        let gifts = gifts.count() as i64;
        self.0
            .retain(|(who, event)| !(*who == about && matches!(event, Event::Gave(_))));
        let found = self
            .0
            .iter()
            .position(|(who, event)| *who == about && matches!(event, Event::Regard(_)));
        let index = found.unwrap_or_else(|| {
            self.push(about, Event::Regard(Regard::default()));
            self.0.len() - 1
        });
        let Event::Regard(regard) = &mut self.0[index].1 else {
            unreachable!("found as a regard");
        };
        regard.opinion += gifts * GIFT_OPINION;
        regard
    }

    fn regard(&self, about: &CharacterId) -> Option<Regard> {
        self.0.iter().find_map(|(who, event)| match event {
            Event::Regard(regard) if who == about => Some(*regard),
            _ => None,
        })
    }

    /// They gave it something: it thinks better of them.
    pub fn remember_gift(&mut self, about: CharacterId) {
        self.regard_mut(about).opinion += GIFT_OPINION;
    }

    /// Their blow landed on it, doing `harm`. `start` is how its kind feels
    /// about theirs. A first blow lowers its standing a step (friendly to
    /// neutral, neutral to unfriendly) and its opinion by at least
    /// `FIRST_HARM_OPINION` (for a neutral one, sets it to minus that); any
    /// later blow makes it unfriendly and lowers its opinion by the harm.
    pub fn remember_harm(&mut self, about: CharacterId, harm: Points, start: Feeling) {
        let first = i64::from(harm.milli().max(FIRST_HARM_OPINION));
        let regard = self.regard_mut(about);
        let standing = regard.standing.unwrap_or(start);
        if regard.harmed {
            regard.standing = Some(Feeling::Unfriendly);
            regard.opinion -= i64::from(harm.milli());
        } else {
            match standing {
                Feeling::Friendly => {
                    regard.standing = Some(Feeling::Neutral);
                    regard.opinion -= first;
                }
                Feeling::Neutral | Feeling::None => {
                    regard.standing = Some(Feeling::Unfriendly);
                    regard.opinion = -first;
                }
                Feeling::Unfriendly => {
                    regard.standing = Some(Feeling::Unfriendly);
                    regard.opinion -= first;
                }
            }
        }
        regard.harmed = true;
    }

    /// They came into its territory at this tick. Only the latest trespass is
    /// kept, so staying put doesn't fill its memory.
    pub fn remember_trespass(&mut self, about: CharacterId, tick: u64) {
        self.0
            .retain(|(who, old)| !(*who == about && matches!(old, Event::Trespassed(_))));
        self.push(about, Event::Trespassed(tick));
    }

    /// Lets go of trespasses older than `TRESPASS_FORGOTTEN_AFTER` at this tick.
    pub fn forget_old(&mut self, now: u64) {
        self.0.retain(|(_, event)| match event {
            Event::Trespassed(at) => now.saturating_sub(*at) < TRESPASS_FORGOTTEN_AFTER,
            Event::Gave(_) | Event::Regard(_) => true,
        });
    }

    /// How it stands with this character, whose kind its own feels `start`
    /// about: unfriendly while it remembers a trespass, otherwise as it has
    /// come to, or `start` if nothing has changed it.
    pub fn standing(&self, about: &CharacterId, start: Feeling) -> Feeling {
        let trespassed = self
            .0
            .iter()
            .any(|(who, event)| who == about && matches!(event, Event::Trespassed(_)));
        if trespassed {
            return Feeling::Unfriendly;
        }
        self.regard(about)
            .and_then(|regard| regard.standing)
            .unwrap_or(start)
    }

    /// What it thinks of this character, in thousandths of a Health point.
    pub fn opinion(&self, about: &CharacterId) -> i64 {
        let gifts = self
            .0
            .iter()
            .filter(|(who, event)| who == about && matches!(event, Event::Gave(_)));
        let regard = self.regard(about).map_or(0, |regard| regard.opinion);
        regard + gifts.count() as i64 * GIFT_OPINION
    }
}

#[cfg(test)]
mod tests;
