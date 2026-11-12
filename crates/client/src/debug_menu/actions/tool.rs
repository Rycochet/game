//! The debug tool each entry needs permission for (`common::DebugTools`): an
//! entry not permitted isn't shown. Accounts needs none: it goes by the
//! account's role (`common::AccountRole`).

use shared::role::DebugTool;

use super::DebugAction;

impl DebugAction {
    pub(in crate::debug_menu) fn tool(self) -> Option<DebugTool> {
        Some(match self {
            Self::Accounts => return None,
            Self::ShowDebugInfo => DebugTool::ShowDebugInfo,
            Self::LightningNow => DebugTool::LightningNow,
            Self::SunsAndMoons => DebugTool::SunsAndMoons,
            Self::DrawingStates => DebugTool::DrawingStates,
            Self::ItemPictures => DebugTool::ItemPictures,
            Self::ShowWidgets => DebugTool::ShowWidgets,
            Self::ViewPlanet => DebugTool::ViewPlanet,
            Self::TeleportMode => DebugTool::TeleportMode,
            Self::TeleportToWorld => DebugTool::TeleportToWorld,
            Self::GodMode => DebugTool::GodMode,
            Self::Truesight => DebugTool::Truesight,
            Self::ChangeCreature => DebugTool::ChangeCreature,
            Self::GiveItem => DebugTool::GiveItem,
            Self::Vitals => DebugTool::Vitals,
            Self::LearnSkills => DebugTool::LearnSkills,
            Self::Alignment => DebugTool::Alignment,
            Self::CreateMob => DebugTool::CreateMob,
            Self::ExamineMob => DebugTool::ExamineMob,
            Self::FreezeMobs => DebugTool::FreezeMobs,
            Self::DetectMobs => DebugTool::DetectMobs,
            Self::RevealMap => DebugTool::RevealMap,
            Self::ForgetMap => DebugTool::ForgetMap,
            Self::RemakeSite => DebugTool::RemakeSite,
            Self::ChangeCellMode => DebugTool::ChangeCellMode,
            Self::OpenAllDoors => DebugTool::OpenAllDoors,
            Self::CloseAllDoors => DebugTool::CloseAllDoors,
            Self::KeyForThisDoor => DebugTool::KeyForThisDoor,
            Self::WorldTime => DebugTool::WorldTime,
            Self::LightningStrikeHere => DebugTool::LightningStrikeHere,
            Self::GeneratePoi => DebugTool::GeneratePoi,
            Self::DevArea | Self::RegenerateDevArea => DebugTool::DevArea,
        })
    }
}
