//! A sheet with its bonuses: what is worn and its effects added up
//! (`shared::bonus`), the attributes and most Health and Mana they make, and
//! changing effects while keeping Health's and Mana's share.

use super::{Attributes, Sheet};
use crate::bonus::{Bonus, Cancel, Effect, Source, Stat, Totals, food_effect, item_bonuses};
use crate::item::{Item, ItemKind};
use crate::vitals::Limits;

/// What working out a sheet's limits needs besides the sheet itself.
#[derive(Clone, Copy, Debug)]
pub struct Situation<'a> {
    /// What it wears (`Inventory::worn`): mobs and the server's players.
    pub worn: &'a [Item],
    /// Its playtime in action ticks (0 for mobs).
    pub playtime: u64,
    /// Ticks to refill from nothing to full (`Limits::refill`).
    pub refill: u64,
}

impl Sheet {
    /// Every bonus from what is worn and the effects lasting at `now`, added up.
    pub fn totals(&self, worn: &[Item], now: u64) -> Totals {
        let from_items: Vec<Bonus> = worn.iter().flat_map(item_bonuses).collect();
        Totals::of(from_items.iter().chain(self.effects.bonuses(now)))
    }

    /// Its attributes with their bonuses: what rules should go by.
    pub fn attributes_with(&self, worn: &[Item], now: u64) -> Attributes {
        self.totals(worn, now).attributes(&self.attributes)
    }

    /// The most Health and Mana at `now` with every bonus, and how they come
    /// back. Call `forget_ended` first: an effect ended but not yet forgotten
    /// no longer counts, so what it restored since the last settling is lost.
    pub fn limits(&self, situation: Situation, now: u64) -> Limits {
        let totals = self.totals(situation.worn, now);
        let attributes = totals.attributes(&self.attributes);
        let until = self.effects.next_end(now);
        Limits {
            refill: situation.refill,
            ..self
                .vitals
                .limits_with(&attributes, &totals, situation.playtime, until)
        }
    }

    /// Something worn has changed since its limits were `before`: keeps
    /// Health's and Mana's share (`Vitals::rebase`).
    pub fn rebase(&mut self, before: Limits, situation: Situation, now: u64) {
        let after = self.limits(situation, now);
        self.vitals.rebase(before, after, now);
    }

    /// Changes its effects, keeping Health's and Mana's share. Those ended by
    /// `now` are forgotten first, at their own ends.
    pub fn change_effects<T>(
        &mut self,
        situation: Situation,
        now: u64,
        change: impl FnOnce(&mut crate::bonus::Effects) -> T,
    ) -> T {
        self.forget_ended(situation, now);
        let before = self.limits(situation, now);
        let answer = change(&mut self.effects);
        self.rebase(before, situation, now);
        answer
    }

    /// Lays an effect on, in place of any from the same source.
    pub fn add_effect(&mut self, effect: Effect, situation: Situation, now: u64) {
        self.change_effects(situation, now, |effects| effects.add(effect));
    }

    /// Eats one of these, if eating it does anything (`bonus::food_effect`).
    /// Whether it did.
    pub fn eat(&mut self, kind: ItemKind, situation: Situation, now: u64) -> bool {
        let Some(effect) = food_effect(kind, now) else {
            return false;
        };
        self.add_effect(effect, situation, now);
        true
    }

    /// Ends the effects this ends (before being hurt, call this first, so
    /// restoring counts up to the harm). Whether any were.
    pub fn cancel_effects(&mut self, cancel: Cancel, situation: Situation, now: u64) -> bool {
        self.forget_ended(situation, now);
        if !self.effects.cancelled_by(cancel) {
            return false;
        }
        self.change_effects(situation, now, |effects| effects.cancel(cancel))
    }

    /// Forgets the effects that have ended by `now`, each settled at its own
    /// end, so restoring stops exactly there. Whether any had.
    pub fn forget_ended(&mut self, situation: Situation, now: u64) -> bool {
        let mut any = false;
        while let Some(end) = self.effects.first_ended(now) {
            // Just before it ended, everything ending then still counts.
            let before = self.limits(situation, end.saturating_sub(1));
            self.effects.forget_ended(end);
            self.vitals.rebase(before, self.limits(situation, end), end);
            any = true;
        }
        any
    }

    /// Older saves kept changes to the most on `Vitals`: each becomes an effect.
    pub(super) fn fold_old_changes(&mut self) {
        for change in std::mem::take(&mut self.vitals.changes) {
            self.effects.0.push(Effect {
                source: Source::Dev,
                bonuses: vec![Bonus::flat(
                    Stat::Most(change.vital),
                    change.amount.saturating_mul(1000),
                )],
                start: 0,
                end: change.until,
                ends_on: crate::bonus::EndsOn::NOTHING,
            });
        }
    }
}

#[cfg(test)]
mod tests;
