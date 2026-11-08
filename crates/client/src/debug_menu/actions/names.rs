//! What each debug menu entry is called, as the menu lists it.

use super::DebugAction;

impl DebugAction {
    /// The entry's name in the menu (and the user documentation).
    pub(in crate::debug_menu) fn name(self) -> &'static str {
        match self {
            Self::AutoWalk => "Auto Walk",
            Self::ChangeCellMode => "Change Cell Mode",
            Self::ChangeCreature => "Change Creature",
            Self::CreateMob => "Create Mob",
            Self::DeleteCharacter => "Delete Character",
            Self::DetectMonsters => "Detect Monsters",
            Self::DevArea => "Dev Area",
            Self::FieldsState => "Fields State",
            Self::ForgetMap => "Forget Map",
            Self::ForgetMapOnServer => "Forget Map (Server)",
            Self::ForgetSkills => "Forget Skills",
            Self::GeneratePoi => "Generate POI",
            Self::GiveItem => "Give Item",
            Self::GodMode => "God Mode",
            Self::GoToWorld => "Go to World",
            Self::HealFully => "Heal Fully",
            Self::HerbsState => "Herbs State",
            Self::Hurt => "Hurt",
            Self::ItemPictures => "Item Pictures",
            Self::KillMob => "Kill Mob",
            Self::DeleteMob => "Delete Mob",
            Self::FreezeMobs => "Freeze Mobs",
            Self::InspectMob => "Inspect Mob",
            Self::LearnSomeSkills => "Learn Some Skills",
            Self::LightningNow => "Lightning Now",
            Self::LightningStrikeHere => "Lightning Strike Here",
            Self::RegenerateDevArea => "Regenerate Dev Area",
            Self::ResetView => "Reset View",
            Self::RevealMap => "Reveal Map",
            Self::ShowDebugInfo => "Show Debug Info",
            Self::ShowWidgets => "Show Widgets",
            Self::SpendMana => "Spend Mana",
            Self::StairsStyle => "Stairs Style",
            Self::SunsAndMoons => "Suns and Moons",
            Self::TeleportMode => "Teleport Mode",
            Self::TeleportToGate => "Teleport to Gate",
            Self::TeleportToHall => "Teleport to Hall of Worlds",
            Self::TeleportToPoiShowcase => "Teleport to POI Showcase",
            Self::TeleportToStart => "Teleport to Start",
            Self::Truesight => "Truesight",
            Self::ViewPlanet => "View Planet",
            Self::VitalsBonus => "Vitals Bonus",
            Self::WorldTime => "World Time",
        }
    }
}
