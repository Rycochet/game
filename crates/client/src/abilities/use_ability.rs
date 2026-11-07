//! Using abilities: from a slot pressed (a click, a tap or its key), or by
//! themselves when automatic, whenever they can be done; and keeping the
//! server's timing of the player's own.

use bevy::prelude::*;
use shared::ability::{Ability, BAR_SLOTS};
use shared::combat::{CANCEL_PENALTY, Wielded, in_reach};
use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::protocol::{ClientMessage, Landed, ServerMessage};
use shared::sheet::Feeling;

use super::own_bar::OwnBar;
use crate::common::{
    CurrentDungeon, GameView, MapPosition, Other, OwnAbility, PlayerSpeed, Target,
};
use crate::connection::{FromServer, ToServer};
use crate::inventory::Carried;
use crate::player::Player;

/// How long an ask is waited on before asking again, in seconds: the server
/// answers at once if it does at all.
const ASK_WAIT: f64 = 1.0;

/// A slot on the action bar was used.
#[derive(Message, Clone, Copy, Debug)]
pub(super) struct UseSlot(pub u8);

/// What the server says of the player's ability: when it lands, and when they
/// can act again.
pub(super) fn take_timing(
    mut from_server: MessageReader<FromServer>,
    time: Res<Time>,
    speed: Res<PlayerSpeed>,
    mut own: ResMut<OwnAbility>,
) {
    let now = time.elapsed_secs_f64();
    let seconds = |ticks: u32| f64::from(ticks) / ACTION_TICKS_PER_SECOND;
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::AbilityStarted {
                who: None,
                lands_in,
                ready_in,
                ..
            } => {
                *own = OwnAbility {
                    started: now,
                    lands: now + seconds(*lands_in),
                    ready: now + seconds(*ready_in),
                    asked_until: 0.0,
                };
            }
            ServerMessage::AbilityLanded {
                who: None,
                landed: Landed::Cancelled,
                ..
            } => {
                let penalty = f64::from(CANCEL_PENALTY) / f64::from(speed.0.max(1));
                own.lands = now;
                own.ready = now + penalty / ACTION_TICKS_PER_SECOND;
            }
            _ => {}
        }
    }
}

/// Asks the server for an ability at the target, and waits for its answer.
fn ask(
    ability: Ability,
    target: shared::id::Id,
    now: f64,
    own: &mut OwnAbility,
    to_server: &mut MessageWriter<ToServer>,
) {
    own.asked_until = now + ASK_WAIT;
    to_server.write(ToServer(ClientMessage::UseAbility { ability, target }));
}

/// A slot used: its ability, at the target, if it can be done now.
pub(super) fn use_slots(
    mut used: MessageReader<UseSlot>,
    bar: Res<OwnBar>,
    carried: Res<Carried>,
    target: Res<Target>,
    time: Res<Time>,
    mut own: ResMut<OwnAbility>,
    mut to_server: MessageWriter<ToServer>,
) {
    let now = time.elapsed_secs_f64();
    for &UseSlot(slot) in used.read() {
        let Some(held) = bar.0.slots.get(usize::from(slot)).copied().flatten() else {
            continue;
        };
        if !held.ability.can_use(carried.0.creature) || own.is_busy(now) {
            continue;
        }
        match target.0 {
            Some(target) => ask(held.ability, target, now, &mut own, &mut to_server),
            None => debug!("{} needs a target", held.ability.name()),
        }
    }
}

/// Automatic abilities, done whenever they can be: Attack with an enemy
/// targeted, in sight and in reach.
#[allow(clippy::too_many_arguments)]
pub(super) fn use_automatic(
    view: Res<State<GameView>>,
    bar: Res<OwnBar>,
    carried: Res<Carried>,
    target: Res<Target>,
    dungeon: Res<CurrentDungeon>,
    time: Res<Time>,
    player: Single<&MapPosition, With<Player>>,
    others: Query<(&Other, &MapPosition)>,
    mut own: ResMut<OwnAbility>,
    mut to_server: MessageWriter<ToServer>,
) {
    let now = time.elapsed_secs_f64();
    if *view.get() != GameView::Dungeon || own.is_busy(now) {
        return;
    }
    let Some(id) = target.0 else {
        return;
    };
    let Some((_, at)) = others
        .iter()
        .find(|(other, _)| other.id == id && other.feeling == Feeling::Unfriendly)
    else {
        return;
    };
    let reach = Wielded::of(&carried.0).main.reach();
    if !in_reach(&dungeon.level, (player.x, player.y), (at.x, at.y), reach) {
        return;
    }
    let automatic = bar.0.slots[..BAR_SLOTS]
        .iter()
        .flatten()
        .any(|slot| slot.auto && slot.ability == Ability::Attack);
    if automatic && Ability::Attack.can_use(carried.0.creature) {
        ask(Ability::Attack, id, now, &mut own, &mut to_server);
    }
}
