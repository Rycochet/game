//! What the debug menu offers, and when each is possible. No Bevy systems here,
//! so it can be tested alone.

use shared::poi::PoiKind;

use crate::common::GameView;
use crate::widgets::matches_filter;

/// One entry in the debug menu. Sending it does what it says.
#[derive(bevy::prelude::Message, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DebugAction {
    AutoWalk,
    ChangeCellMode,
    ChangeCreature,
    DetectMonsters,
    FieldsState,
    ForgetMap,
    ForgetMapOnServer,
    GeneratePoi,
    GiveItem,
    GoToWorld,
    GodMode,
    HerbsState,
    ItemPictures,
    KillMob,
    ResetView,
    RevealMap,
    ShowDebugInfo,
    ShowWidgets,
    StairsStyle,
    SunsAndMoons,
    TeleportMode,
    TeleportToHall,
    TeleportToPoiShowcase,
    TeleportToStart,
    Truesight,
    ViewPlanet,
    WorldTime,
}

/// The state of the game that the menu depends on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Facts {
    pub view: GameView,
    pub has_stairs: bool,
    pub has_fields: bool,
    pub has_herbs: bool,
    pub walking: bool,
    pub detecting: bool,
    pub god_mode: bool,
    pub truesight: bool,
    pub killing: bool,
    pub teleporting: bool,
    pub changing_cells: bool,
    pub showing_info: bool,
    pub in_town: bool,
    pub world_time_open: bool,
    pub suns_and_moons_open: bool,
    /// Whether the server accepts developer tools messages.
    pub dev_tools: bool,
}

impl DebugAction {
    /// Every action, in the order the menu lists them: alphabetical by name.
    pub(super) const ALL: [Self; 27] = [
        Self::AutoWalk,
        Self::ChangeCellMode,
        Self::ChangeCreature,
        Self::DetectMonsters,
        Self::FieldsState,
        Self::ForgetMap,
        Self::ForgetMapOnServer,
        Self::GeneratePoi,
        Self::GiveItem,
        Self::GoToWorld,
        Self::GodMode,
        Self::HerbsState,
        Self::ItemPictures,
        Self::KillMob,
        Self::ResetView,
        Self::RevealMap,
        Self::ShowDebugInfo,
        Self::ShowWidgets,
        Self::StairsStyle,
        Self::SunsAndMoons,
        Self::TeleportMode,
        Self::TeleportToHall,
        Self::TeleportToPoiShowcase,
        Self::TeleportToStart,
        Self::Truesight,
        Self::ViewPlanet,
        Self::WorldTime,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::AutoWalk => "Auto Walk",
            Self::ChangeCellMode => "Change Cell Mode",
            Self::ChangeCreature => "Change Creature",
            Self::DetectMonsters => "Detect Monsters",
            Self::FieldsState => "Fields State",
            Self::ForgetMap => "Forget Map",
            Self::ForgetMapOnServer => "Forget Map (Server)",
            Self::GeneratePoi => "Generate POI",
            Self::GiveItem => "Give Item",
            Self::GodMode => "God Mode",
            Self::GoToWorld => "Go to World",
            Self::HerbsState => "Herbs State",
            Self::ItemPictures => "Item Pictures",
            Self::KillMob => "Kill Mob",
            Self::ResetView => "Reset View",
            Self::RevealMap => "Reveal Map",
            Self::ShowDebugInfo => "Show Debug Info",
            Self::ShowWidgets => "Show Widgets",
            Self::StairsStyle => "Stairs Style",
            Self::SunsAndMoons => "Suns and Moons",
            Self::TeleportMode => "Teleport Mode",
            Self::TeleportToHall => "Teleport to Hall of Worlds",
            Self::TeleportToPoiShowcase => "Teleport to POI Showcase",
            Self::TeleportToStart => "Teleport to Start",
            Self::Truesight => "Truesight",
            Self::ViewPlanet => "View Planet",
            Self::WorldTime => "World Time",
        }
    }

    /// The action with this name, as the menu shows it, in any case (dev
    /// tools: the debug API runs entries by name).
    #[cfg(any(test, all(feature = "debug-api", not(target_arch = "wasm32"))))]
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|action| action.name().eq_ignore_ascii_case(name.trim()))
    }

    /// Every action's name, in the menu's order.
    #[cfg(any(test, all(feature = "debug-api", not(target_arch = "wasm32"))))]
    pub fn names() -> impl Iterator<Item = &'static str> {
        Self::ALL.into_iter().map(Self::name)
    }

    /// Whether choosing it opens a submenu, rather than doing something.
    pub(super) fn has_submenu(self) -> bool {
        self == Self::GeneratePoi
    }

    /// What the menu's entry says: an entry with a submenu ends in "...", and its
    /// plain `name` is the submenu's title.
    pub(super) fn label(self) -> String {
        if self.has_submenu() {
            format!("{}...", self.name())
        } else {
            self.name().to_owned()
        }
    }

    /// Whether it can be used now.
    pub(super) fn is_available(self, facts: &Facts) -> bool {
        if self.needs_server_dev_tools() && !facts.dev_tools {
            return false;
        }
        match self {
            // In a site it would stop at a wall.
            Self::AutoWalk => facts.view == GameView::WorldMap,
            Self::ChangeCellMode => facts.view == GameView::Dungeon,
            // A town is known from the start, and would only come straight back.
            Self::ForgetMap => !facts.in_town,
            Self::RevealMap => facts.view == GameView::Dungeon && !facts.in_town,
            Self::FieldsState => facts.view == GameView::Dungeon && facts.has_fields,
            Self::HerbsState => facts.view == GameView::Dungeon && facts.has_herbs,
            Self::StairsStyle => facts.view == GameView::Dungeon && facts.has_stairs,
            Self::TeleportToPoiShowcase => shared::showcase::ENABLED,
            _ => true,
        }
    }

    /// Whether it does nothing unless the server accepts dev tools.
    fn needs_server_dev_tools(self) -> bool {
        matches!(
            self,
            Self::ChangeCellMode
                | Self::ChangeCreature
                | Self::DetectMonsters
                | Self::ForgetMap
                | Self::ForgetMapOnServer
                | Self::GeneratePoi
                | Self::GiveItem
                | Self::GodMode
                | Self::GoToWorld
                | Self::KillMob
                | Self::RevealMap
                | Self::TeleportMode
                | Self::TeleportToHall
                | Self::TeleportToPoiShowcase
                | Self::TeleportToStart
                | Self::Truesight
                | Self::WorldTime
        )
    }

    /// For one that toggles, whether it is on.
    pub(super) fn is_on(self, facts: &Facts) -> Option<bool> {
        match self {
            Self::AutoWalk => Some(facts.walking),
            Self::ChangeCellMode => Some(facts.changing_cells),
            Self::DetectMonsters => Some(facts.detecting),
            Self::GodMode => Some(facts.god_mode),
            Self::Truesight => Some(facts.truesight),
            Self::KillMob => Some(facts.killing),
            Self::ShowDebugInfo => Some(facts.showing_info),
            Self::SunsAndMoons => Some(facts.suns_and_moons_open),
            Self::WorldTime => Some(facts.world_time_open),
            Self::TeleportMode => Some(facts.teleporting),
            _ => None,
        }
    }

    /// Whether its name matches what has been typed: anywhere in it, whatever
    /// the case.
    pub(super) fn matches(self, filter: &str) -> bool {
        matches_filter(self.name(), filter)
    }
}

/// A kind of poi as a name, with spaces between its words.
pub(super) fn kind_name(kind: PoiKind) -> String {
    let mut name = String::new();
    for letter in format!("{kind:?}").chars() {
        if letter.is_uppercase() && !name.is_empty() {
            name.push(' ');
        }
        name.push(letter);
    }
    name
}

#[cfg(test)]
mod tests;
