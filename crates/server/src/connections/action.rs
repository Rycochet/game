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
    Ground(GroundAction),
    UseAbility {
        ability: shared::ability::Ability,
        /// `None`: on themselves.
        target: Option<Id>,
    },
    Door(shared::protocol::DoorAsk),
}
