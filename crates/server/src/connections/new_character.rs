//! Making a character: asking a client to make one when theirs is new, and
//! checking what it asks for.

use shared::difficulty::Difficulties;
use shared::protocol::{NewCharacter, ServerMessage};

use super::{Client, Phase};
use crate::ServerWorld;
use crate::accounts::Accounts;

/// A client whose character is decided: it joins, unless nothing is saved of
/// them yet and the client can make a character, when it is asked to make them
/// first (`Characters` with none to choose). `false` if the client has gone.
pub(super) fn join_or_make(
    client: &mut Client,
    world: &ServerWorld,
    allowed: Difficulties,
) -> bool {
    let id = client.character_id.expect("decided before joining");
    if client.makes_characters {
        match world.storage.load_sheet(&id) {
            Ok(None) => {
                client.phase = Phase::Choosing;
                return client.link.send(&ServerMessage::Characters {
                    list: Vec::new(),
                    can_create: true,
                    difficulties: allowed,
                });
            }
            Ok(Some(_)) => {}
            Err(error) => return client.refuse_for_saves(&error),
        }
    }
    client.phase = Phase::Joining;
    true
}

/// Makes the character a client asked for, or says why not. An account's new
/// character is added to it; a guest's or the single player's, already given,
/// is named. Their difficulty is the one asked for, or the nearest allowed.
pub(super) fn create(
    client: &mut Client,
    asked: NewCharacter,
    accounts: &mut Accounts,
    allowed: Difficulties,
) -> Result<(), String> {
    if client.phase != Phase::Choosing {
        return Err("not the time to make a character".into());
    }
    // A client can ask for anything: the server decides what may be played.
    let def = asked.kind.def();
    if !def.playable {
        return Err(format!("{} can't be played", def.name));
    }
    if !asked.appearance.suits(asked.kind) {
        return Err("that look doesn't suit that kind of creature".into());
    }
    // The server works the attributes out: only the triangles come from it.
    if !asked.build.is_valid() {
        return Err("those attributes can't be chosen".into());
    }
    let id = match (client.character_id, client.account) {
        (Some(id), _) => {
            accounts.rename_character(&id, &asked.name)?;
            id
        }
        (None, Some(account)) => accounts.create_character(&account, &asked.name)?,
        (None, None) => return Err("not logged in".into()),
    };
    client.character_id = Some(id);
    client.starting_as = Some((asked.kind, asked.appearance, asked.build));
    client.difficulty = Some(allowed.closest(asked.difficulty));
    client.phase = Phase::Joining;
    Ok(())
}
