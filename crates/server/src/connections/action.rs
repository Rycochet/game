//! What a client asks to do that takes time, queued behind its steps.

use shared::id::Id;
use shared::movement::Step;
use shared::poi::ChunkPos;

use crate::ground::GroundAction;

/// Something that takes time, waiting its turn. Things that depend on where the
/// player is have to wait behind the steps that lead there.
#[derive(Clone, Debug)]
pub(super) enum Action {
    Step {
        number: u32,
        step: Step,
    },
    /// A move on a level or in the Hall, to here (units, as the client
    /// counts them).
    Move {
        number: u32,
        to: shared::spot::Spot,
    },
    EnterSite {
        chunk: ChunkPos,
    },
    EnterGate {
        entrance: u8,
    },
    UseExit,
    Down,
    Up,
    OpenTrade {
        with: Id,
    },
    Quest(shared::protocol::QuestAsk),
    Ground(GroundAction),
    UseAbility {
        ability: shared::ability::Ability,
        /// `None`: on themselves.
        target: Option<Id>,
    },
    Door(shared::protocol::DoorAsk),
    /// Eating one of this (`consume.rs`).
    Consume {
        item: Id,
    },
}

impl super::Client {
    /// Queues a step or move, unless too many wait already (its `Position`
    /// then puts the client right). Always stays.
    pub(super) fn queue_move(&mut self, action: Action) -> bool {
        let number = match action {
            Action::Step { number, .. } | Action::Move { number, .. } => number,
            _ => return true,
        };
        if self.waiting.len() < super::MOST_WAITING {
            self.waiting.push_back(action);
        } else {
            self.last_step = Some(number);
        }
        true
    }
}
