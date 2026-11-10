//! Logging in to an account on a server: what `connection` asks the player,
//! for `login`'s screen to show, what they choose there, and the account's
//! email address for the menus.

use bevy::prelude::*;

/// A provider a server logs in with (`LoginMethod::Provider`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginProvider {
    /// What the server calls it.
    pub name: String,
    /// What the player sees it called, such as "Google".
    pub label: String,
}

/// Where logging in has got to. Set by `connection`; `login` shows it.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub enum LoggingIn {
    /// Nothing to ask the player.
    #[default]
    Not,
    /// The player chooses how to log in: with a provider, or as a guest if
    /// `guests`. `failed` says why the last try didn't work.
    Choosing {
        providers: Vec<LoginProvider>,
        guests: bool,
        failed: Option<String>,
    },
    /// Waiting while the player logs in on the provider's page, in their
    /// browser: `url` once the server has sent it.
    InBrowser {
        provider: LoginProvider,
        url: Option<String>,
    },
}

impl LoggingIn {
    /// Whether there is something to show.
    pub fn shows(&self) -> bool {
        *self != Self::Not
    }
}

/// What the player chose on the login screen, Character Select or the menus.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub enum LoginChoice {
    /// Log in on this provider's page (by its name).
    Provider(String),
    Guest,
    /// From waiting on the browser back to choosing.
    Back,
    /// Not now: back to the startup menu.
    Cancel,
    /// Forget the account kept for this server, to log in another way next
    /// time.
    LogOut,
    /// Character Select's Change User: forget the account kept for the server
    /// being played on, and connect again to log in another way.
    ChangeUser,
    /// Connect's bin: forget the account kept for the server at this address
    /// (as Connect shows it).
    ForgetServer(String),
}

/// The email address of the account kept for the server Play goes to, for
/// the startup menu and making a character to show. Never shown in game, and
/// its `Debug` hides it.
#[derive(Resource, Clone, Default, PartialEq, Eq)]
pub struct AccountEmail(pub Option<String>);

impl std::fmt::Debug for AccountEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AccountEmail(hidden)")
    }
}
