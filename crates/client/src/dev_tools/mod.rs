//! Developer helpers, controlled by environment variables. Native only; does
//! nothing unless a variable is set. See `docs/context/client/dev_tools.md`.
//! Starting positions and states are applied once connected to the server.
//!
//! - `HOW_DEBUG_POS=x,y`: starting grid position on the world map (the embedded
//!   server moves the player there).
//! - `HOW_DEBUG_ENTER=kind[,seed[,level]]`: start inside a made-up site at the
//!   starting position (the embedded server allows it): `kind` is an index
//!   into `PoiKind::ALL`, `seed` its map seed (default 1), `level` the
//!   level (default 0, the way in).
//! - `HOW_DEBUG_CELLS=x,y,kind;...`: on arriving in a site, change cells of the
//!   level, from where the player stands (`kind`: `floor`, `wall`, `door`,
//!   `window`, `water` or `lava`), to look at walls and liquids the generators
//!   don't make.
//! - `HOW_DEBUG_MOVE=dx,dy`: two seconds after starting, step the player this far
//!   (e.g. off the stairs they arrived on, to see them).
//! - `HOW_DEBUG_STAIRS=n`: draw all stairs as this style (an index into
//!   `StairsStyle::ALL`).
//! - `HOW_DEBUG_FIELDS=n`: draw every field as this stage (its `variety`: for crops
//!   0 ploughed, 1 grass, 2 short, 3 tall, 4 ripe).
//! - `HOW_DEBUG_HERBS=n`: draw every herb tile `n` stages (game days) further on.
//! - `HOW_DEBUG_TRAVEL=x,y`: start travelling to a grid position (as if clicked),
//!   once at `HOW_DEBUG_POS` if set, otherwise two seconds after starting.
//! - `HOW_DEBUG_WALK=true`: start with auto-walk on (as if G was pressed).
//! - `HOW_DEBUG_SNOW=n`: start with the snow line at `n` (128 normal, 0 frozen).
//! - `HOW_DEBUG_TIME=n`: once connected, ask the server to move the clock to `n` game
//!   hours since the world began.
//! - `HOW_DEBUG_CELESTIAL=n`: the suns and moons of terrain seed `n`, not this world's.
//! - `HOW_DEBUG_WINDOW_SIZE=w,h`: resize the window (logical pixels) a second after
//!   starting, to check that views handle resizing.
//! - `HOW_DEBUG_TOKEN=shape,attitude[,completed[,size]]`: the player's token, as
//!   numbers (indexes into `TokenShape::ALL` and `Attitude::ALL`), e.g. `3,2`
//!   is a blue hexagon; a third value of `1` makes it completed, a fourth is
//!   an index into `TokenSize::ALL` (`0` tiny ... `5` titan).
//! - `HOW_DEBUG_VIEW=yaw[,pitch[,zoom]]`: start with the map view turned by `yaw`
//!   degrees (positive moves the camera towards the east; 90 looks west, 180
//!   looks south), tilted to `pitch` degrees above the horizon and zoomed in
//!   (the camera distance is divided by `zoom`; 1.6667 is the wheel's
//!   closest).
//! - `HOW_DEBUG_MAP=dx,dy`: two seconds after starting, open the full map, moved
//!   this many grid points from the player (`0,0` to centre it on them).
//! - `HOW_DEBUG_MAP_ZOOM=n`: the full map opened by `HOW_DEBUG_MAP` at this zoom.
//! - `HOW_DEBUG_PANEL=1`: open the Show Widgets panel; `suns`: Suns and Moons;
//!   `pictures`: Item Pictures; `generate`: Generate POI's submenu.
//! - `HOW_DEBUG_MENU=text`: start with the debug menu open, filtered by `text`.
//! - `HOW_DEBUG_SYSTEM_MENU=menu`: start with the system menu open; `quit`: at
//!   its requester; `settings`: at its settings.
//! - `HOW_DEBUG_FIRST_FLOOR=n`: number floors from this (0 or 1), whatever the
//!   locale says.
//! - `HOW_DEBUG_SHOW_FLOOR=n`: three seconds after starting, look at this known
//!   floor (an index, from 0) on the maps.
//! - `HOW_DEBUG_INVENTORY=true`: two seconds after starting, have a few items and
//!   show the pockets and any bags. `HOW_DEBUG_HOLD=n`: no bags, but hold the item at
//!   this index in the pockets, under the cursor (see `HOW_DEBUG_CURSOR`).
//!   `HOW_DEBUG_DROP=true`: put all but the first of what is in the pockets on the
//!   ground underfoot first (in a site). `HOW_DEBUG_SECRETS=true`: truesight on, and
//!   things that are magical, blessed and cursed, to look at how they glow.
//! - `HOW_DEBUG_BAGS=true`: two seconds after starting, show a bag of every look in
//!   a row east from the player, only to look at (in a site).
//! - `HOW_DEBUG_CURSOR=x,y`: hold the mouse cursor here (logical pixels), to look
//!   at what hovering shows.
//! - `HOW_DEBUG_SCREENSHOT=path.png`: save a screenshot after `HOW_DEBUG_SCREENSHOT_DELAY`
//!   seconds (default 5), then quit. Lets tools check visuals without a person.
//! - `HOW_DEBUG_SKIP_MENU=true`: straight past the startup menu into the game, as
//!   if Play was pressed. On by default when any other variable here (but the
//!   debug API's and `HOW_DEBUG_KEEP_DRAWING`) is set; `false` keeps the menu.
//! - `HOW_DEBUG_KEEP_DRAWING=false`: with the debug API or a screenshot running,
//!   frames keep their focused rate in the background (`common::KeepDrawing`);
//!   this slows them to a second apart again, as players have it.

#[cfg(feature = "debug-api")]
mod debug_api;
#[cfg(feature = "debug-api")]
mod fake_input;

use bevy::prelude::*;

use crate::common::GameView;

// Split into files (`cargo xtask lint` keeps files small).
mod forced;
mod inventory;
mod keep_drawing;
mod menus;
mod moving;
mod parse;
mod screenshot;
mod skip_menu;
mod start;
mod view;
mod world;

use forced::{Forced, apply_forced};
#[cfg(feature = "debug-api")]
use menus::open_map;
#[cfg(feature = "debug-api")]
use parse::token_of;
use parse::{parse_cell, parse_pos, parse_token};
#[cfg(feature = "debug-api")]
use view::turn_camera;

/// Time between taking the screenshot and quitting, so it can be saved.
const SAVE_SECONDS: f32 = 2.0;

pub struct DevToolsPlugin;

/// The order of what is asked of the server on starting: move, then enter.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
enum Start {
    Place,
    Enter,
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug-api")]
        {
            debug_api::add(app);
            fake_input::add(app);
        }

        app.configure_sets(
            OnExit(GameView::Connecting),
            (Start::Place, Start::Enter).chain(),
        )
        .init_resource::<Forced>()
        .add_systems(Update, apply_forced);

        start::add(app);
        forced::read(app);
        menus::add(app);
        inventory::add(app);
        moving::add(app);
        view::add(app);
        world::add(app);
        screenshot::add(app);
        skip_menu::add(app);
        keep_drawing::add(app);
    }
}
