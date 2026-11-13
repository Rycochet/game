//! Quests' own dungeons (`shared::quest::QuestDungeon`, owner): one copy for
//! each character (`SiteId::Quest`), made on entering from beside the giver's
//! door ("Enter Cellar", or straight in on taking the quest), left by its way
//! up to where they went in, and made afresh about an hour after they came
//! out, the progress made inside it lost. Logging out inside keeps it, with
//! no time running (`DungeonStay::left` is `None` while inside).

use bevy::prelude::*;
use shared::poi::Poi;
use shared::quest::{QuestDungeon, QuestId, Target};

use super::{cache, done_flags, log};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::{DungeonStay, SiteId};

/// How long after coming out a quest's dungeon is made afresh (owner: about
/// an hour).
pub(crate) const RESET_AFTER: i64 = 60 * 60;

/// What quest `id`'s dungeon is made from, if it has one.
pub(crate) fn dungeon_of(world: &mut ServerWorld, id: QuestId) -> Option<QuestDungeon> {
    cache::quest(world, id)?.dungeon
}

/// The made-up poi a quest dungeon site is generated as.
pub(crate) fn dungeon_poi(world: &mut ServerWorld, site: SiteId) -> Option<Poi> {
    let SiteId::Quest { quest, .. } = site else {
        return None;
    };
    dungeon_of(world, quest).map(|dungeon| dungeon.poi)
}

/// This character's copy of quest `id`'s dungeon (none without a character,
/// which tests and the debug API's made-up clients may lack).
pub(crate) fn site_of(client: &Client, id: QuestId) -> Option<SiteId> {
    let character = client.character_id?;
    Some(SiteId::Quest {
        character,
        quest: id,
    })
}

/// Goes into quest `id`'s dungeon, from beside its giver's door (`reach`; the
/// debug API leaves that out), if the quest is taken and has one. Where they
/// stood is kept for coming out. `false` if the client has gone.
pub(crate) fn enter(
    client: &mut Client,
    world: &mut ServerWorld,
    id: QuestId,
    reach: bool,
) -> bool {
    let Some(quest) = cache::quest(world, id) else {
        return true;
    };
    let (Some(dungeon), Some(site)) = (quest.dungeon, site_of(client, id)) else {
        warn!("{} asked to enter a dungeon {id:?} hasn't", client.in_log());
        return true;
    };
    if client.sheet.vitals.fighting(world.action_clock.ticks()) {
        info!("Refused going into a quest's dungeon while fighting");
        return true;
    }
    let taken = client.character.quests.taken.contains_key(&id);
    let beside = !reach || crate::sites::beside_giver(client, world, id.town, quest.building);
    let back = client.visit.as_ref().map(crate::sites::Visit::save);
    let in_one = back.is_some_and(|back| matches!(back.site, SiteId::Quest { .. }));
    if !taken || !beside || in_one || client.character.hall.is_some() {
        warn!(
            "{} asked to enter {id:?}'s dungeon from elsewhere",
            client.in_log()
        );
        return true;
    }
    reset_if_due(client, world, id, super::clock::unix_now());
    client
        .character
        .quests
        .dungeons
        .insert(id, DungeonStay { back, left: None });
    client.unsaved = true;
    info!("{} went into {id:?}'s dungeon", client.in_log());
    crate::sites::enter_dungeon(client, world, dungeon.poi, site)
}

/// The player has come out of a quest's dungeon (by its way up, or killed):
/// its hour starts. Where to put them back, if anywhere.
pub(crate) fn came_out(
    client: &mut Client,
    site: SiteId,
    now: i64,
) -> Option<crate::storage::VisitSave> {
    let SiteId::Quest { quest, .. } = site else {
        return None;
    };
    let stay = client.character.quests.dungeons.entry(quest).or_default();
    stay.left = Some(now);
    client.unsaved = true;
    stay.back
}

/// Every quest dungeon of the character's left more than `RESET_AFTER` ago
/// is made afresh: forgotten, with what was done inside. Steps already
/// finished stay done (Claude). `false` if the client has gone.
pub(crate) fn reset_due(client: &mut Client, world: &mut ServerWorld, now: i64) -> bool {
    let due: Vec<QuestId> = (client.character.quests.dungeons.iter())
        .filter(|(_, stay)| stay.left.is_some_and(|left| now - left >= RESET_AFTER))
        .map(|(&id, _)| id)
        .collect();
    let mut sent = true;
    for id in due {
        if reset(client, world, id) {
            sent &= log::tell_changed(client, world, id);
        }
    }
    sent
}

fn reset_if_due(client: &mut Client, world: &mut ServerWorld, id: QuestId, now: i64) {
    let stay = client.character.quests.dungeons.get(&id);
    if stay
        .and_then(|stay| stay.left)
        .is_some_and(|left| now - left >= RESET_AFTER)
    {
        reset(client, world, id);
    }
}

/// Quest `id`'s dungeon made afresh for this character; whether its progress
/// changed.
fn reset(client: &mut Client, world: &mut ServerWorld, id: QuestId) -> bool {
    info!("{}'s copy of {id:?}'s dungeon resets", client.in_log());
    forget(client, world, id);
    let Some(quest) = cache::quest(world, id) else {
        return false;
    };
    let Some(progress) = client.character.quests.taken.get_mut(&id) else {
        return false;
    };
    progress.counts.resize(quest.steps.len(), 0);
    let mut changed = false;
    for (step, count) in quest.steps.iter().zip(progress.counts.iter_mut()) {
        let finished = *count >= step.count.unwrap_or(1);
        if matches!(step.target, Target::Dungeon { .. }) && !finished && *count > 0 {
            *count = 0;
            changed = true;
        }
    }
    progress.ready = quest.ready_to_turn_in(&done_flags(&quest, &progress.counts));
    changed
}

/// Quest `id` is handed in, failed or given up: out of its dungeon if inside
/// (back where they went in), and their copy of it gone. `false` if the
/// client has gone.
pub(crate) fn quest_ended(client: &mut Client, world: &mut ServerWorld, id: QuestId) -> bool {
    let site = site_of(client, id);
    let inside = site.is_some() && client.visit.as_ref().map(crate::sites::Visit::id) == site;
    let out = !inside || crate::sites::leave_dungeon(client, world);
    forget(client, world, id);
    out
}

/// The character's copy of quest `id`'s dungeon is gone: on a reset, and when
/// the quest is handed in, failed or given up.
pub(crate) fn forget(client: &mut Client, world: &mut ServerWorld, id: QuestId) {
    client.character.quests.dungeons.remove(&id);
    client.unsaved = true;
    if let Some(site) = site_of(client, id) {
        world.forget_site(site);
        client.character.forget_site(&site);
    }
}
