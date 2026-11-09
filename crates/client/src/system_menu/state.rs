//! What is open, what each press does, and the menu's parts.

use bevy::prelude::*;

use crate::widgets::UiId;

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
    /// Settings, and above it the requester asking whether to keep the video
    /// settings just applied.
    KeepVideo,
    /// Settings, and above it the requester asking whether to make Ironman the
    /// player's own difficulty.
    Ironman,
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
    /// The menu, and above it Server Options (native only).
    ServerOptions,
    /// The menu, and above it the requester asking whether to let others join.
    GoingOnline,
    /// The menu, and above it the requester asking whether to send others away.
    GoingOffline,
    /// The menu, and above it why the server couldn't be opened to others.
    OnlineFailed,
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
            (Self::Menu, Action::ServerOptions) => Self::ServerOptions,
            (Self::Settings, Action::ApplyVideo) => Self::KeepVideo,
            (Self::KeepVideo, Action::KeepVideo | Action::Back) => Self::Settings,
            (Self::Ironman, Action::ChooseIronman | Action::Back) => Self::Settings,
            (
                Self::Confirming
                | Self::Settings
                | Self::Credits
                | Self::Connect
                | Self::ServerOptions
                | Self::GoingOnline
                | Self::GoingOffline
                | Self::OnlineFailed,
                Action::Back,
            ) => Self::Menu,
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
    /// From the startup menu: connects to choose a character.
    Characters,
    Settings,
    /// The Video page's Apply.
    ApplyVideo,
    /// Keeps the video settings just applied (going back is `Back`).
    KeepVideo,
    /// The requester's yes to Ironman as the player's own difficulty.
    ChooseIronman,
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
    /// "Online": opens the built-in server to others, or closes it, asking
    /// first while playing on it.
    ToggleOnline,
    /// The requester's yes, to letting others join or sending them away.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    ReallyToggleOnline,
    /// The gear beside Online.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    ServerOptions,
    /// Copies the server's certificate.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    CopyCertificate,
    /// Copies the address the router says players on the internet connect to.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    CopyRouterAddress,
    Quit,
    ReallyQuit,
}

impl Action {
    /// Its button's id in `form`, for the debug API.
    pub(super) fn ui_id(self, form: &'static str) -> UiId {
        let name = match self {
            Self::Open => "menu",
            Self::Back => "back",
            Self::Play => "play",
            Self::Characters => "characters",
            Self::Settings => "settings",
            Self::ApplyVideo => "apply",
            Self::KeepVideo | Self::KeepCertificate => "keep",
            Self::ChooseIronman => "ironman",
            Self::Credits => "credits",
            Self::LogOut => "log_out",
            Self::Connect | Self::ConnectNow => "connect",
            Self::PickServer(_) => "server",
            Self::ForgetServer(_) | Self::ReallyForget => "forget",
            Self::Certificate => "certificate",
            Self::ClearCertificate => "clear",
            Self::ToggleOnline => "online",
            Self::ReallyToggleOnline => "yes",
            Self::ServerOptions => "server_options",
            Self::CopyCertificate => "copy",
            Self::CopyRouterAddress => "copy_address",
            Self::Quit | Self::ReallyQuit => "quit",
        };
        let id = UiId::new(form, name);
        match self {
            Self::PickServer(at) | Self::ForgetServer(at) => id.at(at),
            _ => id,
        }
    }
}

/// A menu button shown only some of the time (`show_buttons`).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MenuButton {
    /// At the startup menu.
    Play,
    /// At the startup menu: the screen for choosing.
    Characters,
    /// While playing.
    Continue,
    /// While playing on another computer's server (not the one built in).
    LogOut,
    /// Enabled only at the startup menu.
    Connect,
    /// Native only: shows whether it is on.
    Online,
}

/// The gear button.
#[derive(Component)]
pub(super) struct GearButton;

/// Which of the layers: shown while `SystemMenu` is one of these.
#[derive(Component, Clone, Copy)]
pub(super) struct Layer(pub(super) &'static [SystemMenu]);

#[derive(Component)]
pub(super) struct PausedText;
