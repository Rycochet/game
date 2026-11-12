//! Tokens: how entities with a `common::Token` are drawn in the 3D views.
//!
//! Each token is a flat prism (its shape extruded upwards) in its attitude's
//! colour, with a black outline, standing on the ground at its owner's exact
//! position. Above it sits an icon anchor that always faces the camera,
//! showing the entity's 2D `Icon` if it has one. What is hidden behind
//! something else shows through it in the token's colour (`xray`).
//!
//! The drawn parts live on a separate "view" entity rather than on the owner,
//! so game entities don't carry rendering components.

// Split into files (`cargo xtask lint` keeps files small).
mod assets;
mod flash;
mod parts;
mod pick;
mod sync;
mod xray;

pub use assets::TokenAssets;
use assets::{base_radius, outline_width, prism, raised_prism};
pub use flash::{TokenFlash, flash_tokens};
pub use parts::{IconAnchor, TokenView};
pub use pick::{ClickedToken, HoveredToken, find_token_under_cursor};
pub use sync::{resize_tokens, sync_icons, sync_tokens};
pub use xray::{XRAY_WHITE, XrayMaterial};

/// Token width in grid units: the diameter of the circle every shape fits in.
pub const DIAMETER: f32 = 16.0;

/// Token thickness, as a fraction of the diameter.
const THICKNESS: f32 = 0.15;

/// Half the token's thickness: how far its centre sits above the ground.
pub const HALF_THICKNESS: f32 = DIAMETER * THICKNESS / 2.0;

/// Outline width on screen.
const OUTLINE_PIXELS: f32 = 2.0;

/// A completed token's brightness (0.7: about 30% darker).
const COMPLETED_SHADE: f32 = 0.7;

/// A completed token's opacity. Alpha to coverage only has coarse steps; 0.9
/// looks about half as see-through as 0.7 did.
const COMPLETED_OPACITY: f32 = 0.9;

/// Icon files are this many pixels square...
const ICON_FILE_PIXELS: f32 = 54.0;

/// ...with the main picture in the central square this size.
const ICON_ART_PIXELS: f32 = 32.0;

/// Width of that central square in grid units, at `TokenSize::Default` (each
/// token's size scales it). Icons scale with the map, like the token itself,
/// so their pixels aren't whole screen pixels (a blockier, retro look).
const ICON_ART_SIZE: f32 = DIAMETER * 1.5;

/// Gap, in icon pixels, between the top of the token and the bottom of the
/// icon's main picture.
const ICON_GAP_PIXELS: f32 = 1.0;

/// Most pictures end in a shadow a few art pixels above the bottom of the
/// central square; the icon is lowered by this many art pixels so characters
/// stand on the token rather than float above it.
const ICON_SHADOW_PIXELS: f32 = 3.0;

/// Grid units per icon art pixel, at `TokenSize::Default`.
const ICON_PIXEL: f32 = ICON_ART_SIZE / ICON_ART_PIXELS;
