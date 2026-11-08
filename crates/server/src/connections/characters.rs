//! An account's characters while its player chooses: offering them
//! (`Characters`), and deleting one asked for.

use bevy::prelude::*;
use shared::difficulty::Difficulties;
use shared::protocol::{CharacterId, CharacterSummary, ServerMessage};

use super::{Client, Phase};
use crate::ServerWorld;
use crate::accounts::Accounts;

/// Offers a logged-in client its account's characters to choose from. A client
/// without the screens for it (`makes_characters`) plays the latest instead,
/// or a plain new one. `false` if the client has gone.
pub(super) fn offer_characters(
    client: &mut Client,
    accounts: &mut Accounts,
    world: &ServerWorld,
    allowed: Difficulties,
) -> bool {
    let answered = (|| {
        let account = client.account.ok_or("not logged in")?;
        client.character_id = None;
        // Those still on the account, and those of them there is a save to show.
        let mut kept = Vec::new();
        let mut list = Vec::new();
        for summary in accounts.account(&account)?.characters.clone() {
            match world.storage.listed(&summary)? {
                Some(listed) => list.push(listed),
                // Made but never played (as a guest's once was before it was
                // named): nothing to keep, and it would take a guest's place.
                None if summary.last_played == 0 => {
                    accounts.remove_character(&summary.id);
                    continue;
                }
                // Brand new and not saved yet (for a few seconds): nothing to
                // show of them, but they are still the account's.
                None => {}
            }
            kept.push(summary);
        }
        if !client.makes_characters {
            let id = match latest(&kept) {
                Some(id) => id,
                None => accounts.plain_character(&account)?,
            };
            client.character_id = Some(id);
            client.phase = Phase::Joining;
            return Ok(true);
        }
        client.phase = Phase::Choosing;
        let can_create = kept.len() < accounts.most_characters(&account)?;
        Ok(client.link.send(&ServerMessage::Characters {
            list,
            can_create,
            difficulties: allowed,
        }))
    })();
    answered.unwrap_or_else(|error: String| client.refuse_for_saves(&error))
}

/// The character played most recently, or else made most recently.
fn latest(kept: &[CharacterSummary]) -> Option<CharacterId> {
    let newest = kept.iter().max_by_key(|c| (c.last_played, c.created));
    newest.map(|c| c.id)
}

/// Deletes one of the account's characters, as asked while choosing, and offers
/// the rest again. Anyone else's is refused. `false` if the client should go.
pub(super) fn delete_chosen(
    client: &mut Client,
    id: CharacterId,
    accounts: &mut Accounts,
    world: &ServerWorld,
    allowed: Difficulties,
) -> bool {
    let Some(account) = client.account.filter(|_| client.phase == Phase::Choosing) else {
        client.link.send(&ServerMessage::Refused(
            "not the time to delete a character".into(),
        ));
        return false;
    };
    // A client can ask for anything: only its own will do.
    match accounts.account(&account) {
        Ok(found) if found.characters.iter().any(|c| c.id == id) => {}
        Ok(_) => {
            client.link.send(&ServerMessage::Refused(
                "that isn't one of this account's characters".into(),
            ));
            return false;
        }
        Err(error) => return client.refuse_for_saves(&error),
    }
    info!("Account {} deleted character {}", account.0, id.0);
    world.storage.remove_character(&id);
    accounts.remove_character(&id);
    offer_characters(client, accounts, world, allowed)
}

#[cfg(test)]
mod tests;
