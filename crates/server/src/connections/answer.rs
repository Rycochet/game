//! Answering a client's message: greeting, logging in, characters and moving about; items and dev tools go to `answer_items` and `answer_dev`.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, LoginMethod, PROTOCOL_VERSION, ServerMessage};

use super::answer_dev::answer_dev;
use super::answer_items::answer_items;
use super::{Action, Client, MOST_WAITING, Phase};
use crate::accounts::{Accounts, LoginKey, MAX_CHARACTERS};
use crate::{ServerOptions, ServerWorld};

/// A client has logged in: a guest goes straight to their one character, anyone
/// else is told theirs to choose from.
pub(super) fn logged_in(client: &mut Client, key: &LoginKey, accounts: &mut Accounts) -> bool {
    let answered = (|| {
        let id = accounts.log_in(key)?;
        let account = accounts.account(&id)?;
        let answered = if account.guest {
            client.character_id = Some(accounts.guest_character(&id)?);
            client.phase = Phase::Joining;
            true
        } else {
            client.phase = Phase::Choosing;
            client.link.send(&ServerMessage::Characters {
                list: account.characters.clone(),
                can_create: account.characters.len() < MAX_CHARACTERS,
            })
        };
        client.account = Some(id);
        Ok::<bool, String>(answered)
    })();
    answered.unwrap_or_else(|error| client.refuse_for_saves(&error))
}

/// Answers one message; `false` if the client should be disconnected.
pub(crate) fn answer(
    client: &mut Client,
    message: ClientMessage,
    world: &mut ServerWorld,
    accounts: &mut Accounts,
    options: &ServerOptions,
) -> bool {
    let refuse = |client: &Client, reason: String| {
        client.link.send(&ServerMessage::Refused(reason));
        false
    };
    match message {
        ClientMessage::Hello { protocol, .. } if protocol != PROTOCOL_VERSION => refuse(
            client,
            format!(
                "this server speaks protocol {PROTOCOL_VERSION}, not {protocol}: \
                 the game needs updating (reload the page)"
            ),
        ),
        ClientMessage::Hello { .. } if client.phase != Phase::Greeting => {
            refuse(client, "hello again?".into())
        }
        // Needs no login: welcomed by `handle_messages`, which can see other
        // clients.
        ClientMessage::Hello { .. } if client.character_id.is_some() => {
            client.phase = Phase::Joining;
            true
        }
        ClientMessage::Hello { .. } => {
            client.phase = Phase::LoggingIn;
            let methods = if options.guests {
                vec![LoginMethod::Guest]
            } else {
                Vec::new()
            };
            client.link.send(&ServerMessage::LoginOptions { methods })
        }
        ClientMessage::Login(_) if client.phase != Phase::LoggingIn => {
            refuse(client, "not the time to log in".into())
        }
        ClientMessage::Login(credentials) => {
            let key = LoginKey::from(&credentials);
            if matches!(key, LoginKey::Guest(_)) && !options.guests {
                return refuse(client, "this server doesn't take guests".into());
            }
            logged_in(client, &key, accounts)
        }
        ClientMessage::ChooseCharacter { id } => {
            let Some(account) = client.account.filter(|_| client.phase == Phase::Choosing) else {
                return refuse(client, "not the time to choose a character".into());
            };
            // A client can ask for anything: only its own will do.
            let characters = match accounts.account(&account) {
                Ok(account) => &account.characters,
                Err(error) => return client.refuse_for_saves(&error),
            };
            if !characters.iter().any(|character| character.id == id) {
                return refuse(client, "that isn't one of this account's characters".into());
            }
            client.character_id = Some(id);
            client.phase = Phase::Joining;
            true
        }
        ClientMessage::CreateCharacter {
            name,
            kind,
            appearance,
        } => {
            let Some(account) = client.account.filter(|_| client.phase == Phase::Choosing) else {
                return refuse(client, "not the time to make a character".into());
            };
            // A client can ask for anything: the server decides what may be played.
            if !kind.def().playable {
                return refuse(client, format!("{} can't be played", kind.def().name));
            }
            if !appearance.suits(kind) {
                return refuse(
                    client,
                    "that look doesn't suit that kind of creature".into(),
                );
            }
            match accounts.create_character(&account, &name) {
                Ok(id) => {
                    client.character_id = Some(id);
                    client.starting_as = Some((kind, appearance));
                    client.phase = Phase::Joining;
                    true
                }
                Err(reason) => refuse(client, reason),
            }
        }
        _ if client.phase != Phase::Playing => refuse(client, "say hello first".into()),
        ClientMessage::Ping { sent } => client.link.send(&ServerMessage::Pong {
            sent,
            clock: world.clock.minutes(),
        }),
        // Meant for a map the player has since left.
        ClientMessage::Step { map, .. } if map != client.map => true,
        ClientMessage::Step { number, step, .. } => {
            if client.waiting.len() < MOST_WAITING {
                client.waiting.push_back(Action::Step { number, step });
            } else {
                // Its `Position` puts the client right.
                client.last_step = Some(number);
            }
            true
        }
        ClientMessage::EnterSite { chunk } => {
            client.waiting.push_back(Action::EnterSite { chunk });
            true
        }
        ClientMessage::UseExit => {
            client.waiting.push_back(Action::UseExit);
            true
        }
        ClientMessage::OpenTrade { with } => {
            client.waiting.push_back(Action::OpenTrade { with });
            true
        }
        ClientMessage::SetOnline(_) if !client.host => {
            warn!("A client asked to open the server to others, which only its host may");
            true
        }
        ClientMessage::SetOnline(settings) => {
            // For `network::online`, which can reach the ports.
            client.asked_online = Some(settings);
            true
        }
        ClientMessage::SetPaused(_) if !options.pausable => {
            warn!("A client asked to pause, which this server doesn't allow");
            true
        }
        ClientMessage::SetPaused(paused) => {
            if paused {
                world.clock.pause();
            } else {
                world.clock.resume();
            }
            client.link.send(&ServerMessage::Paused {
                paused,
                clock: world.clock.minutes(),
            })
        }
        ClientMessage::DropItem { .. }
        | ClientMessage::TakeFromGround { .. }
        | ClientMessage::WearFromGround { .. }
        | ClientMessage::BinFromGround { .. }
        | ClientMessage::MoveItem { .. }
        | ClientMessage::GiveItem { .. }
        | ClientMessage::TakeItem { .. }
        | ClientMessage::WearItem { .. }
        | ClientMessage::BinItem { .. } => answer_items(client, message, world),
        // Dev tools are only for the embedded server; elsewhere a stray key
        // or click is ignored rather than ending the game.
        _ if message.is_dev_tool() && !options.dev_tools_for(client.host) => {
            warn!("A client asked for dev tools ({message:?}), which this server doesn't allow");
            true
        }
        ClientMessage::DevTeleport { .. }
        | ClientMessage::DevBackToStart
        | ClientMessage::DevToShowcase
        | ClientMessage::DevEnterSite { .. }
        | ClientMessage::DevGenerateSite { .. }
        | ClientMessage::DevSetCell { .. }
        | ClientMessage::DevRevealPois
        | ClientMessage::DevRevealMap
        | ClientMessage::DevForgetMap
        | ClientMessage::DevForgetMapOnServer
        | ClientMessage::DevGiveItem { .. }
        | ClientMessage::DevDetectMonsters(..)
        | ClientMessage::DevTruesight(..)
        | ClientMessage::DevGodMode(..)
        | ClientMessage::DevKillMob { .. }
        | ClientMessage::DevSetCreature(..)
        | ClientMessage::DevSnowLine(..)
        | ClientMessage::DevSetTime { .. } => answer_dev(client, message, world),
    }
}
