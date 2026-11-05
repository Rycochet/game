//! Positions on the map, portals, clickable tokens, lights and shared sight.

use bevy::prelude::*;
use shared::protocol::{GateInfo, SeenPoi};

/// Clicking this entity's token sends the player to it; arriving uses it:
/// entering a world map poi's site, or going through a World Gate into the
/// Hall. Needs `Clickable`. (Stairs and the Hall's gates are cells of the
/// level, not entities: see `site`.)
#[derive(Component, Clone, Copy, Debug)]
pub enum Portal {
    Poi(SeenPoi),
    Gate(GateInfo),
}

impl Portal {
    /// The poi it is, if it is one.
    pub fn poi(&self) -> Option<&SeenPoi> {
        match self {
            Self::Poi(poi) => Some(poi),
            Self::Gate(_) => None,
        }
    }
}

/// Its token can be clicked (`view3d::ClickedToken`).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Clickable;

/// Position on the current view's grid: the world grid (1 unit is about 1 km)
/// or a dungeon's cells. `y` increases southwards (towards the viewer in the
/// starting view). For the player while travelling, this is the nearest grid
/// point to `SmoothPosition`.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapPosition {
    pub x: i64,
    pub y: i64,
}

/// Where the player stood on the world map before entering a site: the way
/// back out, and the place whose biome and weather the site's outside shares.
#[derive(Resource, Default)]
pub struct WorldReturn(pub MapPosition);

/// The exact position, fractional while travelling (the player). Equal to
/// `MapPosition` whenever the entity is standing still. Read it for smooth
/// drawing; anything with a `Token` needs one to be drawn. To move the
/// player, change `MapPosition` or send `player::TravelTo`.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct SmoothPosition {
    pub x: f64,
    pub y: f64,
}

impl From<MapPosition> for SmoothPosition {
    fn from(position: MapPosition) -> Self {
        Self {
            x: position.x as f64,
            y: position.y as f64,
        }
    }
}

/// Gives out light in dungeons, `radius` cells (straight-line distance), in
/// `colour`. Put it on any entity with a position.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct LightSource {
    pub radius: u32,
    pub colour: Color,
}

/// Shares its sight with the player in dungeons: what it can see is shown
/// too. The player always counts; put this on allies or summoned eyes.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SharesSight;
