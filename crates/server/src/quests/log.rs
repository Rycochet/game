//! The Quest Log as a client is told it: each of the character's quests on
//! the planet, made again from its town's seed, and their daily givers.

use shared::protocol::{DailyDone, LogEntry, QuestNews, QuestState};
use shared::quest::{Quest, QuestId};

use super::{cache, done_flags};
use crate::ServerWorld;
use crate::connections::Client;

/// One of the character's quests, as the log shows it, or `None` if it isn't
/// theirs (or its town's quests have changed since).
pub(crate) fn entry(client: &Client, world: &mut ServerWorld, id: QuestId) -> Option<LogEntry> {
    let book = &client.character.quests;
    let quest = cache::quest(world, id)?;
    let (state, at, counts) = if let Some(progress) = book.taken.get(&id) {
        let ready = quest.ready_to_turn_in(&done_flags(&quest, &progress.counts));
        let state = if ready {
            QuestState::Ready
        } else {
            QuestState::Taken
        };
        (state, progress.at, progress.counts.clone())
    } else if let Some(&at) = book.done.get(&id) {
        let full = quest
            .steps
            .iter()
            .map(|step| step.count.unwrap_or(1))
            .collect();
        (QuestState::Done, at, full)
    } else {
        let ended = book.ended.get(&id)?;
        let state = if ended.failed {
            QuestState::Failed
        } else {
            QuestState::Abandoned
        };
        (state, ended.at, vec![0; quest.steps.len()])
    };
    Some(as_entry(quest, state, counts, at))
}

fn as_entry(quest: Quest, state: QuestState, counts: Vec<u32>, at: i64) -> LogEntry {
    LogEntry {
        id: quest.id,
        kind: quest.kind,
        building: quest.building,
        state,
        story: Box::new(quest.story),
        steps: quest.steps,
        counts,
        reward: quest.reward,
        at,
    }
}

/// The whole log for this planet.
pub(crate) fn whole_log(client: &Client, world: &mut ServerWorld) -> QuestNews {
    let book = &client.character.quests;
    let mut ids: Vec<QuestId> = book.taken.keys().copied().collect();
    ids.extend(book.done.keys());
    ids.extend(
        book.ended
            .keys()
            .filter(|id| !book.taken.contains_key(id) && !book.done.contains_key(id)),
    );
    let quests = ids
        .into_iter()
        .filter_map(|id| entry(client, world, id))
        .collect();
    let dailies = (book.dailies.iter())
        .map(|(&(town, building), daily)| DailyDone {
            town,
            building,
            count: daily.count,
            last: daily.last,
        })
        .collect();
    QuestNews::Log { quests, dailies }
}

/// Tells the client one quest has changed. `false` if it has gone.
pub(crate) fn tell_changed(client: &mut Client, world: &mut ServerWorld, id: QuestId) -> bool {
    match entry(client, world, id) {
        Some(entry) => {
            client
                .link
                .send(&shared::protocol::ServerMessage::Quest(QuestNews::Changed(
                    entry,
                )))
        }
        None => true,
    }
}
