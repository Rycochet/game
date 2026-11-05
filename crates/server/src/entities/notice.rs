//! What a mob notices: players near enough, and where they are.

use shared::creature::CreatureKind;
use shared::dungeon::{Level, line_is_clear};
use shared::id::Id;
use shared::pathfinding::DistanceMap;
use shared::protocol::CharacterId;
use shared::sheet::{Appearance, Feeling, Reaction};
use shared::wear::WornLook;
use std::collections::BTreeMap;

use super::territory::{is_territorial, territorial_doing};
use super::{ATTACK_GAP, Doing, FRIENDLY_GAP, Mob};
use crate::connections::{Client, Phase};
use crate::storage::SiteId;

/// A player as others see them.
pub(crate) struct Someone {
    pub(super) id: Id,
    pub(super) character: Option<CharacterId>,
    pub(super) look: (CreatureKind, Appearance),
    /// What can be seen of what they wear.
    pub(super) worn: Vec<WornLook>,
    /// The site level they are on, if any.
    pub(super) level: Option<(SiteId, u32)>,
    /// They are in the Hall of Worlds (and `level` is `None`).
    pub(super) in_hall: bool,
    pub(super) position: (i64, i64),
    /// Mobs take no notice of them.
    pub(super) god_mode: bool,
}

pub(super) fn everyone_playing(clients: &mut [Client]) -> Vec<Someone> {
    clients
        .iter_mut()
        .filter(|client| client.phase == Phase::Playing && !client.is_gone())
        .map(|client| {
            let (level, position) = match (&client.visit, client.character.hall) {
                (Some(visit), _) => (Some((visit.id(), visit.index())), visit.position()),
                (None, Some(at)) => (None, at),
                (None, None) => (None, client.character.world_position),
            };
            Someone {
                id: client.entity_id(),
                character: client.character_id,
                look: (client.inventory.creature, client.sheet.appearance.clone()),
                worn: client.inventory.shown(),
                level,
                in_hall: client.visit.is_none() && client.character.hall.is_some(),
                position,
                god_mode: client.god_mode,
            }
        })
        .collect()
}

/// How far apart, counting a diagonal as one.
pub(super) fn cells_apart(a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// The distance maps worked out this tick on one level: mobs after the same place
/// share one.
#[derive(Default)]
pub(super) struct Maps(BTreeMap<(i64, i64), DistanceMap>);

impl Maps {
    pub(super) fn to(&mut self, level: &Level, goal: (i64, i64)) -> &DistanceMap {
        self.0
            .entry(goal)
            .or_insert_with(|| DistanceMap::to(level, goal))
    }
}

/// A player on a level, as far as its mobs go.
pub(super) struct Spot {
    pub(super) id: Id,
    pub(super) character: Option<CharacterId>,
    pub(super) kind: CreatureKind,
    pub(super) position: (i64, i64),
}

/// What a mob does about players now, if it should stop what it is doing.
pub(super) fn notice(mob: &Mob, level: &Level, spots: &[Spot]) -> Option<Doing> {
    if matches!(
        mob.doing,
        Doing::Going { .. } | Doing::Following { .. } | Doing::Fleeing { .. }
    ) {
        return None;
    }
    let sees_within = mob.kind.senses().sight_range();
    spots
        .iter()
        .filter(|spot| {
            cells_apart(spot.position, mob.position) <= sees_within
                && line_is_clear(level, mob.position, spot.position)
        })
        .find_map(|spot| {
            let going = |gap| Doing::Going {
                after: spot.id,
                last_seen: spot.position,
                gap,
            };
            let feeling = mob.feeling(spot.kind, spot.character.as_ref());
            if is_territorial(mob) && feeling != Feeling::Friendly {
                return territorial_doing(mob, spot, feeling);
            }
            match mob.kind.reaction(feeling) {
                Reaction::Ignore => None,
                Reaction::Flee => Some(Doing::Fleeing { from: spot.id }),
                Reaction::Approach => Some(going(FRIENDLY_GAP)),
                Reaction::Attack => Some(going(ATTACK_GAP)),
            }
        })
}
