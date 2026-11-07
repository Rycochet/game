//! Abilities used by hand (a slot clicked, tapped or its key): never lost, and
//! ahead of automatic ones, which wait until they have landed (owner,
//! 2026-11-07).

use bevy::prelude::*;
use shared::ability::Ability;
use shared::protocol::ClientMessage;
use shared::sheet::Feeling;

use super::own_bar::OwnBar;
use super::use_ability::{ASK_WAIT, UseSlot, ask};
use super::vitals_now::affords;
use crate::common::{Other, OwnAbility, OwnVitals, Target};
use crate::connection::ToServer;
use crate::inventory::Carried;

/// The ability the player chose by hand, from pressing it until it lands.
#[derive(Resource, Default, Debug, PartialEq)]
pub(super) struct Manual {
    /// Pressed, and waiting until the player can act to be asked for.
    pub(super) wanted: Option<Ability>,
    /// Asked for and not yet begun: the server's answer is awaited until
    /// `sent_until`.
    pub(super) sent: Option<Ability>,
    pub(super) sent_until: f64,
    /// Begun: when it lands.
    pub(super) lands: f64,
}

impl Manual {
    /// Whether automatic abilities must wait for it.
    pub(super) fn holds(&self, now: f64) -> bool {
        self.wanted.is_some() || (self.sent.is_some() && now < self.sent_until) || now < self.lands
    }

    /// The server began an ability: if it was this one, it holds until it lands.
    pub(super) fn begun(&mut self, ability: Ability, lands: f64) {
        if self.sent == Some(ability) {
            self.sent = None;
            self.lands = lands;
        }
    }

    /// Moving takes the place of whatever was chosen.
    pub(super) fn forget(&mut self) {
        self.wanted = None;
        self.sent = None;
    }
}

/// A slot used: its ability is wanted, in place of any wanted before.
pub(super) fn use_slots(
    mut used: MessageReader<UseSlot>,
    bar: Res<OwnBar>,
    carried: Res<Carried>,
    mut manual: ResMut<Manual>,
) {
    for &UseSlot(slot) in used.read() {
        let Some(held) = bar.0.slots.get(usize::from(slot)).copied().flatten() else {
            continue;
        };
        if held.ability.can_use(carried.0.creature) {
            manual.wanted = Some(held.ability);
        }
    }
}

/// Asks for the wanted ability at the target as soon as the player can act
/// (Heal: at the target unless it is an enemy, otherwise at the player). One
/// asked for and still waiting on the server (an automatic one, or one chosen
/// before) is taken back first, so this goes in its place.
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
    let Some(ability) = manual.wanted else {
        return;
    };
    let waiting_on_server = now < own.asked_until;
    if !waiting_on_server && own.is_busy(now) {
        return;
    }
    manual.wanted = None;
    if !ability.can_use(carried.0.creature) || !affords(ability, &vitals, now) {
        debug!("{} can't be done now", ability.name());
        return;
    }
    let target = match ability {
        // Never an enemy by hand: healing one is to come another way.
        Ability::Heal => target.0.filter(|id| {
            others
                .iter()
                .any(|other| other.id == *id && other.feeling != Feeling::Unfriendly)
        }),
        _ => match target.0 {
            Some(target) => Some(target),
            None => {
                debug!("{} needs a target", ability.name());
                return;
            }
        },
    };
    if waiting_on_server {
        to_server.write(ToServer(ClientMessage::WithdrawAbility));
    }
    ask(ability, target, now, &mut own, &mut to_server);
    manual.sent = Some(ability);
    manual.sent_until = now + ASK_WAIT;
}

#[cfg(test)]
mod tests;
