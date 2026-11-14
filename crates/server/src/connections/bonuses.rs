//! A player's bonuses (`shared::bonus`): what they wear as last counted, taken
//! in again whenever it changes while keeping Health's and Mana's share; the
//! attributes with every bonus, which rules go by and the client is told; and
//! effects ended by moving or being hurt.

use shared::bonus::Cancel;
use shared::item::Item;
use shared::protocol::ServerMessage;
use shared::sheet::{Attributes, Sheet, Situation};

use super::Client;

/// What a player's bonuses were last worked out from, and what came of it.
#[derive(Default)]
pub(crate) struct Bonused {
    /// What was worn: its limits go by this until `keep_worn` sees a change.
    pub(super) wearing: Vec<Item>,
    /// The attributes with every bonus, for fighting.
    pub(super) attributes: Attributes,
    /// What its client was last told they are.
    told: Option<Attributes>,
}

impl Client {
    /// Its sheet, and what the sheet's limits go by besides: taken apart so
    /// both can be used at once.
    pub(super) fn sheet_and_situation(&mut self) -> (&mut Sheet, Situation<'_>) {
        let refill = self.playing_on().refill_ticks();
        let situation = Situation {
            worn: &self.bonused.wearing,
            playtime: self.activity.played,
            refill,
        };
        (&mut self.sheet, situation)
    }

    /// On joining: what it wears counts as it is, with no share to keep.
    pub(super) fn start_wearing(&mut self, now: u64) {
        self.bonused.wearing = self.inventory.worn.clone();
        self.bonused.told = None;
        self.refresh_attributes(now);
    }

    /// Takes in what it wears if that has changed (putting on, taking off,
    /// swapping, binning, giving, dying, a new body), keeping Health's and
    /// Mana's share (`Sheet::rebase`) after settling them at the old limits.
    /// Whether it had.
    pub(crate) fn keep_worn(&mut self, now: u64) -> bool {
        if self.bonused.wearing == self.inventory.worn {
            return false;
        }
        let (sheet, situation) = self.sheet_and_situation();
        sheet.forget_ended(situation, now);
        let before = sheet.limits(situation, now);
        self.bonused.wearing = self.inventory.worn.clone();
        let (sheet, situation) = self.sheet_and_situation();
        sheet.rebase(before, situation, now);
        self.refresh_attributes(now);
        self.unsaved = true;
        true
    }

    /// Works out its attributes with every bonus again.
    pub(super) fn refresh_attributes(&mut self, now: u64) {
        let attributes = self.sheet.attributes_with(&self.bonused.wearing, now);
        self.bonused.attributes = attributes;
    }

    /// Tells the client its attributes with their bonuses, if they have
    /// changed since it was last told. `false` if it has gone.
    pub(super) fn tell_attributes(&mut self) -> bool {
        let now = Some(self.bonused.attributes);
        if self.bonused.told == now {
            return true;
        }
        self.bonused.told = now;
        let attributes = ServerMessage::Attributes(self.bonused.attributes);
        self.link.send(&attributes)
    }

    /// Ends the effects this ends, telling the client if any were. `false` if
    /// it has gone.
    pub(crate) fn cancel_effects(&mut self, cancel: Cancel, now: u64) -> bool {
        let (sheet, situation) = self.sheet_and_situation();
        if !sheet.cancel_effects(cancel, situation, now) {
            return true;
        }
        self.unsaved = true;
        self.refresh_attributes(now);
        self.link.send(&ServerMessage::Sheet(self.sheet.as_sent())) && self.send_vitals(now)
    }

    /// After a step or a move from `from`: moving ends what moving ends.
    pub(super) fn walked(&mut self, from: (i64, i64), now: u64) {
        if self.standing() != from {
            self.cancel_effects(Cancel::Moving, now);
        }
    }
}
