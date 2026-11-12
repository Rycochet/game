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

/// The player's exact place on a level or in the Hall, in units (240 a
/// cell, `shared::spot`), as the server counts it: what free movement moves
/// and the server checks. `MapPosition` is then the cell it is in, and
/// `SmoothPosition` it in cells. Whoever puts the player somewhere on a level
/// (arriving, the server's corrections, teleports) sets all three
/// (`SpotPosition::place`). Unused on the world map.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpotPosition(pub shared::spot::Spot);

impl SpotPosition {
    /// The three positions for standing at `spot` on a level.
    pub fn place(spot: shared::spot::Spot) -> (Self, MapPosition, SmoothPosition) {
        let (x, y) = spot.cell();
        (
            Self(spot),
            MapPosition { x, y },
            SmoothPosition::of_units((spot.x, spot.y)),
        )
    }
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

impl SmoothPosition {
    /// A position the server sends in units (`protocol::moves`: on a level or
    /// in the Hall), in cells.
    pub fn of_units((x, y): (i64, i64)) -> Self {
        let span = shared::spot::CELL_SPAN as f64;
        Self {
            x: x as f64 / span,
            y: y as f64 / span,
        }
    }
}

/// A cell (or world map point) as the server counts it in `view`: a cell's
/// middle in units on a level or in the Hall (`protocol::moves`), the point
/// itself on the world map. For dev tools' teleports (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn in_map_measure(view: crate::common::GameView, cell: (i64, i64)) -> (i64, i64) {
    match view {
        crate::common::GameView::Dungeon => shared::protocol::units_of_cell(cell),
        _ => cell,
    }
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
