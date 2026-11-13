//! A quest's plan before its story is told: its steps and the checkable
//! facts the story needs (place, creatures, boss, item, pet, destination).

use super::{Hook, Step};
use crate::creature::CreatureKind;
use crate::poi::{ChunkPos, Poi};
use crate::story::{Ask, Boss, ItemCategory, Pet, Remains};

#[derive(Clone, Debug, Default)]
pub(super) struct Plan {
    pub steps: Vec<Step>,
    /// The poi the story speaks of, or the town itself for the wilds round it.
    pub place: Option<Poi>,
    /// The kind slain and how many.
    pub creatures: Option<(CreatureKind, u32)>,
    pub boss: Option<Boss>,
    pub item: Option<ItemCategory>,
    /// What a "Collect" step brings back.
    pub remains: Option<Remains>,
    /// The pet led home, for an escort of one.
    pub pet: Option<Pet>,
    /// The town something is taken to.
    pub destination: Option<Poi>,
    /// The exclusive hooks it uses.
    pub claims: Vec<(ChunkPos, Hook)>,
}

impl Plan {
    /// What the story is asked to explain, step by step.
    pub fn asks(&self) -> Vec<Ask> {
        (self.steps.iter())
            .map(|step| Ask {
                goal: step.goal,
                count: step.count,
            })
            .collect()
    }
}
