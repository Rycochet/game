//! The World Gates a character has gone through, which their client draws
//! with orange water: kept by the Hall gate each pair is, so going through
//! either side marks both.

use shared::protocol::ServerMessage;

use super::Client;

impl Client {
    /// The player has gone through the gates of Hall gate `place` (a
    /// `GatePlace::number`): remembered, and the client told if it is new.
    /// `false` if the client has gone.
    pub(crate) fn went_through(&mut self, place: u32) -> bool {
        if !self.gates.insert(place) {
            return true;
        }
        self.unsaved = true;
        self.send_visited_gates()
    }

    /// Tells the client every gate the player has gone through.
    pub(super) fn send_visited_gates(&self) -> bool {
        let places = self.gates.iter().copied().collect();
        self.link.send(&ServerMessage::VisitedGates(places))
    }
}
