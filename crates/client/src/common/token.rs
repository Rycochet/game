//! How entities are shown on maps: tokens, their sizes, shapes, icons and attitudes.

use bevy::prelude::*;

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
    /// Draws no base: the view shows something else there, such as a world
    /// map poi's model. It can still be clicked.
    pub no_base: bool,
}

/// How big an entity's icon is drawn above its token, relative to the
/// `Default` size. The token's base is the same size whatever this is.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TokenSize {
    /// Half size: world map pois, whose pictures fill the whole tile.
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
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub const ALL: [Self; 6] = [
        Self::Tiny,
        Self::Small,
        Self::Default,
        Self::Large,
        Self::Giant,
        Self::Titan,
    ];

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
