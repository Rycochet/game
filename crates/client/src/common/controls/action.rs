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
    /// The next creature out from the player, or the nearest without a target.
    NextTarget,
    /// The next creature in towards the player.
    PreviousTarget,
    /// The Character panel on its Character tab.
    Character,
    /// The Character panel on its Inventory tab.
    Inventory,
    /// The Character panel on its Abilities tab.
    Abilities,
    /// The Skills page.
    Skills,
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
    Targeting,
    Interface,
}

impl Group {
    pub const ALL: [Self; 4] = [
        Self::Movement,
        Self::Camera,
        Self::Targeting,
        Self::Interface,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Movement => "Movement",
            Self::Camera => "Camera",
            Self::Targeting => "Targeting",
            Self::Interface => "Interface",
        }
    }
}

impl Action {
    pub const ALL: [Self; 17] = [
        Self::StepUp,
        Self::StepDown,
        Self::StepLeft,
        Self::StepRight,
        Self::Explore,
        Self::LeaveWilderness,
        Self::ResetView,
        Self::NextTarget,
        Self::PreviousTarget,
        Self::Character,
        Self::Inventory,
        Self::Abilities,
        Self::Skills,
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
            Self::NextTarget => "Next Target",
            Self::PreviousTarget => "Previous Target",
            Self::Character => "Character",
            Self::Inventory => "Inventory",
            Self::Abilities => "Abilities",
            Self::Skills => "Skills",
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
            Self::NextTarget => "next_target",
            Self::PreviousTarget => "previous_target",
            Self::Character => "character",
            Self::Inventory => "inventory",
            Self::Abilities => "abilities",
            Self::Skills => "skills",
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
            Self::NextTarget | Self::PreviousTarget => Group::Targeting,
            Self::Character | Self::Inventory | Self::Abilities | Self::Skills => Group::Interface,
            Self::FullMap | Self::TurnItem | Self::DebugMenu => Group::Interface,
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
            Self::NextTarget => [key(KeyCode::Tab), None],
            Self::PreviousTarget => [shift(KeyCode::Tab), None],
            Self::Character => [key(KeyCode::KeyC), None],
            Self::Inventory => [key(KeyCode::KeyI), None],
            Self::Abilities => [key(KeyCode::KeyB), None],
            Self::Skills => [key(KeyCode::KeyK), None],
            Self::FullMap => [key(KeyCode::KeyM), None],
            Self::TurnItem => [key(KeyCode::KeyR), None],
            Self::DebugMenu => [key(KeyCode::Backquote), None],
            Self::FullScreen => [alt(KeyCode::Enter), alt(KeyCode::NumpadEnter)],
        }
    }
}
