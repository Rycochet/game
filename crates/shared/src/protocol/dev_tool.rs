//! Which client messages are developer tools, and which tool each needs.

use super::{ClientMessage, DevMobTool, DevSkills, DoorAsk};
use crate::role::DebugTool;

impl ClientMessage {
    /// The debug tool a developer tools message (`Dev...`) needs permission
    /// for (`Permission::Debug`), or `None` for any other message.
    pub fn dev_tool(&self) -> Option<DebugTool> {
        Some(match self {
            Self::DevTeleport { .. } => DebugTool::TeleportMode,
            Self::DevGoToWorld { .. } | Self::DevToHall { .. } => DebugTool::TeleportToWorld,
            Self::DevLightning => DebugTool::LightningStrikeHere,
            Self::DevSetCell { .. } => DebugTool::ChangeCellMode,
            Self::DevSetWorldClock { .. } => DebugTool::WorldTime,
            Self::DevRevealMap => DebugTool::RevealMap,
            Self::DevForgetMap => DebugTool::ForgetMap,
            Self::DevForgetMapOnServer => DebugTool::RemakeSite,
            Self::DevGenerateSite { .. } => DebugTool::GeneratePoi,
            Self::DevGenerateArea(_) => DebugTool::DevArea,
            Self::DevMobs(DevMobTool::Create(_)) => DebugTool::CreateMob,
            Self::DevMobs(DevMobTool::Inspect(_)) => DebugTool::ExamineMob,
            Self::DevMobs(DevMobTool::Delete(_)) => DebugTool::DestroyMob,
            Self::DevMobs(DevMobTool::Freeze(_)) => DebugTool::FreezeMobs,
            Self::DevKillMob { .. } => DebugTool::KillMob,
            Self::DevGiveItem { .. } => DebugTool::GiveItem,
            Self::DevSetCreature { .. } => DebugTool::ChangeCreature,
            Self::DevDetectMonsters(_) => DebugTool::DetectMobs,
            Self::DevTruesight(_) => DebugTool::Truesight,
            Self::DevGodMode(_) => DebugTool::GodMode,
            Self::DevVitals(_) => DebugTool::Vitals,
            Self::DevSkills { .. } => DebugTool::LearnSkills,
            Self::Door(DoorAsk::DevAll { open: true }) => DebugTool::OpenAllDoors,
            Self::Door(DoorAsk::DevAll { open: false }) => DebugTool::CloseAllDoors,
            Self::Door(DoorAsk::DevKey) => DebugTool::KeyForThisDoor,
            Self::DevBackToStart
            | Self::DevEnterSite { .. }
            | Self::DevRevealPois
            | Self::DevDeleteCharacter => DebugTool::Other,
            _ => return None,
        })
    }

    /// Whether it is a developer tools message.
    pub fn is_dev_tool(&self) -> bool {
        self.dev_tool().is_some()
    }

    /// Whether using it goes in the audit log: every developer tool but those
    /// only asking (Examine Mob asks twice a second while open).
    pub fn audited(&self) -> bool {
        self.is_dev_tool()
            && !matches!(
                self,
                Self::DevMobs(DevMobTool::Inspect(_))
                    | Self::DevSkills {
                        change: DevSkills::Ask,
                        ..
                    }
            )
    }
}
