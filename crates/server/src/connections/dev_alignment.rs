//! The debug menu's Alignment panel: the character's hidden alignment asked
//! after, moved or set (`DevAlignment`). The only way it reaches a client.

use shared::alignment::Alignment;
use shared::protocol::{DevAlignment, ServerMessage};

use super::Client;

impl Client {
    /// Does what `change` says to the character's alignment and tells them
    /// what it is now. Whether the client is still there.
    pub(super) fn dev_alignment(&mut self, change: DevAlignment) -> bool {
        let was = self.sheet.alignment;
        let now = was.unwrap_or_default();
        let now = match change {
            DevAlignment::Ask => now,
            DevAlignment::Shift(by) => now.shifted(i32::from(by)),
            DevAlignment::Set(to) => Alignment::new(i32::from(to)),
        };
        if was != Some(now) {
            self.sheet.alignment = Some(now);
            self.unsaved = true;
        }
        self.link.send(&ServerMessage::DevAlignment(now))
    }
}
