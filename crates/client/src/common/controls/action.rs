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
    /// Up the stairs underfoot, or out of the wilderness, or a site from
    /// outside ground, to the world map.
    Up,
    /// Down the stairs underfoot, or into the wilderness from the world map.
    Down,
    /// Close the open doors beside the player (until the FAB menu has a way).
    CloseDoors,
    /// The camera swings round the player to the left, by a step or while held.
    TurnLeft,
    TurnRight,
    ZoomIn,
    ZoomOut,
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
    /// Use the action bar's slot (0 to 9, keys 1 to 0).
    UseSlot(u8),
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
    ActionBar,
    Interface,
}

impl Group {
    pub const ALL: [Self; 5] = [
        Self::Movement,
        Self::Camera,
        Self::Targeting,
        Self::ActionBar,
        Self::Interface,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Movement => "Movement",
            Self::Camera => "Camera",
            Self::Targeting => "Targeting",
            Self::ActionBar => "Action Bar",
            Self::Interface => "Interface",
        }
    }
}

impl Action {
    pub const ALL: [Self; 32] = [
        Self::StepUp,
        Self::StepDown,
        Self::StepLeft,
        Self::StepRight,
        Self::Up,
        Self::Down,
        Self::CloseDoors,
        Self::TurnLeft,
        Self::TurnRight,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::ResetView,
        Self::NextTarget,
        Self::PreviousTarget,
        Self::UseSlot(0),
        Self::UseSlot(1),
        Self::UseSlot(2),
        Self::UseSlot(3),
        Self::UseSlot(4),
        Self::UseSlot(5),
        Self::UseSlot(6),
        Self::UseSlot(7),
        Self::UseSlot(8),
        Self::UseSlot(9),
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
            Self::Up => "Up / Leave",
            Self::Down => "Down / Explore",
            Self::CloseDoors => "Close Doors",
            Self::TurnLeft => "Turn Camera Left",
            Self::TurnRight => "Turn Camera Right",
            Self::ZoomIn => "Zoom In",
            Self::ZoomOut => "Zoom Out",
            Self::ResetView => "Reset View",
            Self::NextTarget => "Next Target",
            Self::PreviousTarget => "Previous Target",
            Self::Character => "Character",
            Self::Inventory => "Inventory",
            Self::Abilities => "Abilities",
            Self::Skills => "Skills",
            Self::UseSlot(slot) => BAR_NAMES[slot as usize % BAR_NAMES.len()],
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
            Self::Up => "up",
            Self::Down => "down",
            Self::CloseDoors => "close_doors",
            Self::TurnLeft => "turn_camera_left",
            Self::TurnRight => "turn_camera_right",
            Self::ZoomIn => "zoom_in",
            Self::ZoomOut => "zoom_out",
            Self::ResetView => "reset_view",
            Self::NextTarget => "next_target",
            Self::PreviousTarget => "previous_target",
            Self::Character => "character",
            Self::Inventory => "inventory",
            Self::Abilities => "abilities",
            Self::Skills => "skills",
            Self::UseSlot(slot) => BAR_IDS[slot as usize % BAR_IDS.len()],
            Self::FullMap => "full_map",
            Self::TurnItem => "turn_item",
            Self::DebugMenu => "debug_menu",
            Self::FullScreen => "full_screen",
        }
    }

    /// Its place in `ALL`, which key choices are kept by.
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&each| each == self)
            .expect("every action is in ALL")
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }

    pub fn group(self) -> Group {
        match self {
            Self::StepUp | Self::StepDown | Self::StepLeft | Self::StepRight => Group::Movement,
            Self::Up | Self::Down | Self::CloseDoors => Group::Movement,
            Self::TurnLeft | Self::TurnRight | Self::ZoomIn | Self::ZoomOut => Group::Camera,
            Self::ResetView => Group::Camera,
            Self::NextTarget | Self::PreviousTarget => Group::Targeting,
            Self::UseSlot(_) => Group::ActionBar,
            Self::Character | Self::Inventory | Self::Abilities | Self::Skills => Group::Interface,
            Self::FullMap | Self::TurnItem | Self::DebugMenu => Group::Interface,
            Self::FullScreen => Group::Interface,
        }
    }

    /// The game's own keys for it. The number pad's are always in the second
    /// column.
    pub fn defaults(self) -> [Option<Binding>; SLOTS] {
        let key = |key| Some(Binding::new(key, Mods::NONE));
        let alt = |key| Some(Binding::new(key, Mods::ALT));
        // `>` and `<` on UK and US keyboards.
        let shift = |key| Some(Binding::new(key, Mods::SHIFT));
        match self {
            Self::StepUp => [key(KeyCode::KeyW), key(KeyCode::Numpad8)],
            Self::StepDown => [key(KeyCode::KeyS), key(KeyCode::Numpad2)],
            Self::StepLeft => [key(KeyCode::KeyA), key(KeyCode::Numpad4)],
            Self::StepRight => [key(KeyCode::KeyD), key(KeyCode::Numpad6)],
            Self::Up => [shift(KeyCode::Comma), key(KeyCode::NumpadDecimal)],
            Self::Down => [shift(KeyCode::Period), key(KeyCode::Numpad0)],
            Self::CloseDoors => [key(KeyCode::KeyL), None],
            Self::TurnLeft => [None, key(KeyCode::Numpad7)],
            Self::TurnRight => [None, key(KeyCode::Numpad9)],
            // As the Skills page zooms.
            Self::ZoomIn => [key(KeyCode::Equal), key(KeyCode::NumpadAdd)],
            Self::ZoomOut => [key(KeyCode::Minus), key(KeyCode::NumpadSubtract)],
            Self::ResetView => [key(KeyCode::Home), key(KeyCode::Numpad5)],
            Self::NextTarget => [key(KeyCode::Tab), None],
            Self::PreviousTarget => [shift(KeyCode::Tab), None],
            Self::Character => [key(KeyCode::KeyC), None],
            Self::Inventory => [key(KeyCode::KeyI), None],
            Self::Abilities => [key(KeyCode::KeyB), None],
            Self::Skills => [key(KeyCode::KeyK), None],
            Self::UseSlot(slot) => [key(BAR_KEYS[slot as usize % BAR_KEYS.len()]), None],
            Self::FullMap => [key(KeyCode::KeyM), None],
            Self::TurnItem => [key(KeyCode::KeyR), None],
            Self::DebugMenu => [key(KeyCode::Backquote), None],
            Self::FullScreen => [alt(KeyCode::Enter), alt(KeyCode::NumpadEnter)],
        }
    }
}

/// The action bar's slots, as Settings shows them.
const BAR_NAMES: [&str; 10] = [
    "Action 1",
    "Action 2",
    "Action 3",
    "Action 4",
    "Action 5",
    "Action 6",
    "Action 7",
    "Action 8",
    "Action 9",
    "Action 10",
];
const BAR_IDS: [&str; 10] = [
    "action_1",
    "action_2",
    "action_3",
    "action_4",
    "action_5",
    "action_6",
    "action_7",
    "action_8",
    "action_9",
    "action_10",
];
/// The keys along the top of the keyboard, 1 to 0.
const BAR_KEYS: [KeyCode; 10] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::Digit0,
];
