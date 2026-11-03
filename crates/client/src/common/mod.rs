//! Shared building blocks that every other client module may use.
//!
//! Keep this small: only types that genuinely need to be shared between
//! feature modules belong here. See `docs/context/client/common.md`.

mod floors;
mod preferences;
mod servers;
pub use floors::{FirstFloor, OtherLevels};
#[cfg(not(target_arch = "wasm32"))]
pub use preferences::client_file;
pub use preferences::{InterfaceSize, Preferences};
pub use servers::{ChosenServer, SavedServer};

use bevy::prelude::*;

// Split into files (`cargo xtask lint` keeps files small).
mod clock;
mod creatures;
mod debug;
mod dungeon;
mod items;
mod map;
mod pointer;
mod sky;
mod states;
mod token;
mod view;
mod weather;
mod window;
mod world;

pub use clock::{GameClock, HerbGrowthShift, NIGHT_LIGHT, ServerClock};
pub use creatures::{Other, PlayerSpeed, SheetText, SpentEnergy};
pub use debug::{DebugInfo, DetectMonsters, GodMode, KillMob, ServerDevTools, Truesight};
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
pub use sky::SkyNow;
use sky::update_sky;
pub use states::{ClickMode, GameView, LoadingStep, ServerStatus, Session};
pub use token::{Attitude, Icon, Token, TokenShape, TokenSize};
pub use view::{ViewFacing, ViewStats};
pub use weather::{NORMAL_SNOW_LINE, Weather};
use window::open_maximised;
pub use window::{KeepDrawing, frame_settings};
pub use world::{CurrentWorld, KnownPois};

/// Frame rate cap while the window has focus. Not a twitch game, so this saves
/// GPU and battery; input still wakes the app immediately.
const FOCUSED_FPS: f64 = 30.0;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        let preferences = preferences::load_preferences();
        app.insert_resource(ChosenServer::from_saved(&preferences))
            .insert_resource(preferences);
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(frame_settings(true))
            .init_resource::<KeepDrawing>()
            .init_resource::<LoadingStep>()
            .init_resource::<CurrentDungeon>()
            .init_resource::<OtherLevels>()
            .init_resource::<FirstFloor>()
            .init_resource::<WorldReturn>()
            .init_resource::<ServerStatus>()
            .init_resource::<Session>()
            .init_state::<GameView>()
            .init_resource::<GameClock>()
            .init_resource::<HerbGrowthShift>()
            .init_resource::<ServerClock>()
            .init_resource::<SkyNow>()
            .init_resource::<Weather>()
            .init_resource::<ViewStats>()
            .init_resource::<ViewFacing>()
            .init_resource::<ClickMode>()
            .init_resource::<PlayerSpeed>()
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
            .add_systems(Startup, open_maximised)
            .add_systems(
                PreUpdate,
                update_sky.run_if(resource_exists::<CurrentWorld>),
            );
    }
}

/// How many times round the planet `KnownPois::in_area` will look.
const MOST_PLANETS_IN_AN_AREA: i64 = 8;

#[cfg(test)]
mod tests;
