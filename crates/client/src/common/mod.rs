//! Shared building blocks that every other client module may use.
//!
//! Keep this small: only types that genuinely need to be shared between
//! feature modules belong here. See `docs/context/client/common.md`.

use std::collections::BTreeMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use shared::dungeon::{Level, Sight};
use shared::feature::{CHUNK_SIZE, ChunkPos};
use shared::movement::{WorldConditions, WorldGround};
use shared::protocol::{LevelChunk, LevelInfo, SeenFeature, SeenStairs, WorldInfo};
use shared::surface::Surface;
use shared::world::WorldParams;

/// Frame rate cap while the window has focus. Not a twitch game, so this saves
/// GPU and battery; input still wakes the app immediately.
const FOCUSED_FPS: f64 = 30.0;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(WinitSettings {
                focused_mode: UpdateMode::reactive(Duration::from_secs_f64(1.0 / FOCUSED_FPS)),
                unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(1)),
            })
            .init_resource::<CurrentDungeon>()
            .init_resource::<ServerStatus>()
            .init_state::<GameView>()
            .init_resource::<GameClock>()
            .init_resource::<Weather>()
            .init_resource::<ViewStats>()
            .init_resource::<ViewFacing>();
    }
}

/// The world the player is in, its surface generator and the features the
/// server has told the client about. Only exists once the server has said
/// which world it is (`connection`).
#[derive(Resource)]
pub struct CurrentWorld {
    pub params: WorldParams,
    pub surface: Surface,
    /// The ground as far as walking goes (the server's rules).
    pub ground: WorldGround,
    pub features: KnownFeatures,
}

impl CurrentWorld {
    /// The world a server has told the client about.
    pub fn new(info: &WorldInfo) -> Self {
        let params = WorldParams::from_seed(info.terrain_seed);
        let surface = Surface::new(&params);
        Self {
            features: KnownFeatures::new(surface.width()),
            ground: WorldGround::new(&params),
            surface,
            params,
        }
    }

    /// What decides where can be walked now.
    pub fn conditions(&self, clock: &GameClock, weather: &Weather) -> WorldConditions {
        WorldConditions {
            water: self.params.water_level(clock.hour as f64),
            snow_line: weather.snow_line,
        }
    }
}

/// The world map chunks the server has sent, and their features.
pub struct KnownFeatures {
    /// The world's width: `x` wraps at this.
    width: i64,
    /// By chunk, north-south first (so an area's rows can be looked up);
    /// `None` for a chunk known to have no feature.
    chunks: BTreeMap<(i64, i64), Option<SeenFeature>>,
}

impl KnownFeatures {
    fn new(width: i64) -> Self {
        Self {
            width,
            chunks: BTreeMap::new(),
        }
    }

    /// What the server says is in a chunk.
    pub fn insert(&mut self, chunk: ChunkPos, feature: Option<SeenFeature>) {
        self.chunks.insert((chunk.y, chunk.x), feature);
    }

    /// Every known feature whose position is within `min..=max` (world grid
    /// points, in any planet-width frame: the features are given in the
    /// same one), like `shared::feature::FeatureMap::in_area`.
    pub fn in_area(&self, min: (i64, i64), max: (i64, i64)) -> Vec<SeenFeature> {
        let rows =
            (min.1.div_euclid(CHUNK_SIZE), i64::MIN)..=(max.1.div_euclid(CHUNK_SIZE), i64::MAX);
        let mut found = Vec::new();
        for feature in self.chunks.range(rows).filter_map(|(_, feature)| *feature) {
            // Each copy of the planet the area overlaps.
            for copy in min.0.div_euclid(self.width)..=max.0.div_euclid(self.width) {
                let x = feature.x + copy * self.width;
                if (min.0..=max.0).contains(&x) && (min.1..=max.1).contains(&feature.y) {
                    found.push(SeenFeature { x, ..feature });
                }
            }
        }
        found.sort_by_key(|feature| (feature.y, feature.x));
        found
    }
}

/// Whether the client is connected to the server (set by `connection`).
/// While it isn't, the player can't move, and the last view stays on show.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub enum ServerStatus {
    /// Trying to connect (or to connect again after losing the connection).
    #[default]
    Connecting,
    /// Welcomed by the server.
    Connected,
    /// The server won't have this client, and why: no more tries.
    Refused(String),
}

impl ServerStatus {
    pub fn is_connected(&self) -> bool {
        *self == Self::Connected
    }
}

/// Which view is on screen: the world's surface, or a dungeon or building.
/// Each view shows its scene only while it is the current one.
#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameView {
    /// Waiting for the server to say which world this is: nothing to show
    /// yet, and no `CurrentWorld`. Only until the first `Welcome`: losing the
    /// connection later keeps the view (`ServerStatus` says).
    #[default]
    Connecting,
    WorldMap,
    /// Inside a site (a building or cave): `CurrentDungeon`.
    Dungeon,
}

/// The site level the player is on, as far as the server has told: unknown
/// cells are wall until their cells arrive. Its rules (walls block
/// movement, sight and light) live in `shared::dungeon`. Changed by `site`
/// when the server says the player has arrived, or sends cells.
#[derive(Resource)]
pub struct CurrentDungeon {
    /// What the level is; `None` before any has been entered.
    pub info: Option<LevelInfo>,
    /// The cells the server has sent; the rest are wall.
    pub level: Level,
    /// Which cells the server has sent.
    pub known: Sight,
    /// Which cells the player has seen, now or before: drawn even when out
    /// of sight (in grey). Includes what is in sight now.
    pub remembered: Sight,
    /// The level's known stairs.
    pub stairs: Vec<SeenStairs>,
}

impl Default for CurrentDungeon {
    fn default() -> Self {
        let level = Level::new(0, 0, 1, 1);
        Self {
            info: None,
            known: Sight::new(&level),
            remembered: Sight::new(&level),
            level,
            stairs: Vec::new(),
        }
    }
}

impl CurrentDungeon {
    /// Arriving on a level: nothing of it known yet.
    pub fn enter(&mut self, info: LevelInfo) {
        let (min_x, min_y, width, height) = info.bounds;
        self.level = Level::new(min_x, min_y, width, height);
        self.known = Sight::new(&self.level);
        self.remembered = Sight::new(&self.level);
        self.stairs.clear();
        self.info = Some(info);
    }

    /// Cells the server has sent.
    pub fn add(&mut self, chunk: &LevelChunk) {
        for (x, y, cell) in chunk.cells() {
            self.level.set(x, y, cell);
            self.known.insert(x, y);
            if chunk.remembered {
                self.remembered.insert(x, y);
            }
        }
        for stairs in &chunk.stairs {
            if !self.stairs.contains(stairs) {
                self.stairs.push(*stairs);
            }
        }
    }

    /// Where the player is, for the HUD, e.g. "Castle, floor 2 of 3".
    pub fn describe(&self) -> String {
        let Some(info) = &self.info else {
            return "Nowhere".into();
        };
        let what = if info.building { "floor" } else { "level" };
        format!(
            "{:?}, {what} {} of {}",
            info.kind,
            info.index + 1,
            info.levels
        )
    }
}

/// Clicking this entity's token sends the player to it; arriving uses it:
/// entering a world map feature's site. Needs `Clickable`. (Stairs are cells
/// of the level, not entities: see `site`.)
#[derive(Component, Clone, Copy, Debug)]
pub enum Portal {
    Feature(SeenFeature),
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

/// In-game time of day, in whole hours (`0..24`), as the server says
/// (`connection`). Only changes once per hour, so systems can redraw on
/// `is_changed()`.
#[derive(Resource, Default, PartialEq)]
pub struct GameClock {
    pub hour: u32,
}

/// Snow line for normal weather: above every mountain cap, so only the
/// style's `mountain_height` applies.
pub const NORMAL_SNOW_LINE: f64 = shared::world::NORMAL_SNOW_LINE as f64;

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
    /// How big the entity's `Icon` is drawn (the base stays the same).
    pub size: TokenSize,
}

/// How big an entity's icon is drawn above its token, relative to the
/// `Default` size. The token's base is the same size whatever this is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TokenSize {
    /// Half size: world map features, whose pictures fill the whole tile.
    Tiny,
    Small,
    /// Characters such as the player.
    #[default]
    Default,
    Large,
    Giant,
    Titan,
}

impl TokenSize {
    pub const ALL: [Self; 6] = [
        Self::Tiny,
        Self::Small,
        Self::Default,
        Self::Large,
        Self::Giant,
        Self::Titan,
    ];

    /// The next size in `ALL`, wrapping round (for demos).
    pub fn next(self) -> Self {
        Self::ALL[(self as usize + 1) % Self::ALL.len()]
    }

    /// Icon scale relative to `Default`.
    pub fn scale(self) -> f32 {
        match self {
            Self::Tiny => 0.5,
            Self::Small => 0.75,
            Self::Default => 1.0,
            Self::Large => 1.25,
            Self::Giant => 1.5,
            Self::Titan => 2.0,
        }
    }
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
    use shared::feature::FeatureKind;

    use super::*;

    #[test]
    fn known_features_are_found_in_any_frame() {
        let mut known = KnownFeatures::new(1000);
        let chunk = ChunkPos { x: 0, y: 1 };
        let feature = SeenFeature {
            kind: FeatureKind::Crypt,
            chunk,
            x: 10,
            y: 70,
        };
        known.insert(chunk, Some(feature));
        known.insert(ChunkPos { x: 1, y: 1 }, None);
        let found =
            |min, max| -> Vec<i64> { known.in_area(min, max).iter().map(|f| f.x).collect() };
        assert_eq!(found((0, 64), (100, 127)), [10]);
        // From west of the seam, and a whole planet east.
        assert_eq!(found((-50, 0), (50, 100)), [10]);
        assert_eq!(found((900, 0), (1100, 100)), [1010]);
        // Outside the area, or rows that weren't sent.
        assert!(found((20, 0), (500, 100)).is_empty());
        assert!(found((0, 128), (100, 300)).is_empty());
    }

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
