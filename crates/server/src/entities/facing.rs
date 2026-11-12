//! Which way mobs face (`shared::facing`): the way they go, whoever they
//! strike at, whoever strikes them, and noises they hear while at rest.

use shared::creature::Acuity;
use shared::facing::{Cone, Field, Heading};
use shared::spot::{CELL_SPAN, Spot as Units};

use super::{Doing, Mob, Spot};

impl Mob {
    /// What it sees round it: its kind's cone (mobs have no Observation yet).
    pub(super) fn cone(&self) -> Cone {
        Cone::new(self.spot, self.heading, Field::of(self.kind, 0))
    }

    /// Turns towards `to`, if it is somewhere else.
    pub(crate) fn face(&mut self, to: Units) {
        if let Some(heading) = Heading::towards(self.spot, to) {
            self.heading = heading;
        }
    }

    /// After its turn, having stood at `was`: faces whoever it is winding up
    /// at, else the way it went.
    pub(super) fn turn_after(&mut self, was: Units, spots: &[Spot]) {
        let striking = self.fight.pending.as_ref().map(|pending| pending.target);
        if let Some(them) = striking.and_then(|id| spots.iter().find(|spot| spot.id == id)) {
            self.face(them.at);
        } else if let Some(heading) = Heading::towards(was, self.spot) {
            self.heading = heading;
        }
    }

    /// At rest, it turns towards the nearest noise it hears (fighting, so
    /// far), through walls too: then it may see who made it.
    pub(super) fn listen(&mut self, noises: &[Units]) {
        if self.doing != Doing::Idle {
            return;
        }
        let reach = hearing_reach(self.kind.senses().hearing);
        let heard = noises
            .iter()
            .filter(|noise| self.spot.distance_squared(**noise) <= reach.pow(2))
            .min_by_key(|noise| self.spot.distance_squared(**noise));
        if let Some(&noise) = heard {
            self.face(noise);
        }
    }
}

/// How far a fight is heard, in units.
fn hearing_reach(hearing: Acuity) -> u128 {
    let cells = match hearing {
        Acuity::None => 0,
        Acuity::Poor => 4,
        Acuity::Normal => 8,
        Acuity::Keen => 14,
    };
    cells * CELL_SPAN as u128
}

/// Where fighting can be heard on a level: players in a fight, and mobs
/// winding up a blow.
pub(super) fn noises(fighting: &[Units], mobs: &[Mob]) -> Vec<Units> {
    let striking = mobs
        .iter()
        .filter(|mob| mob.fight.pending.is_some())
        .map(|mob| mob.spot);
    fighting.iter().copied().chain(striking).collect()
}
