//! What the maps are drawn from, and their nodes on screen.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{FullMap, FullMapPanel, FullMapPicture, FullMapTitle, Minimap};
use crate::common::{
    CurrentDungeon, CurrentWorld, FirstFloor, GameClock, GameView, OtherLevels, Portal, SkyNow,
    SmoothPosition, Token, Weather,
};
use crate::player::Player;
use crate::surface_style::SurfaceStyle;
use crate::view3d::OrbitCamera;

/// What a picture was last drawn from: if nothing here changes, it needn't be.
#[derive(Clone, PartialEq)]
pub(super) struct DrawnFrom {
    pub(super) view: GameView,
    pub(super) floor: Option<u32>,
    pub(super) open: bool,
    pub(super) centre: (f64, f64),
    pub(super) player: (f64, f64),
    pub(super) turn: (f32, f32),
    pub(super) scale: f64,
    pub(super) size: (u32, u32),
    pub(super) water_and_snow: (u64, u64),
}

/// What maps are drawn from.
#[derive(SystemParam)]
pub(super) struct Sources<'w, 's> {
    pub(super) view: Res<'w, State<GameView>>,
    pub(super) window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
    pub(super) ui_scale: Res<'w, UiScale>,
    pub(super) player: Single<'w, 's, &'static SmoothPosition, With<Player>>,
    pub(super) camera: Single<'w, 's, &'static OrbitCamera>,
    pub(super) world: Option<Res<'w, CurrentWorld>>,
    pub(super) dungeon: Res<'w, CurrentDungeon>,
    pub(super) levels: Res<'w, OtherLevels>,
    pub(super) first_floor: Res<'w, FirstFloor>,
    pub(super) style: Res<'w, SurfaceStyle>,
    pub(super) clock: Res<'w, GameClock>,
    pub(super) weather: Res<'w, Weather>,
    pub(super) sky: Res<'w, SkyNow>,
    pub(super) pois: Query<'w, 's, (&'static Portal, &'static Token)>,
    pub(super) asset_server: Res<'w, AssetServer>,
    pub(super) full: Res<'w, FullMap>,
}

/// A map's node, to size and show or hide.
pub(super) type Shown = (&'static mut Node, &'static mut Visibility);
pub(super) type OnlyMinimap = (With<Minimap>, Without<FullMapPanel>);
pub(super) type OnlyPanel = (With<FullMapPanel>, Without<Minimap>);
pub(super) type OnlyPicture = (
    With<FullMapPicture>,
    Without<Minimap>,
    Without<FullMapPanel>,
);

/// The maps' parts on screen.
#[derive(SystemParam)]
pub(super) struct Screen<'w, 's> {
    pub(super) minimap: Single<'w, 's, Shown, OnlyMinimap>,
    pub(super) panel: Single<'w, 's, Shown, OnlyPanel>,
    pub(super) picture: Single<'w, 's, &'static mut Node, OnlyPicture>,
    pub(super) title: Single<'w, 's, &'static mut Text, With<FullMapTitle>>,
}
