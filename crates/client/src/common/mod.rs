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
            .init_resource::<ViewFacing>()
            .add_systems(Update, advance_clock);
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

/// A line of performance info from the active view, shown by the UI.
#[derive(Resource, Default)]
pub struct ViewStats {
    pub summary: String,
}

/// Which way the map view is turned, as the compass direction nearest the top
/// of the screen, in quarter turns clockwise from north: 0 north (the default
/// view), 1 east, 2 south, 3 west. Written by the active view; read by
/// anything that turns screen directions into grid steps (the keyboard).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewFacing(pub u8);

impl ViewFacing {
    /// Turns a screen direction (`dy` −1 is up the screen, `dx` +1 right)
    /// into a grid step (`y` −1 is north, `x` +1 east).
    pub fn screen_to_grid(self, dx: i64, dy: i64) -> (i64, i64) {
        let (mut x, mut y) = (dx, dy);
        for _ in 0..self.0 % 4 {
            // A quarter turn clockwise: up becomes east, right becomes south.
            (x, y) = (-y, x);
        }
        (x, y)
    }
}

fn advance_clock(time: Res<Time>, mut clock: ResMut<GameClock>) {
    let hour = (time.elapsed_secs() / SECONDS_PER_HOUR) as u32 % 24;
    // Only write when it changes, so change detection stays meaningful.
    clock.set_if_neq(GameClock { hour });
}

/// How an entity is shown on maps: a flat shape extruded upwards (a "token",
/// like a board game piece), coloured by its attitude to the player. Put it on
/// any entity with a position and the views draw it.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Token {
    pub shape: TokenShape,
    pub attitude: Attitude,
    /// Secondary state, e.g. a place already dealt with: drawn about 30%
    /// darker and slightly see-through.
    pub completed: bool,
}

/// A 2D picture shown above an entity's token, always facing the camera. The
/// path is relative to the assets folder, e.g. `monsters/humanarcher.png`:
/// 54×54 pixel art whose main picture fills the central 32×32 (some spill
/// outside it), on a transparent background.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct Icon(pub String);

/// The base shape of a token, seen from above.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TokenShape {
    /// Becomes a cylinder. Used for the player.
    #[default]
    Circle,
    Triangle,
    Square,
    Hexagon,
    Octagon,
}

impl TokenShape {
    pub const ALL: [Self; 5] = [
        Self::Circle,
        Self::Triangle,
        Self::Square,
        Self::Hexagon,
        Self::Octagon,
    ];

    /// The next shape in `ALL`, wrapping round (for demos).
    pub fn next(self) -> Self {
        Self::ALL[(self as usize + 1) % Self::ALL.len()]
    }
}

/// How an entity feels about the player, shown as the token's colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Attitude {
    /// No attitude (e.g. the player's own token): white.
    #[default]
    None,
    /// Green.
    Friendly,
    /// Blue.
    Neutral,
    /// Red.
    Unfriendly,
}

impl Attitude {
    pub const ALL: [Self; 4] = [Self::None, Self::Friendly, Self::Neutral, Self::Unfriendly];

    /// The next attitude in `ALL`, wrapping round (for demos).
    pub fn next(self) -> Self {
        Self::ALL[(self as usize + 1) % Self::ALL.len()]
    }

    /// The colour every view uses for this attitude.
    pub fn colour(self) -> Color {
        match self {
            Self::None => Color::WHITE,
            Self::Friendly => Color::srgb_u8(0x33, 0xcc, 0x33),
            Self::Neutral => Color::srgb_u8(0x33, 0x88, 0xff),
            Self::Unfriendly => Color::srgb_u8(0xee, 0x33, 0x33),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_up_follows_the_view() {
        // Up the screen, for each way the view can face.
        let up: Vec<_> = (0..4)
            .map(|q| ViewFacing(q).screen_to_grid(0, -1))
            .collect();
        assert_eq!(up, [(0, -1), (1, 0), (0, 1), (-1, 0)]);
        // Right on screen when east is at the top: south.
        assert_eq!(ViewFacing(1).screen_to_grid(1, 0), (0, 1));
    }
}
