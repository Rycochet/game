//! Shared building blocks that every other client module may use.
//!
//! Keep this small: only types that genuinely need to be shared between
//! feature modules belong here. See `docs/context/client/common.md`.

use std::time::Duration;

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use shared::rng::Seed;
use shared::surface::Surface;
use shared::world::WorldParams;

/// Fixed seed until the server tells the client which world it is in.
const WORLD_SEED: u64 = 12345;

/// Real seconds per in-game hour.
const SECONDS_PER_HOUR: f32 = 60.0;

/// Frame rate cap while the window has focus. Not a twitch game, so this saves
/// GPU and battery; input still wakes the app immediately.
const FOCUSED_FPS: f64 = 30.0;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        let params = WorldParams::from_seed(Seed::new(WORLD_SEED));
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(WinitSettings {
                focused_mode: UpdateMode::reactive(Duration::from_secs_f64(1.0 / FOCUSED_FPS)),
                unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(1)),
            })
            .insert_resource(CurrentWorld {
                surface: Surface::new(&params),
                params,
            })
            .init_resource::<GameClock>()
            .init_resource::<Weather>()
            .init_resource::<ViewStats>()
            .init_state::<MapView>()
            .add_systems(Update, (advance_clock, switch_view));
    }
}

/// The world the player is in, and its surface generator.
#[derive(Resource)]
pub struct CurrentWorld {
    pub params: WorldParams,
    pub surface: Surface,
}

/// In-game time of day, in whole hours (`0..24`). Only changes once per hour,
/// so systems can redraw on `is_changed()`.
#[derive(Resource, Default, PartialEq)]
pub struct GameClock {
    pub hour: u32,
}

/// Snow line for normal weather: above every mountain cap, so only the
/// style's `mountain_height` applies.
pub const NORMAL_SNOW_LINE: f64 = 128.0;

/// Global weather. `snow_line` is the height above which everything is snow;
/// as it falls towards 0 the water ices over (see `surface_style`).
#[derive(Resource, Debug, PartialEq)]
pub struct Weather {
    pub snow_line: f64,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            snow_line: NORMAL_SNOW_LINE,
        }
    }
}

/// Which way the world is drawn. Each view module spawns its own camera and
/// entities on entering its state and they are removed on leaving it.
#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MapView {
    /// The original stylised slices (`slice_view`).
    #[default]
    Slices,
    /// A lit 3D terrain mesh (`mesh_view`).
    Mesh,
}

/// A line of performance info from the active view, shown by the UI.
#[derive(Resource, Default)]
pub struct ViewStats {
    pub summary: String,
}

fn advance_clock(time: Res<Time>, mut clock: ResMut<GameClock>) {
    let hour = (time.elapsed_secs() / SECONDS_PER_HOUR) as u32 % 24;
    // Only write when it changes, so change detection stays meaningful.
    clock.set_if_neq(GameClock { hour });
}

/// Tab switches between the views.
fn switch_view(
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<State<MapView>>,
    mut next: ResMut<NextState<MapView>>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        next.set(match view.get() {
            MapView::Slices => MapView::Mesh,
            MapView::Mesh => MapView::Slices,
        });
    }
}
