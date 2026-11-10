//! Logging in: with the account kept for this server, on a provider's page in
//! the player's browser, or as a guest.
//! `receive` hands the server's login messages on as `FromServer`, answered by
//! `answer_login`; what the player chooses on `login`'s screen comes as
//! `LoginChoice` (`choose_login`). Characters are `characters`' to answer.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, Credentials, LoginMethod, ServerMessage};

use super::saved_login::{self, SavedLogin};
use super::socket::Target;
use super::{Connecting, FromServer, ServerConnection, give_up};
use crate::common::{
    AccountEmail, LoggingIn, LoginChoice, LoginProvider, ServerStatus, Session, open_in_browser,
};

/// Answers the server's login messages.
pub(super) fn answer_login(
    mut commands: Commands,
    mut from_server: MessageReader<FromServer>,
    connection: Res<ServerConnection>,
    mut connecting: ResMut<Connecting>,
    mut status: ResMut<ServerStatus>,
    mut logging_in: ResMut<LoggingIn>,
    mut email: ResMut<AccountEmail>,
) {
    for FromServer(message) in from_server.read() {
        let done = match message {
            ServerMessage::LoginOptions { methods } => {
                connecting.methods = methods.clone();
                log_in(&connection, &mut connecting, &mut logging_in)
            }
            ServerMessage::OpenLogin { url } => {
                if let Err(reason) = open_in_browser(url) {
                    warn!("Can't open the login page: {reason}");
                }
                if let LoggingIn::InBrowser { url: shown, .. } = &mut *logging_in {
                    *shown = Some(url.clone());
                }
                Ok(())
            }
            ServerMessage::LoggedIn {
                session,
                email: told,
            } => {
                let login = SavedLogin {
                    session: *session,
                    email: told.0.trim().to_owned(),
                };
                saved_login::keep(&connecting.target, &login);
                email.0 = Some(login.email);
                *logging_in = LoggingIn::Not;
                Ok(())
            }
            ServerMessage::LoginFailed(reason) => {
                // A kept session that no longer works is forgotten.
                if std::mem::take(&mut connecting.tried_session) {
                    saved_login::forget(&connecting.target);
                    email.0 = None;
                }
                *logging_in = choosing(&connecting, Some(reason.clone()));
                Ok(())
            }
            _ => Ok(()),
        };
        if let Err(reason) = done {
            give_up(&mut commands, &mut status, reason);
            return;
        }
    }
}

/// Logs in once `LoginOptions` has come: with the account kept for this
/// server, else by asking the player if it has providers, else as a guest.
fn log_in(
    connection: &ServerConnection,
    connecting: &mut Connecting,
    logging_in: &mut LoggingIn,
) -> Result<(), String> {
    connecting.tried_session = false;
    if providers(connecting).is_empty() {
        return guest(connection, connecting);
    }
    if let Some(saved) = saved_login::saved(&connecting.target) {
        connecting.tried_session = true;
        let credentials = Credentials::Session(saved.session);
        connection.link.send(&ClientMessage::Login(credentials));
        return Ok(());
    }
    *logging_in = choosing(connecting, None);
    Ok(())
}

/// Acts on what the player chose on the login screen or the menu's Logout.
#[allow(clippy::too_many_arguments)]
pub(super) fn choose_login(
    mut commands: Commands,
    mut choices: MessageReader<LoginChoice>,
    connection: Option<Res<ServerConnection>>,
    mut connecting: ResMut<Connecting>,
    mut status: ResMut<ServerStatus>,
    mut logging_in: ResMut<LoggingIn>,
    mut session: ResMut<Session>,
    mut email: ResMut<AccountEmail>,
    chosen: Res<crate::common::ChosenServer>,
) {
    for choice in choices.read() {
        match (choice, &connection) {
            // At the startup menu: forget the account kept for this
            // computer's own server (a browser's page's server).
            (LoginChoice::LogOut, _) => {
                saved_login::forget(&at_start(&chosen, &connecting));
                email.0 = None;
            }
            (LoginChoice::Cancel, _) => {
                *logging_in = LoggingIn::Not;
                *session = Session::AtStart;
            }
            (_, None) => {}
            (LoginChoice::Provider(name), Some(connection)) => {
                let providers = providers(&connecting);
                let Some(provider) = providers.into_iter().find(|p| p.name == *name) else {
                    continue;
                };
                let credentials = Credentials::Provider(name.clone());
                connection.link.send(&ClientMessage::Login(credentials));
                *logging_in = LoggingIn::InBrowser {
                    provider,
                    url: None,
                };
            }
            (LoginChoice::Guest, Some(connection)) => {
                *logging_in = LoggingIn::Not;
                if let Err(reason) = guest(connection, &mut connecting) {
                    give_up(&mut commands, &mut status, reason);
                }
            }
            (LoginChoice::Back, Some(_)) => *logging_in = choosing(&connecting, None),
        }
    }
}

/// The account kept for the server Play goes to (or is playing on), for the
/// menus to show: worked out again when that may have changed.
pub(super) fn show_account(
    chosen: Res<crate::common::ChosenServer>,
    connecting: Res<Connecting>,
    session: Res<Session>,
    mut email: ResMut<AccountEmail>,
) {
    if !chosen.is_changed() && !session.is_changed() {
        return;
    }
    let target = if session.is_playing() {
        connecting.target.clone()
    } else {
        at_start(&chosen, &connecting)
    };
    let kept = saved_login::saved(&target).map(|login| login.email);
    email.set_if_neq(AccountEmail(kept));
}

/// Where Play would connect.
fn at_start(chosen: &crate::common::ChosenServer, connecting: &Connecting) -> Target {
    Target::choose(chosen, connecting.environment)
}

/// The choice to show, from what the server offered.
fn choosing(connecting: &Connecting, failed: Option<String>) -> LoggingIn {
    LoggingIn::Choosing {
        providers: providers(connecting),
        guests: connecting.methods.contains(&LoginMethod::Guest),
        failed,
    }
}

/// The providers offered, if real logins can go to this server: only over a
/// secure connection, which a guest's token needn't have.
fn providers(connecting: &Connecting) -> Vec<LoginProvider> {
    let secure = matches!(&connecting.target, Target::Url { url, .. } if url.starts_with("wss://"));
    let offered = connecting.methods.iter().filter_map(|method| match method {
        LoginMethod::Provider { name, label } if secure => Some(LoginProvider {
            name: name.clone(),
            label: label.clone(),
        }),
        _ => None,
    });
    offered.collect()
}

/// Logs in as a guest. `Err` says why it can't.
fn guest(connection: &ServerConnection, connecting: &mut Connecting) -> Result<(), String> {
    if !connecting.methods.contains(&LoginMethod::Guest) {
        return Err("This server needs an account, which can only be logged in to securely".into());
    }
    let token = match connecting.guest {
        Some(token) => token,
        None => super::guest::token(&connecting.target)?,
    };
    connecting.guest = Some(token);
    connecting.as_guest = true;
    connection
        .link
        .send(&ClientMessage::Login(Credentials::Guest(token)));
    Ok(())
}
