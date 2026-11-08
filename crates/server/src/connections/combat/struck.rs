//! A player struck by a mob: harmed, practising Defence, and perhaps killed.

use bevy::prelude::*;
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

/// Killed: their skills take their difficulty's rust, what it drops falls
/// where they died (Hard and Ironman), then they are put out of the site onto the world map, whole
/// again, and told (owner, 2026-11-07, for now). On Ironman they are deleted
/// after this tick's blows (`died_for_good`).
fn killed(client: &mut Client, world: &mut ServerWorld, tick: u64) {
    client.fight.pending = None;
    client.waiting.clear();
    client.sheet.vitals.fill();
    client.unsaved = true;
    let difficulty = client.playing_on();
    client
        .sheet
        .skills
        .rust_for_hours(difficulty.rust_on_death());
    let dropped = client.inventory.drop_on_death(difficulty);
    if !dropped.is_empty() {
        info!("{} dropped {} things on dying", client.name, dropped.len());
        for item in dropped {
            crate::ground::put_down(client, world, item);
        }
        client.speed = client.sheet.speed(&client.inventory);
        client.send_inventory();
    }
    let permanent = difficulty.permanent_death();
    client.died_for_good = permanent;
    crate::sites::killed(client, world);
    client
        .link
        .send(&ServerMessage::Sheet(client.sheet.clone()));
    client.send_vitals(tick);
    client.link.send(&ServerMessage::Killed { permanent });
}
