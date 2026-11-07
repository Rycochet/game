//! A player struck by a mob: harmed, practising Defence, and perhaps killed.

use shared::creature::CreatureKind;
use shared::protocol::ServerMessage;
use shared::vitals::{Points, Vital};

use super::super::Client;
use crate::ServerWorld;
use crate::combat::{Harm, health_left};

/// The player takes this much harm from a creature of this kind, practising
/// Defence, and is killed if it was the last of their Health. How much is left,
/// per mille.
pub(super) fn struck(
    client: &mut Client,
    world: &mut ServerWorld,
    harm: Points,
    foe: CreatureKind,
    tick: u64,
) -> u16 {
    let limits = client.limits(tick);
    let played = client.activity.played;
    client.sheet.vitals.hurt(harm, limits, tick, played);
    client.practise(client.fighter().practised_defending(), foe, tick);
    let left = health_left(&client.sheet.vitals, limits, tick);
    if harm != Points::ZERO {
        let me = client.entity_id();
        world.harmed.push(Harm {
            id: me,
            left,
            told: me,
        });
    }
    if client.sheet.vitals.at(Vital::Health, limits, tick) == Points::ZERO {
        killed(client, world, tick);
        return left;
    }
    client
        .link
        .send(&ServerMessage::Sheet(client.sheet.clone()));
    client.send_vitals(tick);
    left
}

/// Killed: put out of the site onto the world map, whole again, and told
/// (owner, 2026-11-07, for now).
fn killed(client: &mut Client, world: &mut ServerWorld, tick: u64) {
    client.fight.pending = None;
    client.waiting.clear();
    client.sheet.vitals.fill();
    client.unsaved = true;
    crate::sites::killed(client, world);
    client
        .link
        .send(&ServerMessage::Sheet(client.sheet.clone()));
    client.send_vitals(tick);
    client.link.send(&ServerMessage::Killed);
}
