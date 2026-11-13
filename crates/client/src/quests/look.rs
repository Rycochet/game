//! How a quest marker looks: "!" or "?", in its kind's colour (owner: yellow
//! for normal quests, blue for dailies, orange for the artifact line), a grey
//! "!" for a quest that can't be taken yet, or a grey "?" for one taken and
//! not finished (owner).

use bevy::prelude::*;
use shared::protocol::{Marker, QuestState};
use shared::quest::QuestKind;

/// What a marker shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MarkLook {
    /// "?": one to hand in; else "!".
    pub ready: bool,
    /// Its colour, as sRGB bytes.
    pub rgb: [u8; 3],
}

use crate::common::{
    QUEST_BLUE as BLUE, QUEST_GREY as GREY, QUEST_ORANGE as ORANGE, QUEST_YELLOW as YELLOW,
};

impl MarkLook {
    pub fn of(marker: Marker) -> Self {
        match marker {
            Marker::Available(kind) => Self::new(kind, false),
            Marker::Ready(kind) => Self::new(kind, true),
            Marker::Locked => Self {
                ready: false,
                rgb: GREY,
            },
            Marker::Taken => Self {
                ready: true,
                rgb: GREY,
            },
        }
    }

    /// The marker a quest of the character's gives its giver: "?" ready to
    /// hand in, a grey "?" under way, and the "!" it was offered with once it
    /// is over (handed in, failed or given up), for the Quest Log.
    pub fn of_state(kind: QuestKind, state: QuestState) -> Self {
        match state {
            QuestState::Ready => Self::new(kind, true),
            QuestState::Taken => Self::of(Marker::Taken),
            QuestState::Done | QuestState::Failed | QuestState::Abandoned => Self::new(kind, false),
        }
    }

    pub fn new(kind: QuestKind, ready: bool) -> Self {
        let rgb = match kind {
            QuestKind::Normal => YELLOW,
            QuestKind::Daily => BLUE,
            QuestKind::Artifact => ORANGE,
        };
        Self { ready, rgb }
    }

    pub fn glyph(self) -> &'static str {
        if self.ready { "?" } else { "!" }
    }

    /// Its badge's class in markup (`span.mark-<colour>`).
    pub fn class(self) -> &'static str {
        match self.rgb {
            BLUE => "mark-blue",
            ORANGE => "mark-orange",
            GREY => "mark-grey",
            _ => "mark-yellow",
        }
    }

    pub fn colour(self) -> Color {
        let [r, g, b] = self.rgb;
        Color::srgb_u8(r, g, b)
    }
}
