//! What is on screen: the view, the loading step, the click mode, the server's
//! status, and whether the player has chosen to play.

use bevy::prelude::*;

/// What the loading screen is waiting for (set by `loading_screen`). New
/// startup work gets a step here.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoadingStep {
    /// The screen's first frames: `connection` waits for it to be on show.
    #[default]
    Starting,
    /// For the server's welcome (again, if the connection was lost).
    Connecting,
    /// The game can take input: the screen is gone.
    Ready,
}

/// What a click on the map does (dev tools), chosen in the debug menu.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickMode {
    /// Travels there.
    #[default]
    Normal,
    /// Asks the server to put the player there, even through walls.
    Teleport,
    /// Changes the cell there, to try walls out (in a site).
    ChangeCell,
    /// Puts a mob made to order (`CreateMobOrder`) on the cell (in a site).
    PlaceMob,
}

/// Whether the client is connected to the server (set by `connection`).
/// While it isn't, the player can't move, and the last view stays on show.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub enum ServerStatus {
    /// Trying to connect (or to connect again after losing the connection).
    #[default]
    Connecting,
    /// Welcomed by the server.
    Connected,
    /// Not connecting any more (the server won't have this client, or this
    /// client won't trust the server), and the words saying why.
    Refused(String),
}

impl ServerStatus {
    pub fn is_connected(&self) -> bool {
        *self == Self::Connected
    }
}

/// Which view is on screen: the world's surface, or a dungeon or building.
/// Each view shows its scene only while it is the current one.
#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameView {
    /// Waiting for the server to say which world this is: nothing to show
    /// yet, and no `CurrentWorld`. Until the first `Welcome`, and again after
    /// logging out: losing the connection keeps the view (`ServerStatus` says).
    #[default]
    Connecting,
    WorldMap,
    /// Inside a site (a building or cave): `CurrentDungeon`.
    Dungeon,
}

/// Whether the player has chosen to play. Until then, and again after logging
/// out, the system menu is the startup menu and nothing connects.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Session {
    /// The startup menu, over the title.
    #[default]
    AtStart,
    /// Play was pressed: connecting, or connected.
    Playing,
}

impl Session {
    pub fn is_playing(self) -> bool {
        self == Self::Playing
    }
}
