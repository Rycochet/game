//! Handing a quest in at its giver's door once every step is done: the
//! reward is all or nothing (owner), so if it doesn't all fit in what the
//! player carries, the giver keeps it and the quest stays ready ("?") until it
//! does. Then the giver tells the ending, standing rises in the town, and the
//! quest's deeds move the character's alignment.

use bevy::prelude::*;
use shared::item::{Item, coin_items};
use shared::protocol::{QuestNews, QuestRefusal, ServerMessage};
use shared::quest::{Quest, QuestId, QuestKind};
use shared::story::RewardPart;

use super::accept::refuse;
use super::standing::{FOR_DAILY, FOR_QUEST, add};
use super::{cache, done_flags, log, tell_markers};
use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;

/// Hands in `id` (`reach`: from beside the giver's door; the debug API leaves
/// that out). `false` if the client has gone.
pub(crate) fn turn_in(
    client: &mut Client,
    world: &mut ServerWorld,
    id: QuestId,
    now: i64,
    reach: bool,
) -> bool {
    let Some(quest) = cache::quest(world, id) else {
        return true;
    };
    if reach && !crate::sites::beside_giver(client, world, id.town, quest.building) {
        warn!("{} handed in a quest away from its giver", client.in_log());
        return true;
    }
    let ready = (client.character.quests.taken.get(&id))
        .is_some_and(|progress| quest.ready_to_turn_in(&done_flags(&quest, &progress.counts)));
    if !ready {
        return refuse(client, id, QuestRefusal::NotReady);
    }
    // The quest's own items go back to the giver first (a delivery's parcel
    // is long gone; anything found is handed over).
    let mut inventory = client.inventory.clone();
    inventory.remove_quest_items(id);
    for item in reward_items(world, &quest) {
        if inventory.give(item, world.stacks, &mut new_id).is_err() {
            return refuse(client, id, QuestRefusal::NoRoom);
        }
    }
    client.inventory = inventory;
    finish(client, &quest, now);
    info!("{} handed in quest {id:?}", client.in_log());
    client.update_speed(world.action_clock.ticks());
    client.send_inventory()
        && client.link.send(&ServerMessage::Quest(QuestNews::TurnedIn {
            id,
            ending: quest.ending.clone(),
        }))
        && log::tell_changed(client, world, id)
        && tell_markers(client, world, id.town, now)
}

/// What the giver hands over: its coins, as the fewest stacks. Items,
/// training and favours aren't given yet (later Quest system PRs).
fn reward_items(world: &ServerWorld, quest: &Quest) -> Vec<Item> {
    let copper: u64 = (quest.reward.iter())
        .map(|part| match part {
            RewardPart::Coins(copper) => *copper,
            _ => 0,
        })
        .sum();
    coin_items(
        copper,
        world.stacks.most,
        Some(world.made_now()),
        &mut new_id,
    )
}

/// The quest is done: kept as done (dailies counted by giver), standing in its
/// town rises, and its deeds move alignment.
fn finish(client: &mut Client, quest: &Quest, now: i64) {
    let book = &mut client.character.quests;
    book.taken.remove(&quest.id);
    book.ended.remove(&quest.id);
    let points = if quest.kind == QuestKind::Daily {
        let daily = book
            .dailies
            .entry((quest.id.town, quest.building))
            .or_default();
        daily.count += 1;
        daily.last = now;
        FOR_DAILY
    } else {
        book.done.insert(quest.id, now);
        FOR_QUEST
    };
    add(book, quest.id.town, points);
    if let Some(alignment) = client.sheet.alignment.as_mut() {
        for deed in &quest.deeds {
            *alignment = alignment.after(*deed);
        }
    }
    client.unsaved = true;
}
