//! What the debug menu offers, and when each is possible. No Bevy systems here,
//! so it can be tested alone.

use shared::poi::PoiKind;

use crate::common::{DebugTools, GameView};
use crate::widgets::matches_filter;

/// One entry in the debug menu. Sending it does what it says.
#[derive(bevy::prelude::Message, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DebugAction {
    Accounts,
    ChangeCellMode,
    ChangeCreature,
    CloseAllDoors,
    CreateMob,
    DetectMobs,
    DevArea,
    DrawingStates,
    ForgetMap,
    FreezeMobs,
    GeneratePoi,
    GiveItem,
    GodMode,
    ExamineMob,
    ItemPictures,
    KeyForThisDoor,
    LearnSkills,
    LightningNow,
    LightningStrikeHere,
    OpenAllDoors,
    RegenerateDevArea,
    RemakeSite,
    RevealMap,
    ShowDebugInfo,
    ShowWidgets,
    SunsAndMoons,
    TeleportMode,
    TeleportToWorld,
    Truesight,
    ViewPlanet,
    Vitals,
    WorldTime,
}

/// The state of the game that the menu depends on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Facts {
    pub view: GameView,
    pub detecting: bool,
    pub god_mode: bool,
    pub truesight: bool,
    pub freezing: bool,
    pub teleporting: bool,
    pub changing_cells: bool,
    pub showing_info: bool,
    pub in_town: bool,
    pub world_time_open: bool,
    pub suns_and_moons_open: bool,
    /// The debug tools this player may use: an entry needing another isn't
    /// shown.
    pub tools: DebugTools,
    /// Whether the Accounts panel has anything for this account's role
    /// (`common::AccountRole::has_accounts_panel`).
    pub accounts: bool,
}

impl DebugAction {
    /// Every action, in the order the menu lists them: by group (`Group`),
    /// then as the owner ordered each group.
    pub(super) const ALL: [Self; 32] = [
        // Display
        Self::ShowDebugInfo,
        Self::LightningNow,
        Self::SunsAndMoons,
        Self::DrawingStates,
        // Galleries
        Self::ItemPictures,
        Self::ShowWidgets,
        Self::ViewPlanet,
        // Travel
        Self::TeleportMode,
        Self::TeleportToWorld,
        // Character
        Self::GodMode,
        Self::Truesight,
        Self::ChangeCreature,
        Self::GiveItem,
        Self::Vitals,
        Self::LearnSkills,
        // Mobs
        Self::CreateMob,
        Self::ExamineMob,
        Self::FreezeMobs,
        Self::DetectMobs,
        // Level
        Self::RevealMap,
        Self::ForgetMap,
        Self::RemakeSite,
        Self::ChangeCellMode,
        Self::OpenAllDoors,
        Self::CloseAllDoors,
        Self::KeyForThisDoor,
        // World
        Self::WorldTime,
        Self::LightningStrikeHere,
        Self::GeneratePoi,
        Self::DevArea,
        Self::RegenerateDevArea,
        // Admin Tools
        Self::Accounts,
    ];

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

    /// Whether choosing it opens something: a panel, a dialog, a submenu or
    /// a full-screen view (owner's rule: those end in "...").
    pub(super) fn opens_something(self) -> bool {
        matches!(
            self,
            Self::Accounts
                | Self::SunsAndMoons
                | Self::DrawingStates
                | Self::ItemPictures
                | Self::ShowWidgets
                | Self::ViewPlanet
                | Self::TeleportToWorld
                | Self::ChangeCreature
                | Self::GiveItem
                | Self::Vitals
                | Self::LearnSkills
                | Self::CreateMob
                | Self::ExamineMob
                | Self::WorldTime
                | Self::GeneratePoi
                | Self::DevArea
        )
    }

    /// What the menu's entry says: one that opens something ends in "...",
    /// and its plain `name` is that panel's or submenu's title.
    pub(super) fn label(self) -> String {
        if self.opens_something() {
            format!("{}...", self.name())
        } else {
            self.name().to_owned()
        }
    }

    /// Whether it is shown at all: only with permission for its tool, or for
    /// Accounts, with something on its panel for this role.
    pub(super) fn is_permitted(self, facts: &Facts) -> bool {
        match self.tool() {
            Some(tool) => facts.tools.may(tool),
            None => facts.accounts,
        }
    }

    /// Whether it can be used now.
    pub(super) fn is_available(self, facts: &Facts) -> bool {
        match self {
            Self::ChangeCellMode
            | Self::OpenAllDoors
            | Self::CloseAllDoors
            | Self::KeyForThisDoor => facts.view == GameView::Dungeon,
            // A town is known from the start, and would only come straight back.
            Self::ForgetMap => !facts.in_town,
            Self::RevealMap => facts.view == GameView::Dungeon && !facts.in_town,
            Self::LightningNow | Self::LightningStrikeHere => {
                matches!(facts.view, GameView::WorldMap | GameView::Dungeon)
            }
            _ => true,
        }
    }

    /// For one that toggles, whether it is on.
    pub(super) fn is_on(self, facts: &Facts) -> Option<bool> {
        match self {
            Self::ChangeCellMode => Some(facts.changing_cells),
            Self::DetectMobs => Some(facts.detecting),
            Self::GodMode => Some(facts.god_mode),
            Self::Truesight => Some(facts.truesight),
            Self::FreezeMobs => Some(facts.freezing),
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

mod group;
mod names;
mod tool;

#[cfg(test)]
use group::Group;

#[cfg(test)]
mod tests;
