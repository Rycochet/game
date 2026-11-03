//! What is open, what each press does, and the menu's parts.

use bevy::prelude::*;

/// What is open.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemMenu {
    #[default]
    Closed,
    Menu,
    /// The menu, and above it the requester asking whether to quit.
    Confirming,
    /// The menu, and above it the settings.
    Settings,
    /// The menu, and above it the credits.
    Credits,
    /// The menu, and above it Connect (at the startup menu only).
    Connect,
    /// Connect, and above it the box to paste a server's certificate into.
    Certificate,
    /// Connect, and above it the requester asking whether to forget a server.
    Forgetting,
    /// The startup menu, and above it why the server can't be played on.
    Refused,
}

impl SystemMenu {
    pub fn is_open(self) -> bool {
        self != Self::Closed
    }

    /// Is Connect open, or under what is on top?
    pub(super) fn in_connect(self) -> bool {
        matches!(self, Self::Connect | Self::Certificate | Self::Forgetting)
    }

    /// What is open after this.
    pub(super) fn after(self, action: Action) -> Self {
        match (self, action) {
            (Self::Closed, Action::Open) => Self::Menu,
            (Self::Menu, Action::Back) => Self::Closed,
            (Self::Menu, Action::Settings) => Self::Settings,
            (Self::Menu, Action::Credits) => Self::Credits,
            (Self::Menu, Action::Quit) => Self::Confirming,
            // (Connect's own presses are `connect::use_connect`'s.)
            (Self::Menu, Action::Connect) => Self::Connect,
            (Self::Confirming | Self::Settings | Self::Credits | Self::Connect, Action::Back) => {
                Self::Menu
            }
            (Self::Certificate | Self::Forgetting, Action::Back) => Self::Connect,
            (same, _) => same,
        }
    }
}

/// What a press on this does. Each part can only be pressed while it is on top.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Open,
    /// Closes what is on top: its "No" or close button, or outside its panel.
    /// Continue, on the menu.
    Back,
    /// From the startup menu: connects, and closes the menu.
    Play,
    Settings,
    Credits,
    /// Leaves the server for the startup menu.
    LogOut,
    /// Opens Connect (at the startup menu).
    Connect,
    /// Connect's own Connect: to the server in its box.
    ConnectNow,
    /// Puts the remembered server at this place in the list in the box.
    PickServer(usize),
    /// Asks whether to forget the remembered server at this place in the list.
    ForgetServer(usize),
    ReallyForget,
    /// Opens the box for the server's certificate.
    Certificate,
    /// Keeps the certificate pasted in, for the server in Connect's box.
    KeepCertificate,
    /// Takes the certificate away from the server in Connect's box.
    ClearCertificate,
    Quit,
    ReallyQuit,
}

/// A menu button shown only some of the time (`show_buttons`).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MenuButton {
    /// At the startup menu.
    Play,
    /// While playing.
    Continue,
    /// While playing on another computer's server (not the one built in).
    LogOut,
    /// Enabled only at the startup menu.
    Connect,
}

/// The gear button.
#[derive(Component)]
pub(super) struct GearButton;

/// Which of the layers: shown while `SystemMenu` is one of these.
#[derive(Component, Clone, Copy)]
pub(super) struct Layer(pub(super) &'static [SystemMenu]);

#[derive(Component)]
pub(super) struct PausedText;
