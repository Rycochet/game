//! Shared building blocks that every other client module may use.
//!
//! Keep this small: only types that genuinely need to be shared between
//! feature modules belong here. See `docs/context/client/common.md`.

mod browser;
mod characters;
pub mod controls;
mod floors;
mod hosting;
mod hover;
mod local_time;
mod login;
pub use controls::{Action, Binding, Controls, Group, Mods};
mod preferences;
mod servers;
pub use floors::{FirstFloor, OtherLevels};
pub use hosting::{Hosting, Lockdown};
#[cfg(not(target_arch = "wasm32"))]
pub use preferences::client_file;
pub use preferences::{CreateMobPrefs, DevAreaPrefs, InterfaceSize, Preferences};
pub use servers::{ChosenServer, HomeServer, SavedServer};

use bevy::prelude::*;

// Split into files (`cargo xtask lint` keeps files small).
mod account_role;
mod clock;
mod creatures;
mod debug;
mod difficulty;
mod display;
mod drawn_ground;
mod dungeon;
mod graphics;
mod item_text;
mod items;
mod lightning;
mod map;
mod own_ability;
mod pointer;
mod rivers;
mod roads;
mod sky;
mod sneaking;
mod states;
mod step_pad;
mod target;
mod token;
mod view;
mod vitals;
mod wild_heights;
mod window;
#[cfg(not(target_arch = "wasm32"))]
mod window_follow;
mod window_open;
#[cfg(not(target_arch = "wasm32"))]
mod window_save;
mod world;

pub use account_role::{AccountRole, OpenAccounts};
pub use browser::open_in_browser;
pub use characters::{CharacterChoice, CharacterSelect, PlayFor, PlayableHere};
pub use clock::{GameClock, HerbGrowthShift, NIGHT_LIGHT, ServerClock};
pub use creatures::{
    AbilitiesPage, CancelledEnergy, CarryLimit, Dressing, MakesCharacters, MakingCharacter, Other,
    PlayerGait, PlayerSpeed, SkillsPage, SpentEnergy,
};
pub use debug::{
    CreateMobOrder, DebugInfo, DebugTools, DetectMonsters, FreezeMobs, GodMode, InspectedMob,
    TeleportToWorldAsked, Truesight,
};
pub use difficulty::{IRONMAN_QUESTION, PlayDifficulty, difficulty_words};
pub use display::{
    DisplayChoice, DisplayInUse, DisplayMode, RememberWindow, Resolution, WindowPrefs, resolutions,
};
pub use drawn_ground::DrawnGround;
pub use dungeon::CurrentDungeon;
pub use graphics::{
    AntiAliasing, BRIGHTNESS_MOST, Background, FrameRate, GraphicsPrefs, IN_BROWSER, MaxResolution,
    Precipitation, Preset, RENDER_SCALES, Remembered, Shadows, ViewDistance,
};
pub use hover::{HoverName, HoveredInSky, HoveredOnMap, StaffMark, poi_name, words};
pub use item_text::{DroppedOnText, HeldOverText, KnownItem, KnownItems, TakesItemText};
pub use items::{
    BandShift, GROUND, GaveItem, HeldGift, HeldOverGifts, HeldStock, HoveredBag, NO_QUALITY_NAME,
    StockCell, TakesGifts, TookItem, quality_colour,
};
pub use lightning::{BoltStruck, Lightning, LightningStrike};
pub use local_time::local_date_time;
pub use login::{AccountEmail, LoggingIn, LoginChoice, LoginProvider};
pub use map::{
    Clickable, LightSource, MapPosition, Portal, SharesSight, SmoothPosition, WorldReturn,
};
pub use own_ability::OwnAbility;
pub use pointer::{
    PointerPosition, PointerSystems, TouchGesture, UiPointer, in_interface, interface_area,
};
pub use rivers::NearRivers;
pub use roads::KnownRoads;
pub use sky::SkyNow;
use sky::update_sky;
pub use sneaking::Sneaking;
pub use states::{ClickMode, GameView, LoadingStep, ServerStatus, Session};
pub use step_pad::{PadButton, PadSteps};
pub use target::Target;
pub use token::{Attitude, Icon, IconPicture, Token, TokenShape, TokenSize};
pub use view::{ViewFacing, ViewStats};
pub use vitals::{BottomHud, OwnVitals, ToldVitals, VitalsCircle};
pub use wild_heights::{DRAWN_PER_16TH, WildCellHeights};
pub use window::{KeepDrawing, frame_settings};
pub use world::{CurrentWorld, KnownPois, WorldChanged};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        let preferences = preferences::load_preferences();
        window_open::open_as_saved(app, &preferences.window);
        app.init_resource::<ChosenServer>()
            .init_resource::<HomeServer>()
            .insert_resource(DisplayInUse(preferences.window.display))
            .insert_resource(frame_settings(true, &preferences.graphics))
            .init_resource::<RememberWindow>()
            .insert_resource(preferences);
        app.insert_resource(ClearColor(Color::BLACK))
            .init_resource::<KeepDrawing>()
            .init_resource::<OwnVitals>()
            .init_resource::<BottomHud>()
            .init_resource::<VitalsCircle>()
            .init_resource::<HoveredOnMap>()
            .init_resource::<HoveredInSky>()
            .init_resource::<Target>()
            .init_resource::<AccountRole>()
            .add_message::<OpenAccounts>()
            .init_resource::<PadSteps>()
            .init_resource::<OwnAbility>()
            .init_resource::<LoadingStep>()
            .init_resource::<CurrentDungeon>()
            .init_resource::<OtherLevels>()
            .init_resource::<FirstFloor>()
            .init_resource::<WorldReturn>()
            .init_resource::<ServerStatus>()
            .init_resource::<Session>()
            .init_resource::<Hosting>()
            .init_resource::<Lockdown>()
            .init_resource::<PlayDifficulty>()
            .init_state::<GameView>()
            .init_resource::<GameClock>()
            .init_resource::<HerbGrowthShift>()
            .init_resource::<ServerClock>()
            .add_message::<WorldChanged>()
            .add_message::<LightningStrike>()
            .add_message::<TeleportToWorldAsked>()
            .add_message::<BoltStruck>()
            .add_message::<DroppedOnText>()
            .init_resource::<KnownItems>()
            .init_resource::<HeldOverText>()
            .init_resource::<SkyNow>()
            .init_resource::<Lightning>()
            .init_resource::<ViewStats>()
            .init_resource::<ViewFacing>()
            .init_resource::<ClickMode>()
            .init_resource::<PlayerSpeed>()
            .init_resource::<PlayerGait>()
            .init_resource::<WildCellHeights>()
            .init_resource::<DrawnGround>()
            .init_resource::<NearRivers>()
            .init_resource::<Sneaking>()
            .init_resource::<CarryLimit>()
            .init_resource::<MakingCharacter>()
            .init_resource::<CharacterSelect>()
            .init_resource::<PlayFor>()
            .init_resource::<PlayableHere>()
            .add_message::<CharacterChoice>()
            .init_resource::<LoggingIn>()
            .init_resource::<AccountEmail>()
            .add_message::<LoginChoice>()
            .init_resource::<MakesCharacters>()
            .init_resource::<DetectMonsters>()
            .init_resource::<GodMode>()
            .init_resource::<Truesight>()
            .init_resource::<CreateMobOrder>()
            .init_resource::<FreezeMobs>()
            .init_resource::<InspectedMob>()
            .init_resource::<DebugTools>()
            .init_resource::<DebugInfo>()
            .init_resource::<PointerPosition>()
            .init_resource::<UiPointer>()
            .init_resource::<TouchGesture>()
            .add_systems(Last, preferences::save_preferences)
            .add_systems(Startup, window_open::maximise_as_saved)
            .add_systems(
                PreUpdate,
                update_sky.run_if(resource_exists::<CurrentWorld>),
            );
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Update,
            (
                window_open::place_on_monitor,
                window_follow::follow_display,
                window_save::remember_window,
                window::apply_vsync,
            ),
        );
    }
}

/// How many times round the planet `KnownPois::in_area` will look.
const MOST_PLANETS_IN_AN_AREA: i64 = 8;

#[cfg(test)]
mod tests;
