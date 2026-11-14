//! A poi's unique quest item (owner): it lies on its site's deepest level
//! from the start, for each character who doesn't have one and hasn't found
//! it yet, as their own copy, so one player's picking it up takes nothing
//! from another. It is only ever in what the server tells that player of the
//! ground (`ground`), with a red "!" over it for a stranger to its quest and
//! the quest's colour for one on its step. Picked up, it does that step, or
//! is remembered until the quest is taken (`ahead`), marking the giver.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::item::{Item, ItemKind, Quality};
use shared::item_form::{ItemOrder, make_item};
use shared::mob::chosen_spawns;
use shared::poi::Poi;
use shared::protocol::{FindMark, QuestNews, ServerMessage};
use shared::quest::{Hook, item_of};
use shared::spot::Spot;
use shared::story::ItemCategory;

use super::bosses::{HookQuest, Regard, hook_quest, regard};
use super::{cache, done_flags, log, tell_markers};
use crate::ServerWorld;
use crate::connections::Client;
use crate::world_map::learn_place;

/// A quest item lying for one character: where, what, and whose quest.
#[derive(Clone, Debug)]
pub(crate) struct Find {
    pub(crate) at: Spot,
    pub(crate) item: Item,
    pub(crate) quest: HookQuest,
}

/// The quest item lying on the level the player is on for them, if one is.
pub(crate) fn find_here(client: &Client, world: &mut ServerWorld) -> Option<Find> {
    let visit = client
        .visit
        .as_ref()
        .filter(|visit| !visit.is_wilderness())?;
    let poi = *visit.poi()?;
    let levels = visit.site()?.levels();
    if visit.index() + 1 != levels {
        return None;
    }
    let quest = hook_quest(world, poi.chunk, Hook::Item)?;
    let carried = client.inventory.quest_items(quest.id) > 0;
    if carried || regard(world, &client.character.quests, quest) == Regard::Dealt {
        return None;
    }
    let stairs = world.stairs_of(visit.population());
    let seed = poi.seed.derive("quest item").derive("spot");
    // Clear of the stairs if there is room, else anywhere on the level.
    let spot = |clear_of: &[(i64, i64)]| {
        chosen_spawns(seed, CreatureKind::Human, 1, visit.cells(), clear_of).pop()
    };
    let at = spot(&stairs).or_else(|| spot(&[]))?.position;
    let item = made(world, &poi, quest, client.entity_id());
    Some(Find {
        at: Spot::middle_of(at),
        item,
        quest,
    })
}

/// The item itself, the same each time for one character: rings and
/// necklaces as their kinds, a sword as a dagger, the rest a keepsake named
/// for what it is (owner).
fn made(world: &ServerWorld, poi: &Poi, quest: HookQuest, who: Id) -> Item {
    let id = Id(uuid::Uuid::new_v5(
        &who.0,
        format!("{:?}", quest.id).as_bytes(),
    ));
    let category = item_of(poi);
    let kind = match category {
        ItemCategory::Ring => ItemKind::Ring,
        ItemCategory::Necklace => ItemKind::Necklace,
        ItemCategory::Weapon => ItemKind::Dagger,
        _ => ItemKind::Keepsake,
    };
    let seed = poi.seed.derive("quest item").derive("made");
    let order = ItemOrder::plain(kind, Quality::Common);
    let made_at = Some(world.made_now());
    let mut item = make_item(&order, CreatureKind::Human, id, seed, made_at, &mut || {
        crate::ids::new_id()
    });
    if kind == ItemKind::Keepsake {
        item.name = Some(keepsake_name(category).into());
    }
    item.quest = Some(quest.id);
    item
}

/// A keepsake's name, in English as items' own names are for now.
fn keepsake_name(category: ItemCategory) -> &'static str {
    match category {
        ItemCategory::Brooch => "Brooch",
        ItemCategory::Locket => "Locket",
        ItemCategory::Tool => "Tool",
        ItemCategory::Letter => "Letter",
        ItemCategory::Book => "Book",
        ItemCategory::Box => "Box",
        ItemCategory::Instrument => "Fiddle",
        ItemCategory::Toy => "Toy",
        _ => "Keepsake",
    }
}

/// The character has picked up their quest item: its step done if the quest
/// is taken (whatever came first), else remembered and its giver marked.
/// `false` if the client has gone.
pub(crate) fn picked_up(client: &mut Client, world: &mut ServerWorld, quest: HookQuest) -> bool {
    let now = super::clock::unix_now();
    let id = quest.id;
    if let Some(progress) = client.character.quests.taken.get_mut(&id) {
        let Some(made) = cache::quest(world, id) else {
            return true;
        };
        progress.counts.resize(made.steps.len(), 0);
        progress.counts[quest.step] = 1;
        progress.ready = made.ready_to_turn_in(&done_flags(&made, &progress.counts));
        client.unsaved = true;
        return log::tell_changed(client, world, id) && tell_markers(client, world, id.town, now);
    }
    client.character.quests.found.insert(id);
    client.unsaved = true;
    info!(
        "{} found the item of quest {id:?} before taking it",
        client.in_log()
    );
    learn_place(client, world, id.town)
        && super::ahead::tell_ahead(client, world)
        && tell_markers(client, world, id.town, now)
}

/// Sends the sign over the player's quest item lying in sight, if it has
/// changed. `false` if the client has gone.
pub(super) fn tell_finds(client: &mut Client, world: &mut ServerWorld) -> bool {
    let mut marks: Vec<FindMark> = match find_here(client, world) {
        Some(find) => {
            let (x, y) = find.at.cell();
            let seen = client.visit.as_mut().is_some_and(|v| v.can_see(x, y));
            let sign = regard(world, &client.character.quests, find.quest).sign(find.quest.kind);
            let mark = sign
                .filter(|_| seen)
                .map(|sign| FindMark { at: find.at, sign });
            mark.into_iter().collect()
        }
        None => Vec::new(),
    };
    // A quest dungeon's locked stage door has the quest's marker too.
    marks.extend(super::stage_door::stage_mark(client, world));
    if marks == client.quest_seen.finds {
        return true;
    }
    client.quest_seen.finds = marks.clone();
    client
        .link
        .send(&ServerMessage::Quest(QuestNews::Finds(marks)))
}
