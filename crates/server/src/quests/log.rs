//! The Quest Log as a client is told it: each of the character's quests on
//! the planet, made again from its town's seed, and their daily givers.

use shared::protocol::{DailyDone, LogEntry, QuestNews, QuestState};
use shared::quest::{Quest, QuestId, utc_day};

use super::{cache, done_flags};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::QuestBook;

/// One of the character's quests, as the log shows it, or `None` if it isn't
/// theirs (or its town's quests have changed since).
pub(crate) fn entry(client: &Client, world: &mut ServerWorld, id: QuestId) -> Option<LogEntry> {
    entry_in(&client.character.quests, world, id)
}

/// One quest of `book`, a planet's, made again in `world`, that planet's.
fn entry_in(book: &QuestBook, world: &mut ServerWorld, id: QuestId) -> Option<LogEntry> {
    let quest = super::prize::as_seen(book, cache::quest(world, id)?);
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
    let (quests, dailies) = book_log(book, world);
    QuestNews::Log {
        quests,
        dailies,
        tracked: book.tracked,
    }
}

/// Every quest in `book` and its daily givers, made again in `world`: the
/// planet the book is for.
pub(crate) fn book_log(
    book: &QuestBook,
    world: &mut ServerWorld,
) -> (Vec<LogEntry>, Vec<DailyDone>) {
    let mut ids: Vec<QuestId> = book.taken.keys().copied().collect();
    ids.extend(book.done.keys());
    ids.extend(
        book.ended
            .keys()
            .filter(|id| !book.taken.contains_key(id) && !book.done.contains_key(id)),
    );
    let quests = ids
        .into_iter()
        .filter_map(|id| entry_in(book, world, id))
        .collect();
    let dailies = (book.dailies.iter())
        .map(|(&(town, building), daily)| {
            // Any day's daily names its giver and town.
            let last = cache::daily(world, town, building, utc_day(daily.last));
            DailyDone {
                town,
                building,
                giver: (last.as_ref()).map(|quest| quest.story.cast.giver().person.clone()),
                home: (last.as_ref())
                    .map_or_else(String::new, |quest| quest.story.facts.home.clone()),
                count: daily.count,
                last: daily.last,
            }
        })
        .collect();
    (quests, dailies)
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
