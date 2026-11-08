//! Logging in: as a guest, with a session token, or on a provider's website
//! (`oidc`), whose logins are finished here once the player is back
//! (`finish_logins`). Account events are logged by the account's id, never by
//! anything a person could be known by.

use bevy::prelude::*;
use shared::protocol::{Credentials, EmailAddress, ServerMessage};

use super::new_character::join_or_make;
use super::{Client, Connections, Phase};
use crate::accounts::{AccountId, Accounts, LoginKey, MAX_CHARACTERS};
use crate::oidc::Logins;
use crate::{ServerOptions, ServerWorld, ServerWorlds};

/// Logs in with what the client sent: `false` if it should be disconnected.
pub(super) fn log_in(
    client: &mut Client,
    credentials: Credentials,
    world: &ServerWorld,
    accounts: &mut Accounts,
    options: &ServerOptions,
    logins: Option<&Logins>,
) -> bool {
    let failed = |client: &Client, reason: &str| {
        client.link.send(&ServerMessage::LoginFailed(reason.into()))
    };
    match (credentials, logins) {
        (Credentials::Guest(_), _) if !options.guests => {
            client.link.send(&ServerMessage::Refused(
                "this server doesn't take guests".into(),
            ));
            false
        }
        (Credentials::Guest(token), _) => match accounts.log_in(&LoginKey::Guest(token)) {
            Ok(id) => logged_in(client, id, "as a guest", accounts, world, options),
            Err(error) => client.refuse_for_saves(&error),
        },
        (Credentials::Session(_) | Credentials::Provider(_), None) => {
            failed(client, "this server doesn't take account logins just now")
        }
        (Credentials::Session(token), Some(_)) => match accounts.session(&token) {
            Ok(Some(id)) => logged_in(client, id, "again", accounts, world, options),
            Ok(None) => failed(client, "this login has run out: log in again"),
            Err(error) => client.refuse_for_saves(&error),
        },
        (Credentials::Provider(name), Some(logins)) => match logins.begin(&name) {
            Ok((state, url)) => {
                if let Some(old) = client.login_state.replace(state) {
                    logins.forget(&old);
                }
                client.link.send(&ServerMessage::OpenLogin { url })
            }
            Err(reason) => failed(client, &reason),
        },
    }
}

/// A client has logged in to an account: a guest goes to their one character
/// (made first, if new: `join_or_make`), anyone else is told theirs to choose
/// from. `how` says how, for the log.
pub(super) fn logged_in(
    client: &mut Client,
    id: AccountId,
    how: &str,
    accounts: &mut Accounts,
    world: &ServerWorld,
    options: &ServerOptions,
) -> bool {
    info!("Account {} logged in {how}", id.0);
    let answered = (|| {
        let account = accounts.account(&id)?;
        client.account = Some(id);
        let answered = if account.guest {
            client.character_id = Some(accounts.guest_character(&id)?);
            join_or_make(client, world, options.difficulties)
        } else {
            client.phase = Phase::Choosing;
            client.link.send(&ServerMessage::Characters {
                list: account.characters.clone(),
                can_create: account.characters.len() < MAX_CHARACTERS,
                difficulties: options.difficulties,
            })
        };
        Ok::<bool, String>(answered)
    })();
    answered.unwrap_or_else(|error| client.refuse_for_saves(&error))
}

/// Finishes the logins players have come back from providers' websites with,
/// for the clients still waiting on them.
pub(crate) fn finish_logins(
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
    options: Res<ServerOptions>,
    logins: Option<Res<Logins>>,
) {
    let Some(logins) = logins else {
        return;
    };
    let Connections { clients, accounts } = &mut *connections;
    for finished in logins.finished() {
        let waiting = clients.iter_mut().find(|client| {
            !client.gone
                && client.phase == Phase::LoggingIn
                && client.login_state.as_deref() == Some(finished.state.as_str())
        });
        // It left, or began another.
        let Some(client) = waiting else {
            continue;
        };
        client.login_state = None;
        let world = worlds.of(client.world);
        let stay = match finished.email {
            Ok(email) => {
                let key = LoginKey::Email(logins.email_key(&email));
                let made = accounts
                    .log_in(&key)
                    .and_then(|id| Ok((id, accounts.new_session(id)?)));
                match made {
                    Ok((id, session)) => {
                        let email = EmailAddress(email);
                        let how = format!("through {}", finished.provider);
                        client
                            .link
                            .send(&ServerMessage::LoggedIn { session, email })
                            && logged_in(client, id, &how, accounts, world, &options)
                    }
                    Err(error) => client.refuse_for_saves(&error),
                }
            }
            Err(reason) => {
                info!(
                    "A login through {} didn't work: {reason}",
                    finished.provider
                );
                client.link.send(&ServerMessage::LoginFailed(reason))
            }
        };
        if !stay {
            client.disconnect(world);
        }
    }
}
