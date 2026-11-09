//! The groups the menu's entries are shown in, each under its own heading.

use super::DebugAction;

/// A group of entries, in the order the menu shows them (owner's design,
/// 2026-11-09).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(in crate::debug_menu) enum Group {
    /// What only this screen shows.
    Display,
    /// Pictures of what the game draws.
    Galleries,
    Travel,
    /// The player's own character.
    Character,
    Mobs,
    /// The level or map the player is on.
    Level,
    /// The whole world, and making places.
    World,
}

impl Group {
    /// Its heading in the menu.
    pub(in crate::debug_menu) fn name(self) -> &'static str {
        match self {
            Self::Display => "Display",
            Self::Galleries => "Galleries",
            Self::Travel => "Travel",
            Self::Character => "Character",
            Self::Mobs => "Mobs",
            Self::Level => "Level",
            Self::World => "World",
        }
    }
}

impl DebugAction {
    /// The group it is shown in.
    pub(in crate::debug_menu) fn group(self) -> Group {
        match self {
            Self::ShowDebugInfo | Self::LightningNow | Self::SunsAndMoons | Self::DrawingStates => {
                Group::Display
            }
            Self::ItemPictures | Self::ShowWidgets | Self::ViewPlanet => Group::Galleries,
            Self::TeleportMode | Self::TeleportToWorld => Group::Travel,
            Self::GodMode
            | Self::Truesight
            | Self::ChangeCreature
            | Self::GiveItem
            | Self::Vitals
            | Self::LearnSomeSkills
            | Self::ForgetSkills => Group::Character,
            Self::CreateMob
            | Self::InspectMob
            | Self::KillMob
            | Self::DeleteMob
            | Self::FreezeMobs
            | Self::DetectMobs => Group::Mobs,
            Self::RevealMap
            | Self::ForgetMap
            | Self::RemakeSite
            | Self::ChangeCellMode
            | Self::OpenAllDoors
            | Self::CloseAllDoors
            | Self::KeyForThisDoor => Group::Level,
            Self::WorldTime
            | Self::LightningStrikeHere
            | Self::GeneratePoi
            | Self::DevArea
            | Self::RegenerateDevArea => Group::World,
        }
    }
}
