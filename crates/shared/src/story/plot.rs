//! Telling a story: choosing its reason and beats from the text seed, in up
//! to three paragraphs (the catalogue's "rules for putting beats together").

use super::draft::Draft;
use super::facts::{Goal, Reason, State};
use super::request::{Length, Request};
use super::told::{Ending, Lie, Story, Telling, Truth};
use super::{Beat, Slot, roll};
use crate::rng::Rng;

/// The reason a quest's first goal and place suit best, weighted
/// (`Reason::goal_weight`, `place_weight`). A chain keeps its reason.
pub fn choose_reason(request: &Request) -> Reason {
    if let Some(reason) = request.reason {
        return reason;
    }
    let goal = request.main_goal();
    let place = request.place.as_ref().map(|p| p.description.kind);
    let weights: Vec<(Reason, u64)> = (Reason::ALL.iter())
        .map(|&reason| {
            let by_place = place.map_or(1, |p| reason.place_weight(p));
            (reason, reason.goal_weight(goal) * by_place)
        })
        .filter(|(_, weight)| *weight > 0)
        .collect();
    request
        .text_seed
        .derive("reason")
        .rng()
        .pick_weighted(&weights)
}

impl Draft<'_> {
    /// Whether a beat may be drawn now: its reason, darkness, twist and
    /// needs all allow it, and it isn't told already.
    fn allowed(&self, beat: Beat) -> bool {
        let reasons = beat.reasons();
        (reasons.is_empty() || reasons.contains(&self.reason))
            && (!beat.is_dark() || self.request.dark)
            && (!beat.is_twist() || self.request.later_step)
            && !self.has(beat)
            && self.fits(beat)
    }

    /// Up to `most` beats of `slot`, drawn one at a time (each changes what
    /// fits next). `first` is told first when it can be.
    fn draw(&mut self, rng: &mut Rng, slot: Slot, most: u64, first: &[Beat]) {
        let count = if most == 0 {
            0
        } else {
            rng.range_u64(1, most + 1)
        };
        for n in 0..count {
            let preferred = (first.iter().copied()).find(|&b| n == 0 && self.allowed(b));
            let candidates: Vec<Beat> = (Beat::ALL.iter().copied())
                .filter(|&b| b.slot() == slot && self.allowed(b))
                .collect();
            match preferred
                .or_else(|| (!candidates.is_empty()).then(|| roll::pick(rng, &candidates)))
            {
                Some(beat) => self.apply(beat),
                None => break,
            }
        }
    }

    /// The ask for one goal, dark when the quest is and one fits.
    fn ask(&mut self, goal: Goal) -> Beat {
        use Beat::*;
        let dark = self.request.dark;
        match goal {
            Goal::Find if dark && self.dark_place() => StealFrom,
            Goal::Find
                if self.relative_used()
                    && self.member(super::Role::Relative).state == State::Dead
                    && self.request.item.is_some_and(|i| i.is_keepsake()) =>
            {
                LayToRest
            }
            Goal::Find => Retrieve,
            Goal::Slay => Clear,
            Goal::Deliver if dark => Frame,
            Goal::Deliver => Take,
            Goal::Gather => Collect,
            Goal::Escort if self.request.pet.is_some() || self.reason == Reason::LostPet => {
                BringPet
            }
            Goal::Escort => Bring,
            Goal::Explore | Goal::Visit if dark => Silence,
            Goal::Explore | Goal::Visit => {
                let relative_lost = self.relative_used()
                    && self.member(super::Role::Relative).state == State::Missing;
                if relative_lost { LearnFate } else { Look }
            }
        }
    }

    fn dark_place(&self) -> bool {
        use super::PlaceKind::*;
        matches!(self.place_kind(), Some(Dwelling | Store | Farm))
    }
}

/// Tell the story a request asks for.
pub fn tell(request: &Request) -> Telling {
    use Beat::*;
    let reason = choose_reason(request);
    let mut draft = Draft::new(request, reason);
    let mut rng = request.text_seed.derive("beats").rng();
    let full = request.length == Length::Full;
    let dark_first: &[Beat] = if request.dark { &[Murdered] } else { &[] };
    draft.draw(&mut rng, Slot::Hook, if full { 2 } else { 1 }, dark_first);
    let hooks = draft.beats.len();
    if full {
        let twists = [
            NotGuilty,
            LeftOnPurpose,
            BossWasOneOfUs,
            WhoToHelp,
            KeepOrReturn,
            SpareOrSlay,
        ];
        let first: &[Beat] = if request.later_step { &twists } else { &[] };
        let most = rng.range_u64(0, 3);
        draft.draw(&mut rng, Slot::Complication, most, first);
    }
    let middle = draft.beats.len();
    for ask in &request.asks {
        let beat = draft.ask(ask.goal);
        if !draft.has(beat) {
            draft.apply(beat);
        }
    }
    // A dark quest's story always has a dark beat; a stake will do.
    if request.dark && !draft.beats.iter().any(|b| b.is_dark()) {
        draft.apply(Revenge);
    }
    if full {
        let most = rng.range_u64(0, 2);
        draft.draw(&mut rng, Slot::Stake, most, &[]);
    }
    if draft.fits(Reward) && !draft.has(Reward) {
        draft.apply(Reward);
    }
    lie(&mut draft, &mut rng);
    let ending = ending(&draft);
    let beats = draft.beats.clone();
    let paragraphs: Vec<Vec<Beat>> = if full {
        [&beats[..hooks], &beats[hooks..middle], &beats[middle..]]
            .into_iter()
            .filter(|p| !p.is_empty())
            .map(<[Beat]>::to_vec)
            .collect()
    } else {
        vec![beats]
    };
    let (cast, facts, lies) = draft.finish();
    Telling {
        story: Story {
            text_seed: request.text_seed,
            reason,
            cast,
            facts,
            paragraphs,
        },
        ending: Ending {
            beats: ending,
            lies,
        },
    }
}

/// Now and then the giver (or the witness, for a rumour) tells a wrong
/// errand; the truth is kept for the ending (owner: stories may lie).
fn lie(draft: &mut Draft, rng: &mut Rng) {
    let (Some(truth), Some(place)) = (draft.errand, draft.place_kind()) else {
        return;
    };
    // A carried errand was told before: a new lie would contradict it.
    if !draft.used.errand || draft.request.carried.errand.is_some() || rng.range_u64(0, 5) != 0 {
        return;
    }
    let others: Vec<_> = (super::facts::Errand::ALL.iter().copied())
        .filter(|e| *e != truth && e.allowed_at(place))
        .collect();
    if others.is_empty() {
        return;
    }
    draft.errand = Some(roll::pick(rng, &others));
    let liar = if draft.has(Beat::RumourHeard) {
        super::Role::Witness
    } else {
        super::Role::Giver
    };
    draft.lies.push(Lie {
        liar,
        truth: Truth::Errand(truth),
    });
}

/// What the giver says at turn-in.
fn ending(draft: &Draft) -> Vec<Beat> {
    use Beat::*;
    let mut beats = vec![Thanks];
    if draft.has(LearnFate) || draft.has(Look) {
        beats.push(Found);
    }
    if !draft.lies.is_empty() {
        beats.push(TruthFound);
    }
    let staked =
        (draft.beats.iter()).any(|b| b.slot() == Slot::Stake && !matches!(b, Reward | Warning));
    if staked {
        beats.push(StakeMet);
    }
    if draft.request.continues {
        beats.push(Lead);
    }
    beats
}
