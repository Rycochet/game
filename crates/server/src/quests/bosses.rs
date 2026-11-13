//! Lone quest bosses (owner): a poi's boss is part of its site from the start
//! when a town's quest asks for it (`shared::quest::owner` names the town),
//! seen by everyone. What it is to a character: their own quest's step, a
//! stranger's quest (a red "!"), or dealt with.

use std::collections::HashMap;

use shared::poi::{ChunkPos, Poi};
use shared::protocol::BossSign;
use shared::quest::{Hook, QuestId, QuestKind, QuestWorld, Target, owner};

use super::{cache, done_flags};
use crate::ServerWorld;
use crate::storage::QuestBook;

/// The quest a poi's boss belongs to, and which of its steps slays it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BossQuest {
    pub(crate) id: QuestId,
    pub(crate) kind: QuestKind,
    pub(crate) step: usize,
}

/// Each poi's boss quest, or none, once worked out (`TownQuests::bosses`).
pub(crate) type Bosses = HashMap<ChunkPos, Option<BossQuest>>;

/// The quest whose step slays the boss of the poi in `chunk`, if any town's
/// quest asks for it.
pub(crate) fn boss_quest(world: &mut ServerWorld, chunk: ChunkPos) -> Option<BossQuest> {
    if let Some(found) = world.quests.bosses.get(&chunk) {
        return *found;
    }
    let found = find(world, chunk);
    if world.quests.bosses.len() >= cache::MOST_TOWNS {
        world.quests.bosses.clear();
    }
    world.quests.bosses.insert(chunk, found);
    found
}

fn find(world: &mut ServerWorld, chunk: ChunkPos) -> Option<BossQuest> {
    let poi: Poi = world.pois.in_chunk(chunk)?;
    if !Hook::of_poi(&poi).contains(&Hook::Boss) {
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
        owner(&quest_world, &poi, Hook::Boss)?
    };
    let wanted = Target::Poi {
        chunk,
        hook: Hook::Boss,
    };
    let town = cache::town(world, town)?;
    town.quests.iter().find_map(|quest| {
        let step = quest.steps.iter().position(|step| step.target == wanted)?;
        Some(BossQuest {
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
    /// They don't have the quest and haven't dealt with it: a red "!".
    Stranger,
    /// Dealt with: killed and remembered, its step done, or handed in.
    Dealt,
}

/// What the boss of `boss` is to the character whose book this is.
pub(crate) fn regard(world: &mut ServerWorld, book: &QuestBook, boss: BossQuest) -> Regard {
    if book.done.contains_key(&boss.id) || book.slain.contains(&boss.id) {
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
