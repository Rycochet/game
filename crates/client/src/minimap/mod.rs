//! The maps: the round minimap always in the top-right corner, and the full
//! parchment map that M or clicking the minimap opens. Both show only what is known, drawn on the CPU into
//! a picture. See `docs/context/client/minimap.md`.

mod canvas;
mod floors;
mod frame;
mod grid;
mod lines;
mod projection;
mod sky;
mod sprites;
mod world;

use bevy::input::InputSystems;
use bevy::prelude::*;
use lines::MapRivers;
use sprites::Sprites;
use world::WorldTiles;

use crate::common::{GameView, PointerSystems};
use crate::player::PlayerSystems;
use crate::view3d::View3dSystems;
use crate::widgets::WidgetSystems;

// Split into files (`cargo xtask lint` keeps files small).
mod draw;
mod forget;
mod full_map;
mod gate_runes;
mod hold;
mod hover;
mod input;
mod layout;
mod marks;
mod markup;
mod names;
mod others;
mod overlay;
mod parts;
mod place;
mod redraw;
mod room;
mod scale;
mod site_icons;
mod sources;
mod spawn;
mod stamps;
mod teleport;

use draw::draw_maps;
pub use full_map::FullMap;
use full_map::show_darkening;
use hover::{find_hovered_body, name_what_is_hovered, spot_on_full_map};
use input::take_input;
use layout::{
    OverFullMap, full_map_area, full_map_corner, full_map_picture_corner, full_map_side,
    minimap_across, minimap_picture_size, minimap_trim, over_full_map,
};
use marks::{full_map_compass, minimap_border, minimap_marks};
use names::{hovered_name, hovered_poi, location_title};
use parts::{FullMapDarkening, HoverRunesPicture, MapPictures};
pub(super) use room::Room;
use scale::{shows_known, under_the_sky, units_per_pixel};
use sources::{DrawnFrom, Sources};
use stamps::prepare_stamps;
use teleport::go_into_clicked_poi;

/// The minimap's size against the window's height, its border and notch, and
/// its gap from the window's corner. Lengths are in logical pixels.
const MINIMAP_OF_HEIGHT: f32 = 0.2;
const BORDER: f32 = 5.0;
const NOTCH: f32 = 6.0;
/// The window height at which the sizes round the minimap (`BORDER`, `NOTCH`,
/// and the suns and moons) are as given: the height a window opens at.
const MINIMAP_BASE_HEIGHT: f32 = 720.0;
const CORNER_GAP: f32 = 8.0;

/// A grid square, plus the grid line shared with the next one.
const CELL_PIXELS: f32 = 11.0;
/// How big a shop's sign is drawn on its square.
const SIGN_PIXELS: f32 = 10.0;
/// How big stairs are drawn: a little smaller than the player's dot, for more
/// of a gap round them in their square.
const STAIRS_PIXELS: f32 = 8.0;
/// How tall a secret door's "S" is drawn, and the edge round it in the
/// parchment's colour.
const SECRET_DOOR_PIXELS: f32 = 7.2;
const SECRET_DOOR_EDGE: f32 = 1.0;
/// How big a poi's picture is drawn.
const ICON_PIXELS: f32 = 16.0;
/// How solid a completed poi's picture is.
const COMPLETED_OPACITY: f32 = 0.5;

/// The player's dot, across, and its outline; how far inside the map's edge
/// the arrow to it, in the same style and as wide, sits.
const DOT_PIXELS: f32 = 10.0;
const DOT_OUTLINE: f32 = 1.0;
const ARROW_INSET: f32 = 12.0;
/// The little arrow beside them on another floor, saying which way their floor is.
const FLOOR_HINT_PIXELS: f32 = 4.9;
/// The player's dot and arrow on another floor, against their usual size.
const OTHER_FLOOR_SIZE: f32 = 0.8;

/// The full map's parchment against the window's shorter side, the margin
/// left bare on it, and the title's height above the map.
const FULL_MAP_OF_WINDOW: f32 = 0.9;
const FULL_MAP_MARGIN: f32 = 20.0;
const TITLE_HEIGHT: f32 = 34.0;
/// How far round the close button still counts as on it, for fingers.
const CLOSE_REACH: f32 = 10.0;
/// How far the box naming what is under the mouse sits above the margin.
const HOVER_LIFT: f32 = 12.0;

/// The full map's scale against the minimap's: when it opens, and its limits.
const OPENING_ZOOM: f32 = 2.0;
const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 4.0;
/// Scale per wheel notch; touchpads scroll in pixels, this many to a notch.
const ZOOM_PER_NOTCH: f32 = 1.25;
const PIXELS_PER_NOTCH: f32 = 100.0;

/// The compass's lines, and the cross's length against the window's height,
/// with the shortest it may be; how far it stands in from the map's corner.
const COMPASS_LINE: f32 = 5.0;
const COMPASS_OF_HEIGHT: f32 = 0.05;
const COMPASS_MIN_LENGTH: f32 = 36.0;
const COMPASS_INSET: f32 = 40.0;
/// A map dragged away has to keep something known this far inside its edges.
const KEEP_KNOWN_INSIDE: f64 = 40.0;

/// How long working out colours for the world map may take per redraw.
const MINIMAP_BUDGET_MS: f64 = 3.0;
const FULL_MAP_BUDGET_MS: f64 = 10.0;

/// A map is redrawn at least this often, since what is remembered can change
/// without marking anything changed (`dungeon_map` adds to it).
const REDRAW_SECONDS: f32 = 0.25;

/// Which full-screen layer the full map is.
const FULL_MAP_LAYER: i32 = 0;

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FullMap>()
            .init_resource::<hold::MinimapHold>()
            .init_resource::<WorldTiles>()
            .init_resource::<MapRivers>()
            .init_resource::<Sprites>()
            .init_resource::<overlay::MarksLayer>()
            .add_systems(Startup, (markup::spawn_maps, spawn::spawn_darkening))
            .add_systems(
                Update,
                forget::forget_world
                    .before(draw_maps)
                    .run_if(on_message::<crate::common::WorldChanged>),
            )
            .add_systems(
                PreUpdate,
                (
                    go_into_clicked_poi,
                    hold::hold_minimap,
                    take_input,
                    floors::back_to_own_floor,
                )
                    .chain()
                    .after(InputSystems)
                    .after(PointerSystems)
                    .after(WidgetSystems::Modal),
            )
            .add_systems(
                Update,
                (
                    markup::bind_maps,
                    place::place_maps,
                    draw_maps,
                    overlay::draw_marks,
                )
                    .chain()
                    .after(PlayerSystems::Travel)
                    .after(View3dSystems::Place)
                    .run_if(not(in_state(GameView::Connecting))),
            )
            .add_systems(
                Update,
                (
                    name_what_is_hovered,
                    find_hovered_body,
                    floors::show_buttons,
                    show_darkening,
                )
                    .run_if(not(in_state(GameView::Connecting))),
            );
    }
}

#[cfg(test)]
mod tests;
