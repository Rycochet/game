//! Deleting a character (dev tools, or dying on Ironman), and asking for a new
//! one in its place.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::characters::offer_characters;
use super::{Client, Phase};
use crate::accounts::Accounts;
use crate::{ServerWorld, sites};

/// Deletes the client's character: out of any site or the Hall, their save gone
/// (nothing saved on the way out), and their account's others offered to choose
/// from, or another to make. `false` if the client has gone.
pub(super) fn delete_character(
    client: &mut Client,
    world: &mut ServerWorld,
    accounts: &mut Accounts,
    allowed: shared::difficulty::Difficulties,
    why: &str,
) -> bool {
    let Some(old) = client.character_id else {
        return true;
    };
    info!("Deleting character {} ({why})", old.0);
    sites::leave(client, world);
    world.storage.remove_character(&old);
    accounts.remove_character(&old);
    client.forget_character();
    if client.account.is_some() {
        return offer_characters(client, accounts, world, allowed);
    }
    // Playing without an account (tests): nothing to choose from.
    client.phase = Phase::Choosing;
    client.link.send(&ServerMessage::Characters {
        list: Vec::new(),
        can_create: true,
        difficulties: allowed,
    })
}
