//! A player's Health and Mana (`shared::vitals`): telling the client when they
//! change, and the dev tools that change them.

use shared::protocol::{DEV_BONUS, DEV_BONUS_TICKS, DevVitals, ServerMessage};
use shared::vitals::{Change, Limits, Vital};

use super::Client;

impl Client {
    /// The most Health and Mana its character can have at this tick.
    pub(crate) fn limits(&self, now: u64) -> Limits {
        let sheet = &self.sheet;
        sheet
            .vitals
            .limits(&sheet.attributes, self.activity.played, now)
    }

    /// Tells the client its Health and Mana as they are now. `false` if it
    /// has gone.
    pub(super) fn send_vitals(&mut self, now: u64) -> bool {
        let limits = self.limits(now);
        self.vitals_told = Some(limits);
        let vitals = &self.sheet.vitals;
        self.link.send(&ServerMessage::Vitals {
            who: None,
            health: vitals.told(Vital::Health, limits, now),
            mana: vitals.told(Vital::Mana, limits, now),
        })
    }

    /// Every tick: forgets changes that have ended, and tells the client if
    /// how much there can be has changed (refilling it works out itself).
    /// `false` if it has gone.
    pub(super) fn keep_vitals(&mut self, now: u64) -> bool {
        let before = self.limits(now);
        if self.sheet.vitals.forget_ended(before, now) {
            self.unsaved = true;
        }
        if self.vitals_told == Some(self.limits(now)) {
            return true;
        }
        self.send_vitals(now)
    }

    /// Dev tools: harms, spends, fills or raises them as asked, and says so.
    pub(super) fn dev_vitals(&mut self, what: DevVitals, now: u64) -> bool {
        let limits = self.limits(now);
        let played = self.activity.played;
        let vitals = &mut self.sheet.vitals;
        match what {
            DevVitals::Hurt(amount) => vitals.hurt(amount, limits, now, played),
            DevVitals::SpendMana(amount) => {
                vitals.spend_mana(amount, limits, now, played);
            }
            DevVitals::Fill => vitals.fill(),
            DevVitals::Bonus => {
                for vital in Vital::ALL {
                    let change = Change {
                        vital,
                        amount: DEV_BONUS,
                        until: Some(now + DEV_BONUS_TICKS),
                    };
                    vitals.add_change(change, limits, now);
                }
            }
        }
        self.unsaved = true;
        self.send_vitals(now)
    }
}
