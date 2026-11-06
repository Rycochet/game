//! What keys can be bound to, in the groups Settings shows them in.

use bevy::prelude::*;

use super::{Binding, Mods, SLOTS};

/// Something done with a key. The order is Settings' order within each group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    StepUp,
    StepDown,
    StepLeft,
    StepRight,
    /// Into the wilderness from the world map.
    Explore,
    /// Out of the wilderness to the world map.
    LeaveWilderness,
    ResetView,
    Character,
    FullMap,
    TurnItem,
    DebugMenu,
    /// Native only: a browser decides its own full screen.
    FullScreen,
}

/// The headings actions are shown under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Movement,
    Camera,
    Interface,
}

impl Group {
    pub const ALL: [Self; 3] = [Self::Movement, Self::Camera, Self::Interface];

    pub fn name(self) -> &'static str {
        match self {
            Self::Movement => "Movement",
            Self::Camera => "Camera",
            Self::Interface => "Interface",
        }
    }
}

impl Action {
    pub const ALL: [Self; 12] = [
        Self::StepUp,
        Self::StepDown,
        Self::StepLeft,
        Self::StepRight,
        Self::Explore,
        Self::LeaveWilderness,
        Self::ResetView,
        Self::Character,
        Self::FullMap,
        Self::TurnItem,
        Self::DebugMenu,
        Self::FullScreen,
    ];

    /// As Settings shows it.
    pub fn name(self) -> &'static str {
        match self {
            Self::StepUp => "Step Up",
            Self::StepDown => "Step Down",
            Self::StepLeft => "Step Left",
            Self::StepRight => "Step Right",
            Self::Explore => "Explore Wilderness",
            Self::LeaveWilderness => "Leave Wilderness",
            Self::ResetView => "Reset View",
            Self::Character => "Character and Inventory",
            Self::FullMap => "Full Map",
            Self::TurnItem => "Turn Held Item",
            Self::DebugMenu => "Debug Menu",
            Self::FullScreen => "Full Screen",
        }
    }

    /// As it is saved and named in widget ids.
    pub fn id(self) -> &'static str {
        match self {
            Self::StepUp => "step_up",
            Self::StepDown => "step_down",
            Self::StepLeft => "step_left",
            Self::StepRight => "step_right",
            Self::Explore => "explore",
            Self::LeaveWilderness => "leave_wilderness",
            Self::ResetView => "reset_view",
            Self::Character => "character",
            Self::FullMap => "full_map",
            Self::TurnItem => "turn_item",
            Self::DebugMenu => "debug_menu",
            Self::FullScreen => "full_screen",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }

    pub fn group(self) -> Group {
        match self {
            Self::StepUp | Self::StepDown | Self::StepLeft | Self::StepRight => Group::Movement,
            Self::Explore | Self::LeaveWilderness => Group::Movement,
            Self::ResetView => Group::Camera,
            Self::Character | Self::FullMap | Self::TurnItem | Self::DebugMenu => Group::Interface,
            Self::FullScreen => Group::Interface,
        }
    }

    /// The game's own keys for it.
    pub fn defaults(self) -> [Option<Binding>; SLOTS] {
        let key = |key| Some(Binding::new(key, Mods::NONE));
        let alt = |key| Some(Binding::new(key, Mods::ALT));
        // `>` and `<` on UK and US keyboards.
        let shift = |key| Some(Binding::new(key, Mods::SHIFT));
        match self {
            Self::StepUp => [key(KeyCode::ArrowUp), key(KeyCode::KeyW)],
            Self::StepDown => [key(KeyCode::ArrowDown), key(KeyCode::KeyS)],
            Self::StepLeft => [key(KeyCode::ArrowLeft), key(KeyCode::KeyA)],
            Self::StepRight => [key(KeyCode::ArrowRight), key(KeyCode::KeyD)],
            Self::Explore => [shift(KeyCode::Period), None],
            Self::LeaveWilderness => [shift(KeyCode::Comma), None],
            Self::ResetView => [key(KeyCode::Home), None],
            Self::Character => [key(KeyCode::KeyC), key(KeyCode::KeyI)],
            Self::FullMap => [key(KeyCode::KeyM), None],
            Self::TurnItem => [key(KeyCode::KeyR), None],
            Self::DebugMenu => [key(KeyCode::Backquote), None],
            Self::FullScreen => [alt(KeyCode::Enter), alt(KeyCode::NumpadEnter)],
        }
    }
}
