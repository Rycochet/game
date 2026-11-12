//! What a mob notices: players near enough, and where they are.

use shared::creature::{CreatureKind, Size};
use shared::dungeon::{Level, line_is_clear};
use shared::id::Id;
use shared::mob::scent_strength;
use shared::pathfinding::DistanceMap;
use shared::protocol::CharacterId;
use shared::sheet::{Appearance, Feeling, Reaction};
use shared::wear::WornLook;
use std::collections::BTreeMap;

use super::territory::{is_territorial, territorial_doing};
use super::timid::too_near;
use super::{ATTACK_GAP, Doing, FRIENDLY_GAP, Mob};
use crate::connections::{Client, Phase};
use crate::storage::SiteId;

/// A player as others see them.
pub(crate) struct Someone {
    pub(super) id: Id,
    pub(super) character: Option<CharacterId>,
    /// Their character's name.
    pub(super) name: String,
    pub(super) look: (CreatureKind, Appearance),
    /// What can be seen of what they wear.
    pub(super) worn: Vec<WornLook>,
    /// The mark before their name (`Client::mark`).
    pub(super) mark: Option<shared::role::RoleBadge>,
    /// The site level they are on, if any.
    pub(super) level: Option<(SiteId, u32)>,
    /// They are in the Hall of Worlds (and `level` is `None`).
    pub(super) in_hall: bool,
    /// The cell (or world map point) they are in: what sight and mobs go by.
    pub(super) position: (i64, i64),
    /// Exactly where, in units, on a level or in the Hall (the Hall's own).
    pub(super) spot: shared::spot::Spot,
    /// How fast they act (`Client::speed`), for how quickly they are shown
    /// moving.
    pub(super) speed: u32,
    /// Mobs take no notice of them.
    pub(super) god_mode: bool,
    /// How strong a trail they leave, as a percentage.
    pub(super) scent: u32,
    /// The way they face.
    pub(super) heading: shared::facing::Heading,
    /// Winding up a blow: a noise mobs near enough hear.
    pub(super) fighting: bool,
}

pub(super) fn everyone_playing(clients: &mut [Client]) -> Vec<Someone> {
    clients
        .iter_mut()
        .filter(|client| client.phase == Phase::Playing && !client.is_gone())
        .map(|client| {
            let (level, spot) = match (&client.visit, client.character.hall) {
                (Some(visit), _) => (Some(visit.population()), visit.spot()),
                (None, Some(at)) => (None, at),
                (None, None) => (
                    None,
                    shared::spot::Spot::middle_of(client.character.world_position),
                ),
            };
            let position = match (&client.visit, client.character.hall) {
                (None, None) => client.character.world_position,
                _ => spot.cell(),
            };
            Someone {
                id: client.entity_id(),
                character: client.character_id,
                name: client.name.clone(),
                look: (client.inventory.creature, client.sheet.appearance.clone()),
                worn: client.inventory.shown(),
                mark: client.mark(),
                level,
                in_hall: client.visit.is_none() && client.character.hall.is_some(),
                position,
                spot,
                speed: client.speed,
                god_mode: client.god_mode,
                scent: scent_strength(&client.inventory),
                heading: client.heading,
                fighting: client.winding_up(),
            }
        })
        .collect()
}

/// How far apart, counting a diagonal as one.
pub(super) fn cells_apart(a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// The distance maps worked out this tick on one level: mobs after the same place
/// share one, one for each size after it.
#[derive(Default)]
pub(super) struct Maps(BTreeMap<((i64, i64), Size), DistanceMap>);

impl Maps {
    pub(super) fn to(&mut self, level: &Level, size: Size, goal: (i64, i64)) -> &DistanceMap {
        self.0
            .entry((goal, size))
            .or_insert_with(|| DistanceMap::sized(level, size, goal))
    }
}

/// A player on a level, as far as its mobs go.
pub(super) struct Spot {
    pub(super) id: Id,
    pub(super) character: Option<CharacterId>,
    pub(super) kind: CreatureKind,
    pub(super) position: (i64, i64),
    /// Exactly where, in units (`Someone::spot`).
    pub(super) at: shared::spot::Spot,
    /// How strong a trail they leave, as a percentage.
    pub(super) scent: u32,
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
    // Only what is in its cone: sneaking up behind works.
    let cone = mob.cone();
    spots
        .iter()
        .filter(|spot| {
            cells_apart(spot.position, mob.position) <= sees_within
                && cone.sees(spot.at)
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
                Reaction::Flee => too_near(mob, spot).then_some(Doing::Fleeing {
                    from: spot.id,
                    struck: false,
                }),
                Reaction::Approach => Some(going(FRIENDLY_GAP)),
                Reaction::Attack => Some(going(ATTACK_GAP)),
            }
        })
}
