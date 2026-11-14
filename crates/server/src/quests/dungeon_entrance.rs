//! Hidden entrances to quests' own dungeons (owner): near the giver's town,
//! seen only by the quester while the quest is taken, as a poi of its own
//! in a chunk with none (`shared::quest::HIDDEN_KIND`), marked like any
//! quest poi and gone into from the world map as one.

use shared::poi::{ChunkPos, Poi, can_enter_from};
use shared::quest::{DungeonWay, Quest, QuestId};

use crate::connections::Client;

/// The hidden entrance in `chunk` of a quest the character has taken, if any.
pub(crate) fn entrance_in(client: &Client, chunk: ChunkPos) -> Option<(QuestId, Poi)> {
    let book = &client.character.quests;
    (book.dungeons.iter())
        .filter(|(id, _)| book.taken.contains_key(id))
        .find_map(|(&id, stay)| {
            stay.entrance
                .filter(|poi| poi.chunk == chunk)
                .map(|poi| (id, poi))
        })
}

/// A hidden entrance of a quest the character has taken that they are near
/// enough on the world map to go into (`poi::can_enter_from`), if any.
pub(crate) fn entrance_near(client: &Client, width: i64) -> Option<QuestId> {
    let at = client.world_position();
    let book = &client.character.quests;
    (book.dungeons.iter())
        .filter(|(id, _)| book.taken.contains_key(id))
        .find(|(_, stay)| {
            stay.entrance
                .is_some_and(|poi| can_enter_from(&poi, at, width))
        })
        .map(|(&id, _)| id)
}

/// On taking `quest`: a hidden entrance is noted on the character, so their
/// map shows it from now on (sent with its chunk, `world_map::chunk_message`).
pub(super) fn note_entrance(client: &mut Client, quest: &Quest) {
    let Some(dungeon) = quest.dungeon.filter(|d| d.way == DungeonWay::Hidden) else {
        return;
    };
    let stay = client
        .character
        .quests
        .dungeons
        .entry(quest.id)
        .or_default();
    stay.entrance = Some(dungeon.poi);
    client.unsaved = true;
}
