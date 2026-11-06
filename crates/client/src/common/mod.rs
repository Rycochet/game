//! Shared building blocks that every other client module may use.
//!
//! Keep this small: only types that genuinely need to be shared between
//! feature modules belong here. See `docs/context/client/common.md`.

mod floors;
mod hosting;
mod preferences;
mod servers;
pub use floors::{FirstFloor, OtherLevels};
pub use hosting::Hosting;
#[cfg(not(target_arch = "wasm32"))]
pub use preferences::client_file;
pub use preferences::{InterfaceSize, Preferences};
pub use servers::{ChosenServer, SavedServer};

use bevy::prelude::*;

// Split into files (`cargo xtask lint` keeps files small).
mod clock;
mod creatures;
mod debug;
mod display;
mod dungeon;
mod items;
mod map;
mod pointer;
mod roads;
mod sky;
mod states;
mod token;
mod video;
mod view;
mod window;
#[cfg(not(target_arch = "wasm32"))]
mod window_follow;
mod window_open;
#[cfg(not(target_arch = "wasm32"))]
mod window_save;
mod world;

pub use clock::{GameClock, HerbGrowthShift, NIGHT_LIGHT, ServerClock};
pub use creatures::{
    CarryLimit, MakesCharacters, MakingCharacter, Other, PlayerSpeed, SheetText, SpentEnergy,
};
pub use debug::{DebugInfo, DetectMonsters, GodMode, KillMob, ServerDevTools, Truesight};
pub use display::{
    DisplayChoice, DisplayInUse, DisplayMode, RememberWindow, Resolution, WindowPrefs, resolutions,
};
pub use dungeon::CurrentDungeon;
pub use items::{
    BandShift, GROUND, GaveItem, HeldGift, HeldOverGifts, HeldStock, HoveredBag, NO_QUALITY_NAME,
    StockCell, TakesGifts, TookItem, quality_colour,
};
pub use map::{
    Clickable, LightSource, MapPosition, Portal, SharesSight, SmoothPosition, WorldReturn,
};
pub use pointer::{
    PointerPosition, PointerSystems, TouchGesture, UiPointer, in_interface, interface_area,
};
pub use roads::KnownRoads;
pub use sky::SkyNow;
use sky::update_sky;
pub use states::{ClickMode, GameView, LoadingStep, ServerStatus, Session};
pub use token::{Attitude, Icon, IconPicture, Token, TokenShape, TokenSize};
pub use video::{
    AntiAliasing, BRIGHTNESS_MOST, Background, FrameRate, IN_BROWSER, Precipitation, Preset,
    RENDER_SCALES, Remembered, Shadows, VideoPrefs, ViewDistance,
};
pub use view::{ViewFacing, ViewStats};
pub use window::{KeepDrawing, frame_settings};
pub use world::{CurrentWorld, KnownPois, WorldChanged};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        let preferences = preferences::load_preferences();
        window_open::open_as_saved(app, &preferences.window);
        app.insert_resource(ChosenServer::from_saved(&preferences))
            .insert_resource(DisplayInUse(preferences.window.display))
            .insert_resource(frame_settings(true, &preferences.video))
            .init_resource::<RememberWindow>()
            .insert_resource(preferences);
        app.insert_resource(ClearColor(Color::BLACK))
            .init_resource::<KeepDrawing>()
            .init_resource::<LoadingStep>()
            .init_resource::<CurrentDungeon>()
            .init_resource::<OtherLevels>()
            .init_resource::<FirstFloor>()
            .init_resource::<WorldReturn>()
            .init_resource::<ServerStatus>()
            .init_resource::<Session>()
            .init_resource::<Hosting>()
            .init_state::<GameView>()
            .init_resource::<GameClock>()
            .init_resource::<HerbGrowthShift>()
            .init_resource::<ServerClock>()
            .add_message::<WorldChanged>()
            .init_resource::<SkyNow>()
            .init_resource::<ViewStats>()
            .init_resource::<ViewFacing>()
            .init_resource::<ClickMode>()
            .init_resource::<PlayerSpeed>()
            .init_resource::<CarryLimit>()
            .init_resource::<MakingCharacter>()
            .init_resource::<MakesCharacters>()
            .init_resource::<DetectMonsters>()
            .init_resource::<GodMode>()
            .init_resource::<Truesight>()
            .init_resource::<KillMob>()
            .init_resource::<ServerDevTools>()
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
