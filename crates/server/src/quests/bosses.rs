//! A poi's boss and its unique item (owner): each part of its site from the
//! start when a town's quest asks for it (`shared::quest::owner` names the
//! town); the boss seen by everyone, the item by each character still wanting
//! it, their own copy (`finds`). What either is to a character: their own
//! quest's step, a stranger's quest (a red "!"), or dealt with.

use std::collections::HashMap;

use shared::poi::{ChunkPos, Poi};
use shared::protocol::BossSign;
use shared::quest::{Hook, QuestId, QuestKind, QuestWorld, Target, owner};

use super::{cache, done_flags};
use crate::ServerWorld;
use crate::storage::QuestBook;

/// The quest a poi's boss (or item) belongs to, and which of its steps slays
/// (or finds) it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HookQuest {
    pub(crate) id: QuestId,
    pub(crate) kind: QuestKind,
    pub(crate) step: usize,
}

pub(crate) type BossQuest = HookQuest;

/// Each poi's boss and item quests, or none, once worked out
/// (`TownQuests::bosses`).
pub(crate) type Bosses = HashMap<(ChunkPos, Hook), Option<HookQuest>>;

/// The quest whose step slays the boss of the poi in `chunk`, if any town's
/// quest asks for it.
pub(crate) fn boss_quest(world: &mut ServerWorld, chunk: ChunkPos) -> Option<BossQuest> {
    hook_quest(world, chunk, Hook::Boss)
}

/// The quest using an exclusive hook (a boss, an item) of the poi in
/// `chunk`, if any town's quest asks for it.
pub(crate) fn hook_quest(
    world: &mut ServerWorld,
    chunk: ChunkPos,
    hook: Hook,
) -> Option<HookQuest> {
    if let Some(found) = world.quests.bosses.get(&(chunk, hook)) {
        return *found;
    }
    let found = find(world, chunk, hook);
    if world.quests.bosses.len() >= cache::MOST_TOWNS {
        world.quests.bosses.clear();
    }
    world.quests.bosses.insert((chunk, hook), found);
    found
}

fn find(world: &mut ServerWorld, chunk: ChunkPos, hook: Hook) -> Option<HookQuest> {
    let poi: Poi = world.pois.in_chunk(chunk)?;
    if !Hook::of_poi(&poi).contains(&hook) {
        return None;
    }
    let town = {
        let ServerWorld {
            pois,
            ground,
            quests,
            ..
        } = &*world;
        let name_of = |poi: &Poi| {
            shared::place_name::place_name(poi.kind, poi.seed, ground.metres(poi.x, poi.y))
        };
        let quest_world = QuestWorld {
            pois,
            roads: quests.roads(),
            name_of: &name_of,
        };
        owner(&quest_world, &poi, hook)?
    };
    let wanted = Target::Poi { chunk, hook };
    let town = cache::town(world, town)?;
    town.quests.iter().find_map(|quest| {
        let step = quest.steps.iter().position(|step| step.target == wanted)?;
        Some(HookQuest {
            id: quest.id,
            kind: quest.kind,
            step,
        })
    })
}

/// What a boss is to a character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Regard {
    /// Their quest is taken and its step not done: the quest's colour.
    OnStep,
    /// They don't have the quest and haven't dealt with it (killed it, or
    /// found it): a red "!".
    Stranger,
    /// Dealt with: killed and remembered, its step done, or handed in.
    Dealt,
}

/// What the boss of `boss` is to the character whose book this is.
pub(crate) fn regard(world: &mut ServerWorld, book: &QuestBook, boss: HookQuest) -> Regard {
    let ahead = book.slain.contains(&boss.id) || book.found.contains(&boss.id);
    if book.done.contains_key(&boss.id) || ahead {
        return Regard::Dealt;
    }
    let Some(progress) = book.taken.get(&boss.id) else {
        return Regard::Stranger;
    };
    let Some(quest) = cache::quest(world, boss.id) else {
        return Regard::Dealt;
    };
    let done = done_flags(&quest, &progress.counts);
    match done.get(boss.step).copied().unwrap_or(true) {
        true => Regard::Dealt,
        false => Regard::OnStep,
    }
}

impl Regard {
    /// The sign over the boss for them, if any.
    pub(crate) fn sign(self, kind: QuestKind) -> Option<BossSign> {
        match self {
            Self::OnStep => Some(BossSign::Quest(kind)),
            Self::Stranger => Some(BossSign::Unknown),
            Self::Dealt => None,
        }
    }

    /// Whether it should come back for them, if it isn't alive (Claude, owner:
    /// for anyone on the step or who hasn't killed it yet).
    pub(crate) fn brings_back(self) -> bool {
        self != Self::Dealt
    }
}
