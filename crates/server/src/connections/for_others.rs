//! Dev tools aimed at another player (Give Item, Learn Skills): done once every
//! client's messages this tick have been read, as only then can two clients be
//! reached at once. Only for a player on the same level and in the asker's
//! sight; otherwise nothing is done.

use bevy::prelude::*;
use shared::id::Id;
use shared::protocol::{DevSkills, ServerMessage};

use super::Client;
use super::give_items::{Batch, give_to_player};
use super::heal::pair;
use crate::ServerWorlds;

/// What a client asked to do to another player.
#[derive(Debug)]
pub(crate) enum ForOther {
    Gift(Batch),
    Skills(DevSkills),
}

/// Does what each client asked for another player.
pub(in crate::connections) fn do_for_others(clients: &mut [Client], worlds: &mut ServerWorlds) {
    for i in 0..clients.len() {
        let asked = std::mem::take(&mut clients[i].for_others);
        for (to, what) in asked {
            let world = clients[i].world;
            let found = clients
                .iter()
                .position(|other| !other.gone && other.world == world && other.entity_id() == to);
            let Some(j) = found.filter(|&j| j != i) else {
                warn!("Nobody to do {what:?} to: {to:?}");
                continue;
            };
            let (asker, other) = pair(clients, i, j);
            if !in_sight(asker, other) {
                warn!("Can't do {what:?} to {to:?}: out of sight");
                continue;
            }
            let world = worlds.of(world);
            match what {
                ForOther::Gift(batch) => give_to_player(other, world, &batch),
                ForOther::Skills(change) => {
                    other.change_skills(change, world.action_clock.ticks());
                    other
                        .link
                        .send(&ServerMessage::Sheet(other.sheet.as_sent()));
                    asker.link.send(&ServerMessage::DevSkillsOf {
                        who: Some(to),
                        skills: other.sheet.skills.clone(),
                    });
                }
            }
        }
    }
}

/// Whether `other` stands on the asker's level where the asker can see.
fn in_sight(asker: &mut Client, other: &Client) -> bool {
    let at = other
        .visit
        .as_ref()
        .zip(asker.visit.as_ref())
        .filter(|(theirs, mine)| theirs.population() == mine.population())
        .map(|(theirs, _)| theirs.position());
    let visit = asker.visit.as_mut();
    at.zip(visit)
        .is_some_and(|((x, y), visit)| visit.can_see(x, y))
}

/// Asks for `what` to be done to the player `to` once messages are read.
pub(in crate::connections) fn ask(client: &mut Client, to: Id, what: ForOther) {
    client.for_others.push((to, what));
}
