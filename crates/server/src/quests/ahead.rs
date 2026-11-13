//! Steps done before a quest is taken (owner): its boss killed (`boss_kill`)
//! or its item found (`finds`). Remembered on the character until it is
//! taken, which does that step; meanwhile the giver's town is marked and the
//! giver says they know.

use shared::protocol::{Ahead, DoneAhead, QuestNews, ServerMessage};
use shared::quest::{Hook, Quest, QuestId, Target};

use super::{cache, done_flags};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::QuestBook;

/// What the character did ahead of taking `id`, if anything.
pub(super) fn ahead_of(book: &QuestBook, id: QuestId) -> Option<Ahead> {
    if book.slain.contains(&id) {
        Some(Ahead::Slain)
    } else if book.found.contains(&id) {
        Some(Ahead::Found)
    } else {
        None
    }
}

/// Every quest with a step done ahead, not taken yet.
pub(super) fn ahead_news(client: &Client, world: &mut ServerWorld) -> QuestNews {
    let book = &client.character.quests;
    let ids = book.slain.iter().chain(&book.found);
    let ahead = ids
        .filter_map(|&id| {
            let quest = cache::quest(world, id)?;
            Some(DoneAhead {
                id,
                kind: quest.kind,
                ahead: ahead_of(book, id)?,
            })
        })
        .collect();
    QuestNews::Ahead(ahead)
}

/// Tells the client. `false` if it has gone.
pub(super) fn tell_ahead(client: &mut Client, world: &mut ServerWorld) -> bool {
    let news = ahead_news(client, world);
    client.link.send(&ServerMessage::Quest(news))
}

/// On taking a quest with a step done ahead: that step is done, and the
/// memory goes. Whether there was one.
pub(super) fn already_done(client: &mut Client, quest: &Quest) -> bool {
    let book = &mut client.character.quests;
    let hook = match ahead_of(book, quest.id) {
        Some(Ahead::Slain) => Hook::Boss,
        Some(Ahead::Found) => Hook::Item,
        None => return false,
    };
    book.slain.remove(&quest.id);
    book.found.remove(&quest.id);
    let step = (quest.steps.iter())
        .position(|step| matches!(step.target, Target::Poi { hook: h, .. } if h == hook));
    if let (Some(step), Some(progress)) = (step, book.taken.get_mut(&quest.id)) {
        progress.counts.resize(quest.steps.len(), 0);
        progress.counts[step] = 1;
        progress.ready = quest.ready_to_turn_in(&done_flags(quest, &progress.counts));
    }
    true
}
