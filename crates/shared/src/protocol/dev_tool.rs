//! Which client messages are developer tools.

use super::ClientMessage;

impl ClientMessage {
    /// Whether it is a developer tools message (`Dev...`), which only an
    /// embedded server accepts.
    pub fn is_dev_tool(&self) -> bool {
        matches!(
            self,
            Self::DevTeleport { .. }
                | Self::DevBackToStart
                | Self::DevToShowcase
                | Self::DevToHall { .. }
                | Self::DevLightning
                | Self::DevEnterSite { .. }
                | Self::DevSetCell { .. }
                | Self::DevSetWorldClock { .. }
                | Self::DevGoToWorld { .. }
                | Self::DevRevealPois
                | Self::DevRevealMap
                | Self::DevForgetMap
                | Self::DevForgetMapOnServer
                | Self::DevGenerateSite { .. }
                | Self::DevGenerateArea(_)
                | Self::DevMobs(_)
                | Self::DevGiveItem { .. }
                | Self::DevSetCreature { .. }
                | Self::DevDeleteCharacter
                | Self::DevDetectMonsters(_)
                | Self::DevTruesight(_)
                | Self::DevGodMode(_)
                | Self::DevKillMob { .. }
                | Self::DevVitals(_)
                | Self::DevSkills { .. }
                | Self::Door(super::DoorAsk::DevAll { .. } | super::DoorAsk::DevKey)
        )
    }
}
