//! How a quest marker looks: "!" or "?", in its kind's colour (owner: yellow
//! for normal quests, blue for dailies, orange for the artifact line), or a
//! grey "!" for a quest that can't be taken yet.

use bevy::prelude::*;
use shared::protocol::Marker;
use shared::quest::QuestKind;

/// What a marker shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MarkLook {
    /// "?": one to hand in; else "!".
    pub ready: bool,
    /// Its colour, as sRGB bytes.
    pub rgb: [u8; 3],
}

const YELLOW: [u8; 3] = [255, 214, 40];
const BLUE: [u8; 3] = [80, 160, 255];
const ORANGE: [u8; 3] = [255, 140, 30];
const GREY: [u8; 3] = [165, 165, 165];

impl MarkLook {
    pub fn of(marker: Marker) -> Self {
        match marker {
            Marker::Available(kind) => Self::new(kind, false),
            Marker::Ready(kind) => Self::new(kind, true),
            Marker::Locked => Self {
                ready: false,
                rgb: GREY,
            },
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

    pub fn colour(self) -> Color {
        let [r, g, b] = self.rgb;
        Color::srgb_u8(r, g, b)
    }
}
