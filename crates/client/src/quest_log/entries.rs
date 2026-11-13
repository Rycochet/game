//! The log's list in order: quests under way, then those handed in, then
//! those failed or given up, newest first in each.

use shared::protocol::{LogEntry, QuestState};

/// The list's groups, in the order shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Group {
    Active,
    Completed,
    Failed,
}

impl Group {
    pub(super) const ALL: [Self; 3] = [Self::Active, Self::Completed, Self::Failed];

    pub(super) fn of(state: QuestState) -> Self {
        match state {
            QuestState::Taken | QuestState::Ready => Self::Active,
            QuestState::Done => Self::Completed,
            QuestState::Failed | QuestState::Abandoned => Self::Failed,
        }
    }

    /// Its heading's message.
    pub(super) fn heading(self) -> &'static str {
        match self {
            Self::Active => "log-active",
            Self::Completed => "log-completed",
            Self::Failed => "log-failed",
        }
    }
}

/// Each group that has any, with its quests newest first (by when they last
/// changed state).
pub(super) fn grouped<'a>(
    entries: impl IntoIterator<Item = &'a LogEntry>,
) -> Vec<(Group, Vec<&'a LogEntry>)> {
    let entries: Vec<&LogEntry> = entries.into_iter().collect();
    Group::ALL
        .into_iter()
        .filter_map(|group| {
            let mut those: Vec<&LogEntry> = (entries.iter().copied())
                .filter(|entry| Group::of(entry.state) == group)
                .collect();
            those.sort_by_key(|entry| std::cmp::Reverse(entry.at));
            (!those.is_empty()).then_some((group, those))
        })
        .collect()
}
