//! Making a character: asking a client to make one when theirs is new, and
//! checking what it asks for.

use shared::difficulty::Difficulties;
use shared::protocol::{NewCharacter, ServerMessage};

use super::{Client, Phase};
use crate::ServerWorld;
use crate::accounts::Accounts;
use crate::storage::now;

/// What the player who claimed the server is told, as it sends them back to
/// the menu.
pub(super) const CLAIMED: &str = "this account is now the server's Owner: connect again to play";

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

/// Why a new character wasn't made: as a turn the player can put right
/// (`ServerMessage::CharacterRefused`, still choosing), or as one that ends the
/// connection (`Refused`), or saves that can't be reached.
#[derive(Debug, PartialEq)]
pub(super) enum NotMade {
    /// Choose again: the name taken, too many characters, what the client
    /// shouldn't have sent.
    Again(String),
    /// Hung up: not the time for it, or the server just claimed.
    Gone(String),
    /// The account's saves couldn't be reached (not told to the player).
    Saves(String),
}

impl NotMade {
    /// Tells the client; `false` if it should be disconnected.
    pub(super) fn tell(self, client: &Client) -> bool {
        match self {
            NotMade::Again(reason) => client.link.send(&ServerMessage::CharacterRefused(reason)),
            NotMade::Gone(reason) => {
                client.link.send(&ServerMessage::Refused(reason));
                false
            }
            NotMade::Saves(error) => client.refuse_for_saves(&error),
        }
    }
}

/// Makes the character a client asked for, or says why not: added to the
/// account (or, with none, the one already given: tests). Their difficulty is
/// the one asked for, or the nearest allowed.
pub(super) fn create(
    client: &mut Client,
    asked: NewCharacter,
    accounts: &mut Accounts,
    allowed: Difficulties,
) -> Result<(), NotMade> {
    if client.phase != Phase::Choosing {
        return Err(NotMade::Gone("not the time to make a character".into()));
    }
    // Whoever runs a server with no Owner claims it by naming a character
    // with the code it made: no character is made, and they connect again.
    if let Some(account) = client.account {
        // Loaded here, so what follows can only fail by the rules.
        let guest = accounts.account(&account).map_err(NotMade::Saves)?.guest;
        if let Some(claimed) = accounts.roles.try_claim(account, guest, &asked.name, now()) {
            claimed.map_err(NotMade::Again)?;
            return Err(NotMade::Gone(CLAIMED.into()));
        }
    }
    // A client can ask for anything: the server decides what may be played.
    let def = asked.kind.def();
    if !def.playable {
        return Err(NotMade::Again(format!("{} can't be played", def.name)));
    }
    if !asked.appearance.suits(asked.kind, asked.gender) {
        return Err(NotMade::Again(
            "that look doesn't suit that kind of creature".into(),
        ));
    }
    // The server works the attributes out: only the triangles come from it.
    if !asked.build.is_valid() {
        return Err(NotMade::Again("those attributes can't be chosen".into()));
    }
    let id = match (client.account, client.character_id) {
        (Some(account), _) => accounts
            .create_character(&account, &asked.name)
            .map_err(NotMade::Again)?,
        (None, Some(id)) => id,
        (None, None) => return Err(NotMade::Gone("not logged in".into())),
    };
    client.character_id = Some(id);
    client.starting_as = Some((asked.kind, asked.gender, asked.appearance, asked.build));
    client.difficulty = Some(allowed.closest(asked.difficulty));
    client.phase = Phase::Joining;
    Ok(())
}
