//! Placeholder 3D models of world map pois, built from simple shapes to match
//! what the poi leads to (`PoiLook`): a building's outline, size and floors, a
//! town's number of buildings. The same triangles make the `Mesh` on the world
//! map and, projected isometrically, its picture on the maps (`picture`).
//!
//! Units are world grid points: `x` east, `y` up, `z` south, with the ground at
//! `y` 0 and the poi at the origin. Every model stays within `REACH` of its
//! centre, the size of the base it replaced.

// Split into files (`cargo xtask lint` keeps files small).
mod base;
mod build;
mod building;
mod model;
mod oasis;
mod output;
mod shapes;
mod underground;
mod wall;

pub use base::base_mesh;
pub use build::build;
use building::{building, ruined};
pub use model::PoiModel;
use oasis::oasis;
use shapes::{circle, edge, half_size, hashed, rectangle, ring};
use underground::underground;

/// How far a model reaches from its centre.
const REACH: f32 = 8.0;

/// The flat base under a poi: its sides, its rim's width, and its colour,
/// which undoes the sun's dimming of flat ground so it shows white.
const BASE_ROUND_SIDES: usize = 32;
const BASE_RIM: f32 = 0.5;
const BASE_WHITE: f32 = 1.41;

/// How big a farmhouse is on its model, against a building alone: its fields
/// have to fit round it.
const FARMHOUSE_SHARE: f32 = 0.55;

/// How far the hovered outline stands off a model.
const OUTLINE_WIDTH: f32 = 0.5;

/// How much of a picture's edge is left clear on each side, as a share of its
/// size, for the border the maps draw round it.
const PICTURE_MARGIN: f32 = 0.06;

/// How wide each piece of a ruined building's wall is.
const WALL_STONE: f32 = 1.2;

/// One floor's height.
const FLOOR: f32 = 2.2;

/// The longest side of a building that still grows its model: anything longer
/// is drawn at `REACH`.
const CELLS_AT_REACH: f32 = 50.0;

/// A picture shows this many grid points across, or the whole model if bigger,
/// so a hut looks smaller than a castle.
const PICTURE_SPAN: f32 = 18.0;

/// Sunlight on a picture's faces, and the light they always get.
const PICTURE_LIGHT: [f32; 3] = [-0.45, 0.8, 0.4];
const PICTURE_AMBIENT: f32 = 0.45;

type Rgb = [u8; 3];

const EARTH: Rgb = [112, 92, 66];
const WOOD: Rgb = [122, 86, 52];
const THATCH: Rgb = [186, 158, 88];
const STONE: Rgb = [146, 144, 138];
const DARK_STONE: Rgb = [98, 98, 100];
const ADOBE: Rgb = [204, 164, 112];
const PLASTER: Rgb = [216, 204, 178];
const RED_ROOF: Rgb = [168, 62, 46];
const GREY_ROOF: Rgb = [92, 96, 106];
const SNOW: Rgb = [232, 238, 244];
const ICE: Rgb = [176, 214, 232];
const MOSS: Rgb = [94, 124, 70];
const BLOOD: Rgb = [120, 30, 30];
const SAND: Rgb = [214, 186, 122];
const GRASS: Rgb = [108, 138, 72];
const ROCK: Rgb = [118, 112, 104];
const DARK: Rgb = [20, 18, 18];

#[cfg(test)]
mod tests;
