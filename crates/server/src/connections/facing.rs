//! Which way a player faces, and what they see by it (`shared::facing`):
//! turning by moving, by asking (`ClientMessage::Face`) and towards whoever
//! they use an ability on; practising Observation by seeing.

use shared::facing::{Cone, Field, Heading, observation_skill};
use shared::spot::Spot;

use super::Client;

/// Practice for Observation each time a creature comes into sight (at most
/// one lot every 5 seconds: `Windows`).
const SEEING_PRACTICE: u32 = 60;

impl Client {
    /// What the player sees round them, by kind and Observation.
    pub(crate) fn field(&self) -> Field {
        let observation = self.sheet.skills.value(observation_skill());
        Field::of(self.inventory.creature, observation)
    }

    /// The player's field of view on a level or in the Hall, where they stand;
    /// `None` on the world map, which is seen all round.
    pub(crate) fn cone(&self) -> Option<Cone> {
        let at = match (&self.visit, self.character.hall) {
            (Some(visit), _) => visit.spot(),
            (None, Some(at)) => at,
            (None, None) => return None,
        };
        Some(Cone::new(at, self.heading, self.field()))
    }

    /// Turns to face `heading` without moving. What they see is worked out
    /// again on a level. `false` if the client has gone.
    pub(crate) fn face(&mut self, heading: Heading) -> bool {
        if heading == self.heading {
            return true;
        }
        self.heading = heading;
        crate::sites::update_sight(self)
    }

    /// Turns towards `to` from where they stand on their level, if it is
    /// somewhere else. `false` if the client has gone.
    pub(crate) fn face_spot(&mut self, to: Spot) -> bool {
        let Some(from) = self.visit.as_ref().map(|visit| visit.spot()) else {
            return true;
        };
        Heading::towards(from, to).is_none_or(|heading| self.face(heading))
    }

    /// Where the player is now, in the map's measure, for turning the way
    /// they moved (`moved_from`).
    pub(super) fn standing(&self) -> (i64, i64) {
        match (&self.visit, self.character.hall) {
            (Some(visit), _) => (visit.spot().x, visit.spot().y),
            (None, Some(at)) => (at.x, at.y),
            (None, None) => self.character.world_position,
        }
    }

    /// After a move or a step: faces the way they went from `from`, if they
    /// went anywhere on the same map (a jump across the world map's seam
    /// keeps the way they faced).
    pub(super) fn moved_from(&mut self, from: (i64, i64), map: u32) {
        let to = self.standing();
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        if map != self.map || dx.abs() > SEAM || dy.abs() > SEAM {
            return;
        }
        if let Some(heading) = Heading::of(dx, dy) {
            self.heading = heading;
        }
    }

    /// Whether they are winding up an ability: a noise mobs may hear.
    pub(crate) fn winding_up(&self) -> bool {
        self.fight.pending.is_some()
    }

    /// Practises Observation for `seen` creatures that have just come into
    /// sight, at action tick `now`.
    pub(crate) fn practise_seeing(&mut self, seen: usize, now: u64) {
        if seen == 0 {
            return;
        }
        let skill = observation_skill();
        let amount = self.learning.skills.take(skill, SEEING_PRACTICE, now);
        if amount > 0 {
            self.sheet.skills.practise(skill, amount);
            self.sheet.skills.used(skill, self.activity.played);
            self.unsaved = true;
        }
    }
}

/// Further than any one move goes, in the map's measure: a move this long
/// went round the world map's seam (or was a teleport).
const SEAM: i64 = 4 * shared::spot::CELL_SPAN;
