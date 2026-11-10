//! The debug menu's tools, one permission each (owner, 2026-11-09: per tool,
//! for security), and who has each by default.

use serde::{Deserialize, Serialize};

/// One debug tool: a debug menu entry, or a button inside one's panel that
/// does something of its own (Examine Mob's Kill and Destroy).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DebugTool {
    // Display: only this screen.
    ShowDebugInfo,
    LightningNow,
    SunsAndMoons,
    DrawingStates,
    // Galleries.
    ItemPictures,
    ShowWidgets,
    ViewPlanet,
    // Travel.
    TeleportMode,
    /// Teleport to World: other worlds, their gates and the Hall of Worlds.
    TeleportToWorld,
    // Character.
    GodMode,
    Truesight,
    ChangeCreature,
    GiveItem,
    Vitals,
    LearnSkills,
    // Mobs.
    CreateMob,
    ExamineMob,
    KillMob,
    DestroyMob,
    FreezeMobs,
    DetectMobs,
    // Level.
    RevealMap,
    ForgetMap,
    RemakeSite,
    ChangeCellMode,
    OpenAllDoors,
    CloseAllDoors,
    KeyForThisDoor,
    // World.
    WorldTime,
    LightningStrikeHere,
    GeneratePoi,
    /// Dev Area, and making it again.
    DevArea,
    /// What only the debug API and starting variables use, with no entry:
    /// going straight into a site, back to the start, the snow line,
    /// revealing pois and deleting the character.
    Other,
}

impl DebugTool {
    pub const ALL: [Self; 33] = [
        Self::ShowDebugInfo,
        Self::LightningNow,
        Self::SunsAndMoons,
        Self::DrawingStates,
        Self::ItemPictures,
        Self::ShowWidgets,
        Self::ViewPlanet,
        Self::TeleportMode,
        Self::TeleportToWorld,
        Self::GodMode,
        Self::Truesight,
        Self::ChangeCreature,
        Self::GiveItem,
        Self::Vitals,
        Self::LearnSkills,
        Self::CreateMob,
        Self::ExamineMob,
        Self::KillMob,
        Self::DestroyMob,
        Self::FreezeMobs,
        Self::DetectMobs,
        Self::RevealMap,
        Self::ForgetMap,
        Self::RemakeSite,
        Self::ChangeCellMode,
        Self::OpenAllDoors,
        Self::CloseAllDoors,
        Self::KeyForThisDoor,
        Self::WorldTime,
        Self::LightningStrikeHere,
        Self::GeneratePoi,
        Self::DevArea,
        Self::Other,
    ];

    /// Only for Owners and Admins by default, and not given by
    /// `HOW_DEBUG_TOOLS`: they change the world for everyone in it.
    pub fn admin_only(self) -> bool {
        matches!(self, Self::WorldTime | Self::FreezeMobs)
    }

    /// Whether Moderators have it by default, as well as Developers (who have
    /// everything but the Admin-only ones): what helps look after players.
    pub fn for_moderators(self) -> bool {
        matches!(
            self,
            Self::ShowDebugInfo
                | Self::TeleportMode
                | Self::TeleportToWorld
                | Self::GodMode
                | Self::Truesight
                | Self::ExamineMob
                | Self::KillMob
                | Self::DestroyMob
                | Self::DetectMobs
                | Self::OpenAllDoors
                | Self::CloseAllDoors
                | Self::KeyForThisDoor
        )
    }
}
