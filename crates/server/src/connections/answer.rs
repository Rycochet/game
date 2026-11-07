//! Answering a client's message: greeting, logging in, characters and moving about; items and dev tools go to `answer_items` and `answer_dev`.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, LoginMethod, PROTOCOL_VERSION, ServerMessage};

use super::answer_dev::answer_dev;
use super::answer_items::answer_items;
use super::delete_character::delete_character;
use super::new_character::{Asked, create, join_or_make};
use super::{Action, Client, MOST_WAITING, Phase};
use crate::accounts::{Accounts, LoginKey, MAX_CHARACTERS};
use crate::{ServerOptions, ServerWorld};

/// A client has logged in: a guest goes to their one character (made first, if
/// new: `join_or_make`), anyone else is told theirs to choose from.
pub(super) fn logged_in(
    client: &mut Client,
    key: &LoginKey,
    accounts: &mut Accounts,
    world: &ServerWorld,
) -> bool {
    let answered = (|| {
        let id = accounts.log_in(key)?;
        let account = accounts.account(&id)?;
        client.account = Some(id);
        let answered = if account.guest {
            client.character_id = Some(accounts.guest_character(&id)?);
            join_or_make(client, world)
        } else {
            client.phase = Phase::Choosing;
            client.link.send(&ServerMessage::Characters {
                list: account.characters.clone(),
                can_create: account.characters.len() < MAX_CHARACTERS,
            })
        };
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
        // clients, once made if new.
        ClientMessage::Hello {
            makes_characters, ..
        } if client.character_id.is_some() => {
            client.makes_characters = makes_characters;
            join_or_make(client, world)
        }
        ClientMessage::Hello {
            makes_characters, ..
        } => {
            client.makes_characters = makes_characters;
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
            logged_in(client, &key, accounts, world)
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
            build,
        } => {
            let asked = Asked {
                name,
                kind,
                appearance,
                build,
            };
            match create(client, asked, accounts) {
                Ok(()) => true,
                Err(reason) => refuse(client, reason),
            }
        }
        // Making a new one after deleting the last (dev tools): steps and the
        // like sent before may still be on their way.
        _ if client.phase == Phase::Choosing && client.character_id.is_some() => true,
        _ if client.phase != Phase::Playing => refuse(client, "say hello first".into()),
        ClientMessage::Ping { sent } => client.link.send(&ServerMessage::Pong {
            sent,
            clock: world.server_clock.seconds(),
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
        ClientMessage::EnterGate { entrance } => {
            client.waiting.push_back(Action::EnterGate { entrance });
            true
        }
        ClientMessage::UseExit => {
            client.waiting.push_back(Action::UseExit);
            true
        }
        ClientMessage::Explore => {
            client.waiting.push_back(Action::Explore);
            true
        }
        ClientMessage::LeaveWilderness => {
            client.waiting.push_back(Action::LeaveWilderness);
            true
        }
        ClientMessage::OpenTrade { with } => {
            client.waiting.push_back(Action::OpenTrade { with });
            true
        }
        ClientMessage::LookUpItems(ids) => {
            client.ask_after(ids);
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
            // The server's clock, so every world stops.
            if paused {
                world.server_clock.pause();
            } else {
                world.server_clock.resume();
            }
            client.link.send(&ServerMessage::Paused {
                paused,
                clock: world.server_clock.seconds(),
            })
        }
        ClientMessage::ActionBar(change) => client.change_bar(change),
        ClientMessage::UseAbility { ability, target } => {
            // One at a time: an automatic one asks again once it can.
            if !client
                .waiting
                .iter()
                .any(|a| matches!(a, Action::UseAbility { .. }))
            {
                client
                    .waiting
                    .push_back(Action::UseAbility { ability, target });
            }
            true
        }
        ClientMessage::CancelAbility => super::combat::cancel_ability(client, world),
        ClientMessage::WithdrawAbility => {
            client
                .waiting
                .retain(|action| !matches!(action, Action::UseAbility { .. }));
            true
        }
        ClientMessage::DropItem { .. }
        | ClientMessage::TakeFromGround { .. }
        | ClientMessage::WearFromGround { .. }
        | ClientMessage::BinFromGround { .. }
        | ClientMessage::MoveItem { .. }
        | ClientMessage::StackItem { .. }
        | ClientMessage::SortGrid { .. }
        | ClientMessage::FromStack { .. }
        | ClientMessage::StackFromGround { .. }
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
        | ClientMessage::DevToHall
        | ClientMessage::DevEnterSite { .. }
        | ClientMessage::DevGenerateSite { .. }
        | ClientMessage::DevGenerateArea(_)
        | ClientMessage::DevCreateMob(_)
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
        | ClientMessage::DevSetCreature { .. }
        | ClientMessage::DevSetWorldClock { .. }
        | ClientMessage::DevGoToWorld { .. }
        | ClientMessage::DevVitals(_)
        | ClientMessage::DevSkills(_) => answer_dev(client, message, world),
        ClientMessage::DevDeleteCharacter => delete_character(client, world, accounts),
    }
}
