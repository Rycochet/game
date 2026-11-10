//! What each debug menu entry is called, as the menu lists it.

use super::DebugAction;

impl DebugAction {
    /// The entry's name in the menu (and the user documentation), without the
    /// "..." of one that opens something (`label`).
    pub(in crate::debug_menu) fn name(self) -> &'static str {
        match self {
            Self::Accounts => "Accounts",
            Self::ChangeCellMode => "Change Cell Mode",
            Self::ChangeCreature => "Change Creature",
            Self::CloseAllDoors => "Close All Doors",
            Self::CreateMob => "Create Mob",
            Self::DetectMobs => "Detect Mobs",
            Self::DevArea => "Dev Area",
            Self::DrawingStates => "Drawing States",
            Self::ForgetMap => "Forget Map",
            Self::FreezeMobs => "Freeze Mobs",
            Self::GeneratePoi => "Generate POI",
            Self::GiveItem => "Give Item",
            Self::GodMode => "God Mode",
            Self::ExamineMob => "Examine Mob",
            Self::ItemPictures => "Item Pictures",
            Self::KeyForThisDoor => "Key for This Door",
            Self::LearnSkills => "Learn Skills",
            Self::LightningNow => "Lightning Now",
            Self::LightningStrikeHere => "Lightning Strike Here",
            Self::OpenAllDoors => "Open All Doors",
            Self::RegenerateDevArea => "Regenerate Dev Area",
            Self::RemakeSite => "Remake Site",
            Self::RevealMap => "Reveal Map",
            Self::ShowDebugInfo => "Show Debug Info",
            Self::ShowWidgets => "Show Widgets",
            Self::SunsAndMoons => "Suns and Moons",
            Self::TeleportMode => "Teleport Mode",
            Self::TeleportToWorld => "Teleport to World",
            Self::Truesight => "Truesight",
            Self::ViewPlanet => "View Planet",
            Self::Vitals => "Vitals",
            Self::WorldTime => "World Time",
        }
    }
}
