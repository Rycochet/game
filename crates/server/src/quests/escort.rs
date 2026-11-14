//! Escorts (owner): someone to see home, found on the deepest level of their
//! step's poi, seen there by everyone. Come near, they follow their quester
//! from level to level and site to site, and onto the world map, where only
//! the quester sees them (`world_look`) and the two go at the slower's pace
//! (`world_gait`). Left too far behind on a site's level, they wait there
//! (lost: a dot, or an arrow at the minimap's edge, points to them) until
//! the quester comes back for them. Brought into the giver's town, they are
//! home: the step is done and they go. Killed, the quest fails
//! (`escortee_killed`).

use shared::protocol::{QuestNews, ServerMessage};
use shared::quest::{Quest, QuestId};
use shared::spot::Spot;

use super::escortee::{escort_step, escortee_id, made, who};
use super::{cache, done_flags};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::{EscortAt, SiteId};

/// How near a quester must come to someone waiting, in cells, for them to
/// follow (Claude).
pub(super) const MEET_GAP: i64 = 2;
/// How far behind on a site's level someone escorted may fall, in cells,
/// before they are lost and wait there (Claude). In the wilderness they are
/// never lost: like the world map, it is open land.
const LOST_GAP: i64 = 12;

/// A level in use and a spot on it.
type Where = ((SiteId, u32), Spot);

/// Every tick, for one player: their escorts met (looked for only when they
/// have `arrived` on another level), following, lost, brought along and
/// seen home. `false` if the client has gone.
pub(super) fn keep_escorts(client: &mut Client, world: &mut ServerWorld, arrived: bool) -> bool {
    let mut sent = true;
    let ids: Vec<QuestId> = client.character.quests.taken.keys().copied().collect();
    for id in ids {
        let state = client
            .character
            .quests
            .taken
            .get(&id)
            .and_then(|p| p.escort);
        // Quests are only made again for an escort under way, or one that
        // may be met where the player is.
        if state.is_none() && !(arrived && at_a_deepest_level(client)) {
            continue;
        }
        let Some(quest) = cache::quest(world, id) else {
            continue;
        };
        sent &= keep_one(client, world, &quest, state);
    }
    sent && tell_gait(client)
}

/// Whether the player is on the deepest level of a site (not a quest's own
/// dungeon), where escortees wait to be met.
fn at_a_deepest_level(client: &Client) -> bool {
    let place = client.visit.as_ref().and_then(|v| v.quest_place());
    place.is_some_and(|place| place.deepest && place.dungeon.is_none())
}

fn keep_one(
    client: &mut Client,
    world: &mut ServerWorld,
    quest: &Quest,
    state: Option<EscortAt>,
) -> bool {
    let id = quest.id;
    let mob = escortee_id(client, id);
    let progress = &client.character.quests.taken[&id];
    let done = done_flags(quest, &progress.counts);
    let open = escort_step(quest).filter(|(i, _)| quest.open_steps(&done).contains(i));
    let (Some((step, chunk)), Some(who)) = (open, who(quest)) else {
        // Done another way (developer tools), or not an escort at all.
        if state.is_some() {
            world.take_escortee(mob);
            set_state(client, id, None);
        }
        return true;
    };
    let here = client.visit.as_ref().map(|v| (v.population(), v.spot()));
    let found = world.escortee_at(mob);
    match state {
        None => {
            let place = client.visit.as_ref().and_then(|v| v.quest_place());
            let theirs = place.is_some_and(|p| p.poi.chunk == chunk && p.deepest);
            if let (true, Some((level, _))) = (theirs, here)
                && found.is_none()
                && let Some(cell) = waiting_cell(client, world, &who)
            {
                let escortee = made(world, client, id, &who, (cell, false));
                if world.put_escortee(level, escortee, Spot::middle_of(cell)) {
                    let spot = world
                        .escortee_at(mob)
                        .map_or(Spot::middle_of(cell), |at| at.1);
                    let (site, level) = level;
                    set_state(client, id, Some(EscortAt::Waiting { site, level, spot }));
                }
            }
            true
        }
        Some(EscortAt::Waiting { site, level, spot }) => {
            match (found, here) {
                (Some((at, them)), Some((place, me)))
                    if at == place && near(them, me, MEET_GAP) =>
                {
                    world.set_following(mob, true);
                    set_state(client, id, Some(EscortAt::With));
                }
                (Some((_, them)), _) if them != spot => {
                    set_state(
                        client,
                        id,
                        Some(EscortAt::Waiting {
                            site,
                            level,
                            spot: them,
                        }),
                    );
                }
                (None, Some((place, _))) if place == (site, level) => {
                    let escortee = made(world, client, id, &who, (spot.cell(), false));
                    world.put_escortee(place, escortee, spot);
                }
                _ => {}
            }
            true
        }
        Some(EscortAt::With) => with(client, world, (quest, step), (found, here)),
    }
}

/// Someone following: brought along wherever the player has gone, lost if
/// left far behind on a site's level, home once in the giver's town.
fn with(
    client: &mut Client,
    world: &mut ServerWorld,
    (quest, step): (&Quest, usize),
    (found, here): (Option<Where>, Option<Where>),
) -> bool {
    let id = quest.id;
    let mob = escortee_id(client, id);
    let wild = client.visit.as_ref().is_some_and(|v| v.is_wilderness());
    match (found, here) {
        // On the world map or in the Hall: with them, unseen by others.
        (_, None) => {
            world.take_escortee(mob);
        }
        (Some((at, them)), Some((place, me))) if at == place => {
            if !wild && !near(them, me, LOST_GAP) {
                world.set_following(mob, false);
                let (site, level) = place;
                set_state(
                    client,
                    id,
                    Some(EscortAt::Waiting {
                        site,
                        level,
                        spot: them,
                    }),
                );
                return true;
            }
        }
        // They have gone to another level, or come off the world map: it
        // was with them, so it comes too, beside them.
        (_, Some((place, me))) => {
            world.take_escortee(mob);
            if let Some(who) = who(quest) {
                let escortee = made(world, client, id, &who, (me.cell(), true));
                world.put_escortee(place, escortee, me);
            }
        }
    }
    let home = client.visit.as_ref().and_then(|v| v.quest_place());
    if home.is_some_and(|p| p.poi.chunk == id.town && p.dungeon.is_none()) {
        world.take_escortee(mob);
        set_state(client, id, None);
        return super::progress::escorted_home(client, world, id, step);
    }
    true
}

/// Where someone waits to be met on the level the player is on: a cell from
/// the quest's own seed, clear of the stairs if there is room.
fn waiting_cell(
    client: &Client,
    world: &ServerWorld,
    who: &super::escortee::Who,
) -> Option<(i64, i64)> {
    let visit = client.visit.as_ref()?;
    let stairs = world.stairs_of(visit.population());
    let spawn = |clear_of: &[(i64, i64)]| {
        shared::mob::chosen_spawns(who.seed, who.kind, 1, visit.cells(), clear_of).pop()
    };
    Some(spawn(&stairs).or_else(|| spawn(&[]))?.position)
}

fn near(a: Spot, b: Spot, cells: i64) -> bool {
    let (a, b) = (a.cell(), b.cell());
    (a.0 - b.0).abs().max((a.1 - b.1).abs()) <= cells
}

fn set_state(client: &mut Client, id: QuestId, state: Option<EscortAt>) {
    if let Some(progress) = client.character.quests.taken.get_mut(&id)
        && progress.escort != state
    {
        progress.escort = state;
        client.unsaved = true;
    }
}

/// Whether someone escorted is with the player now.
pub(crate) fn has_escort_with(client: &Client) -> bool {
    (client.character.quests.taken.values()).any(|p| p.escort == Some(EscortAt::With))
}

/// The gait world map steps go at: the player's, slowed to an escort's
/// with them (owner). Escortees have no skills of their own.
pub(crate) fn world_gait(client: &Client) -> shared::movement::Gait {
    let own = shared::movement::Gait::of(&client.sheet.skills);
    match has_escort_with(client) {
        true => own.slower(shared::movement::Gait::UNSKILLED),
        false => own,
    }
}

/// Tells the client the gait an escort with them sets, when it changes.
fn tell_gait(client: &mut Client) -> bool {
    let gait = has_escort_with(client).then_some(shared::movement::Gait::UNSKILLED);
    if client.quest_seen.escort_gait == Some(gait) {
        return true;
    }
    client.quest_seen.escort_gait = Some(gait);
    client
        .link
        .send(&ServerMessage::Quest(QuestNews::Escort(gait)))
}

/// Someone escorted was killed (owner): their quester's quest fails.
pub(crate) fn escortee_killed(
    clients: &mut [Client],
    world: &mut ServerWorld,
    (quester, id): (shared::id::Id, QuestId),
) {
    let Some(client) = clients.iter_mut().find(|c| c.entity_id() == quester) else {
        return;
    };
    if let Some(progress) = client.character.quests.taken.get_mut(&id) {
        progress.escort = None;
    }
    bevy::log::info!("{}'s escort in quest {id:?} was killed", client.in_log());
    super::give_up::end(client, world, id, true, super::unix_now());
}

/// A quest ended (failed, given up, handed in): whoever was escorted goes.
pub(super) fn dismiss(client: &Client, world: &mut ServerWorld, id: QuestId) {
    world.take_escortee(escortee_id(client, id));
}
