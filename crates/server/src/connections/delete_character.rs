//! Deleting a character (dev tools), and asking for a new one in its place.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::{Client, Phase};
use crate::accounts::Accounts;
use crate::{ServerWorld, sites};

/// Deletes the client's character: out of any site or the Hall, their save gone
/// (nothing saved on the way out), and the account given a new character, made
/// as on first joining. A guest's or the single player's account gets a new one
/// of its own, so nothing that remembers the old one knows the new. `false` if
/// the client has gone.
pub(super) fn delete_character(
    client: &mut Client,
    world: &mut ServerWorld,
    accounts: &mut Accounts,
    allowed: shared::difficulty::Difficulties,
) -> bool {
    let Some(old) = client.character_id else {
        return true;
    };
    info!("Deleting character {old:?} (dev tools)");
    sites::leave(client, world);
    world.storage.remove_character(&old);
    let account = accounts.remove_character(&old).or(client.account);
    client.forget_character();
    let new = account.map(|account| match accounts.is_guest(&account) {
        Ok(true) => accounts.guest_character(&account).map(Some),
        Ok(false) => Ok(None),
        Err(error) => Err(error),
    });
    match new.transpose() {
        Ok(new) => client.character_id = new.flatten(),
        Err(error) => return client.refuse_for_saves(&error),
    }
    client.phase = Phase::Choosing;
    // An account's others aren't offered: there is no choosing yet.
    client.link.send(&ServerMessage::Characters {
        list: Vec::new(),
        can_create: true,
        difficulties: allowed,
    })
}
