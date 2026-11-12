//! Using abilities: from a slot pressed (a click, a tap or its key), or by
//! themselves when automatic, whenever they can be done; and keeping the
//! server's timing of the player's own.

use bevy::prelude::*;
use shared::ability::{Ability, BAR_SLOTS};
use shared::combat::{Body, CANCEL_PENALTY, Wielded, in_reach};
use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::protocol::{ClientMessage, Landed, ServerMessage};
use shared::sheet::Feeling;

use super::manual::Manual;
use super::own_bar::OwnBar;
use super::vitals_now::{affords, wants_healing};
use crate::common::{
    CurrentDungeon, DrawnGround, GameView, Other, OwnAbility, OwnVitals, PlayerSize, PlayerSpeed,
    SpentEnergy, SpotPosition, Target,
};
use crate::connection::{FromServer, ToServer};
use crate::inventory::Carried;
use crate::player::{Player, PlayerMoving};

/// How long an ask is waited on before asking again, in seconds: the server
/// answers at once if it does at all.
pub(super) const ASK_WAIT: f64 = 1.0;

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
    mut manual: ResMut<Manual>,
    mut spent: MessageWriter<SpentEnergy>,
) {
    let now = time.elapsed_secs_f64();
    let seconds = |ticks: u32| f64::from(ticks) / ACTION_TICKS_PER_SECOND;
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::AbilityStarted {
                who: None,
                ability,
                lands_in,
                ready_in,
                ..
            } => {
                // Steps wait for it here as on the server, and a journey stops.
                spent.write(SpentEnergy(ready_in.saturating_mul(speed.0)));
                *own = OwnAbility {
                    started: now,
                    lands: now + seconds(*lands_in),
                    ready: now + seconds(*ready_in),
                    asked_until: 0.0,
                };
                manual.begun(*ability, own.lands);
            }
            ServerMessage::AbilityLanded {
                who: None,
                landed: Landed::Cancelled,
                ..
            } => {
                let penalty = f64::from(CANCEL_PENALTY) / f64::from(speed.0.max(1));
                own.lands = now;
                own.ready = now + penalty / ACTION_TICKS_PER_SECOND;
                manual.lands = now;
            }
            _ => {}
        }
    }
}

/// Asks the server for an ability at the target (`None`: the player).
pub(super) fn ask(
    ability: Ability,
    target: Option<shared::id::Id>,
    now: f64,
    own: &mut OwnAbility,
    to_server: &mut MessageWriter<ToServer>,
) {
    own.asked_until = now + ASK_WAIT;
    to_server.write(ToServer(ClientMessage::UseAbility { ability, target }));
}

/// Automatic abilities, done whenever they can be, taking turns in the bar's
/// order: the first whose time has come is asked for (owner, 2026-11-07).
/// Attack with an enemy targeted, in sight and in reach; Heal on the player at
/// a quarter of their Health, with the Mana. Never while the player is moving
/// or an ability they chose by hand is waiting or winding up: those come first.
#[allow(clippy::too_many_arguments)]
pub(super) fn use_automatic(
    view: Res<State<GameView>>,
    bar: Res<OwnBar>,
    carried: Res<Carried>,
    target: Res<Target>,
    (dungeon, ground): (Res<CurrentDungeon>, Res<DrawnGround>),
    (time, vitals): (Res<Time>, Res<OwnVitals>),
    (player, size): (Single<&SpotPosition, With<Player>>, Res<PlayerSize>),
    others: Query<(&Other, &SpotPosition)>,
    (moving, manual): (Res<PlayerMoving>, Res<Manual>),
    mut own: ResMut<OwnAbility>,
    mut to_server: MessageWriter<ToServer>,
) {
    let now = time.elapsed_secs_f64();
    if *view.get() != GameView::Dungeon
        || own.is_busy(now)
        || moving.is_moving()
        || manual.holds(now)
    {
        return;
    }
    let kind = carried.0.creature;
    let automatic = bar.0.slots[..BAR_SLOTS]
        .iter()
        .flatten()
        .filter(|slot| slot.auto && slot.ability.can_use(kind))
        .map(|slot| slot.ability)
        .filter(|ability| affords(*ability, &vitals, now));
    for ability in automatic {
        let at = match ability {
            Ability::Attack => {
                let reach = Wielded::of(&carried.0).main.reach();
                let relief = ground.relief.as_ref();
                let me = Body::on(relief, player.0, size.0);
                let enemy = target.0.and_then(|id| {
                    others
                        .iter()
                        .find(|(other, _)| other.id == id && other.feeling == Feeling::Unfriendly)
                });
                enemy
                    .filter(|(other, at)| {
                        let them = Body::on(relief, at.0, other.kind.def().size);
                        in_reach(&dungeon.level, &me, reach, &them)
                    })
                    .map(|(other, _)| Some(other.id))
            }
            Ability::Heal => wants_healing(&vitals, now).then_some(None),
            _ => None,
        };
        if let Some(at) = at {
            ask(ability, at, now, &mut own, &mut to_server);
            return;
        }
    }
}
