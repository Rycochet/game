//! Quests' own dungeons (`shared::quest::QuestDungeon`, owner): one copy for
//! each character (`SiteId::Quest`), made on entering from beside the giver's
//! door ("Enter Cellar", or straight in on taking the quest), left by its way
//! up to where they went in, and made afresh about an hour after they came
//! out, the progress made inside it lost. Logging out inside keeps it, with
//! no time running (`DungeonStay::left` is `None` while inside).

use bevy::prelude::*;
use shared::poi::Poi;
use shared::poi::can_enter_from;
use shared::quest::{DungeonWay, QuestDungeon, QuestId};

use super::cache;
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::SiteId;

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

/// Goes into quest `id`'s dungeon, from beside its giver's door or, for a
/// hidden one, near its entrance on the world map (`reach`; the debug API
/// leaves that out), if the quest is taken and has one. Where they
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
    let beside = !reach
        || match dungeon.way {
            DungeonWay::Cellar => {
                crate::sites::beside_giver(client, world, id.town, quest.building)
            }
            // On the world map, near enough its entrance.
            DungeonWay::Hidden => {
                let at = client.world_position();
                client.visit.is_none() && can_enter_from(&dungeon.poi, at, world.width)
            }
        };
    let back = client.visit.as_ref().map(crate::sites::Visit::save);
    let in_one = back.is_some_and(|back| matches!(back.site, SiteId::Quest { .. }));
    if !taken || !beside || in_one || client.character.hall.is_some() {
        warn!(
            "{} asked to enter {id:?}'s dungeon from elsewhere",
            client.in_log()
        );
        return true;
    }
    super::dungeon_reset::reset_if_due(client, world, id, super::clock::unix_now());
    let stay = client.character.quests.dungeons.entry(id).or_default();
    stay.back = back;
    stay.left = None;
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

/// Quest `id` is handed in, failed or given up: out of its dungeon if inside
/// (back where they went in), their copy of it gone, and a hidden entrance
/// off their map. `false` if the client has gone.
pub(crate) fn quest_ended(client: &mut Client, world: &mut ServerWorld, id: QuestId) -> bool {
    let site = site_of(client, id);
    let inside = site.is_some() && client.visit.as_ref().map(crate::sites::Visit::id) == site;
    let out = !inside || crate::sites::leave_dungeon(client, world);
    super::dungeon_reset::forget_copy(client, world, id);
    let stay = client.character.quests.dungeons.remove(&id);
    client.unsaved = true;
    let entrance = stay.and_then(|stay| stay.entrance);
    out && entrance.is_none_or(|poi| crate::world_map::send_chunk(client, world, poi.chunk))
}
