//! Abilities used by hand (a slot clicked, tapped or its key): queued in the
//! order they were pressed, never lost, and ahead of automatic ones, which wait
//! until the queue is done and the last has landed (owner, 2026-11-07).

use std::collections::VecDeque;

use bevy::prelude::*;
use shared::ability::Ability;
use shared::id::Id;
use shared::protocol::ClientMessage;
use shared::sheet::Feeling;

use super::own_bar::OwnBar;
use super::use_ability::{ASK_WAIT, UseSlot, ask};
use super::vitals_now::affords;
use crate::common::{Other, OwnAbility, OwnVitals, Target};
use crate::connection::ToServer;
use crate::inventory::Carried;

/// The abilities the player chose by hand, from pressing them until the last
/// lands.
#[derive(Resource, Default, Debug, PartialEq)]
pub(super) struct Manual {
    /// Pressed, in order, each at most once, waiting their turn to be asked
    /// for. The bar marks them with a moving stripe.
    pub(super) queued: VecDeque<Ability>,
    /// Asked for and not yet begun: the server's answer is awaited until
    /// `sent_until`.
    pub(super) sent: Option<Ability>,
    pub(super) sent_until: f64,
    /// Begun: when it lands.
    pub(super) lands: f64,
}

impl Manual {
    /// Whether automatic abilities must wait for these.
    pub(super) fn holds(&self, now: f64) -> bool {
        !self.queued.is_empty() || self.is_sent(now) || now < self.lands
    }

    /// Whether one was asked for and the server's answer is still awaited.
    fn is_sent(&self, now: f64) -> bool {
        self.sent.is_some() && now < self.sent_until
    }

    /// A press: queued at the back, or taken out if it already waits.
    pub(super) fn press(&mut self, ability: Ability) {
        match self.queued.iter().position(|&one| one == ability) {
            Some(index) => {
                self.queued.remove(index);
            }
            None => self.queued.push_back(ability),
        }
    }

    /// The server began an ability: if it was this one, it holds until it lands.
    pub(super) fn begun(&mut self, ability: Ability, lands: f64) {
        if self.sent == Some(ability) {
            self.sent = None;
            self.lands = lands;
        }
    }

    /// Moving takes the place of everything chosen.
    pub(super) fn forget(&mut self) {
        self.queued.clear();
        self.sent = None;
    }
}

/// A slot used: its ability queued, or taken out of the queue; a toggle
/// (Sneak) asked for at once, as it needs no target and takes no time.
pub(super) fn use_slots(
    mut used: MessageReader<UseSlot>,
    bar: Res<OwnBar>,
    carried: Res<Carried>,
    mut manual: ResMut<Manual>,
    mut to_server: MessageWriter<ToServer>,
) {
    for &UseSlot(slot) in used.read() {
        let Some(held) = bar.0.slots.get(usize::from(slot)).copied().flatten() else {
            continue;
        };
        if !held.ability.can_use(carried.0.creature) {
            continue;
        }
        if held.ability.is_toggle() {
            to_server.write(ToServer(ClientMessage::UseAbility {
                ability: held.ability,
                target: None,
            }));
        } else {
            manual.press(held.ability);
        }
    }
}

/// Asks for the first queued ability that can be done, at the target, as soon
/// as the player can act; those that can't be done when their turn comes are
/// passed over. An automatic one asked for and still waiting on the server is
/// taken back first, so this goes in its place.
#[allow(clippy::too_many_arguments)]
pub(super) fn ask_wanted(
    carried: Res<Carried>,
    target: Res<Target>,
    others: Query<&Other>,
    vitals: Res<OwnVitals>,
    time: Res<Time>,
    mut manual: ResMut<Manual>,
    mut own: ResMut<OwnAbility>,
    mut to_server: MessageWriter<ToServer>,
) {
    let now = time.elapsed_secs_f64();
    if manual.queued.is_empty() {
        return;
    }
    let automatic_waiting = now < own.asked_until && !manual.is_sent(now);
    if !automatic_waiting && own.is_busy(now) {
        return;
    }
    while let Some(ability) = manual.queued.pop_front() {
        let fits = ability.can_use(carried.0.creature) && affords(ability, &vitals, now);
        let Some(at) = fits.then(|| aim(ability, target.0, &others)).flatten() else {
            debug!("{} can't be done now", ability.name());
            continue;
        };
        if automatic_waiting {
            to_server.write(ToServer(ClientMessage::WithdrawAbility));
        }
        ask(ability, at, now, &mut own, &mut to_server);
        manual.sent = Some(ability);
        manual.sent_until = now + ASK_WAIT;
        return;
    }
}

/// Who an ability chosen by hand is aimed at: `Some(None)` for the player
/// themselves, `None` if it can't be aimed. Heal goes to the target unless it
/// is an enemy, otherwise to the player; anything else needs a target.
fn aim(ability: Ability, target: Option<Id>, others: &Query<&Other>) -> Option<Option<Id>> {
    match ability {
        // Never an enemy by hand: healing one is to come another way.
        Ability::Heal => Some(target.filter(|id| {
            others
                .iter()
                .any(|other| other.id == *id && other.feeling != Feeling::Unfriendly)
        })),
        _ => target.map(Some),
    }
}

#[cfg(test)]
mod tests;
