//! Stairs in 3D, one model for each `StairsStyle`. Stairs up stand on the
//! floor. Stairs down are the same model sunk into a hole (a shaft with a
//! solid black base) so that its top meets the floor, with handrails and the
//! tops of ladders coming up above it.
//!
//! Everything is unlit, so faces are shaded by hand from the way they face.

use bevy::prelude::*;

use super::{CELL_SIZE, WALL_HEIGHT};

// Split into files (`cargo xtask lint` keeps files small).
mod build;
mod floor;
mod hides;
mod ladders;
mod model;
mod ropes;
mod shaft;
mod steps;

use build::Hole;
pub use build::build_stairs;
use floor::{floor_uv, floor_whole, floor_with_hole, hole_outline, light_under_ladder};
pub use hides::hides_viewer;
use model::Model;
use shaft::{facing_shade, front_tint, shaft};

/// How deep the hole for stairs down is, and so how high their model is.
const STAIRS_DOWN_DEPTH: f32 = WALL_HEIGHT * 0.75;
/// How high the model is going up: three times that, so stairs climb at a
/// believable rate, cut off at `STAIRS_UP_TOP`.
const STAIRS_UP_HEIGHT: f32 = STAIRS_DOWN_DEPTH * 3.0;
/// The same for turning stairs, whose flights are short and need less.
const TURNING_UP_HEIGHT: f32 = STAIRS_DOWN_DEPTH * 2.25;
/// Where models going up are cut off: a tile and a half.
const STAIRS_UP_TOP: f32 = CELL_SIZE * 1.5;
/// How much further apart rungs and knots are going up.
const UP_SPACING: f32 = 1.5;

/// How tall a viewer's token is taken to be, when working out whether stairs
/// stand in the way of seeing it.
const VIEWER_HEIGHT: f32 = CELL_SIZE;

/// Half the width of the holes and models: floor is left all round them.
const SQUARE_HALF: f32 = CELL_SIZE * 0.4;
const ROUND_RADIUS: f32 = CELL_SIZE * 0.4;
/// The gap between a spiral's steps and the side of its hole.
const SPIRAL_GAP: f32 = 0.3;

/// Sides of a round hole: a multiple of 8, so some point at the cell's corners.
const ROUND_SIDES: usize = 32;

/// How far above its steps a handrail runs, and how thick it is.
const RAIL_HEIGHT: f32 = 9.0;
const RAIL_RADIUS: f32 = 0.3;
/// How far outside the hole the rail round its top stands.
const RIM_RAIL_OUTSIDE: f32 = 0.3;
/// Pieces of the rail round the top of a round hole, and a post every so many.
const RIM_RAIL_PIECES: usize = 16;
const RIM_POST_EVERY: usize = 4;
const POST_RADIUS: f32 = 0.22;

const SPIRAL_STEPS: usize = 9;
const SPIRAL_TURNS: f32 = 1.5;
/// Pieces each step's outer edge and tread are cut into, to follow the circle.
const SPIRAL_STEP_PIECES: usize = 4;
const COLUMN_RADIUS: f32 = 1.5;
/// How far a step's sides reach below the tread of the step before.
const SPIRAL_STEP_OVERLAP: f32 = 0.4;

/// Steps in each of the two flights of turning stairs.
const FLIGHT_STEPS: usize = 2;
/// How much of the model's depth the landing takes.
const LANDING_DEPTH: f32 = SQUARE_HALF * 0.5;

const LADDER_HALF_WIDTH: f32 = 2.4;
const LADDER_POLE_RADIUS: f32 = 0.4;
const LADDER_RUNG_RADIUS: f32 = 0.28;
const RUNG_SPACING: f32 = 4.2;
/// How far the tops of ladders come up above the floor, going down.
const LADDER_ABOVE_FLOOR: f32 = 4.0;
/// How far a leaning ladder reaches from the middle, where it meets the floor
/// (it rests just inside the edge of the hole).
const LEAN_REACH: f32 = ROUND_RADIUS * 0.9;

const LOG_RADIUS: f32 = 1.1;
const LOG_RUNG_LENGTH: f32 = 2.6;

/// How thick ropes are, and how far a rope ladder's rungs sag.
const ROPE_RADIUS: f32 = 0.3;
const ROPE_SAG: f32 = 0.7;
const SLAT_RADIUS: f32 = 0.5;
const KNOT_RADIUS: f32 = 0.85;
const KNOT_SPACING: f32 = 4.5;
/// How far a knot reaches up and down from its middle.
const KNOT_HALF_HEIGHT: f32 = 0.3;
/// The curl of rope left on the floor: how far its middle is from the rope,
/// how tight it winds to, how many turns and pieces, and its height.
const CURL_RADIUS: f32 = 1.8;
const CURL_INNER_RADIUS: f32 = 0.5;
const CURL_TURNS: f32 = 0.875;
const CURL_PIECES: usize = 8;
const CURL_HEIGHT: f32 = 0.3;
/// A hanging rope's way over the edge of the hole: outwards from its side
/// and up from the floor, ending on the floor at a peg.
const ROPE_OVER_EDGE: [(f32, f32); 4] = [(0.0, 3.6), (0.4, 4.4), (1.2, 3.4), (1.6, 0.15)];
const ROPE_INSIDE: f32 = 0.7;
const PEG_HEIGHT: f32 = 2.2;
const PEG_RADIUS: f32 = 0.4;

/// Rope is the wood's picture made paler (vertex colours multiply it).
const ROPE_TINT: Vec3 = Vec3::new(2.8, 2.35, 1.7);
/// A spot on the wood's picture away from the dark lines, for rope.
const ROPE_UV: [f32; 2] = [0.5, 2.5 / 32.0];
/// One plank of the wood's picture, for bars, which run along its grain:
/// from `PLANK_U.0`, and across `PLANK_V`.
const PLANK_U: (f32, f32) = (12.1 / 32.0, 31.9 / 32.0);
const PLANK_V: (f32, f32) = (1.1 / 32.0, 3.9 / 32.0);

/// How much darker the outer sides of steps are than their treads, and how
/// bright their fronts are compared with their treads.
const RISER_SHADE: f32 = 0.6;
const FRONT_SHADE: f32 = 0.45;
/// Treads are the floor's picture made brighter, to stand out from the floor.
const TREAD_TINT: f32 = 1.35;
/// The circle of floor under a ladder going up, the size of the hole it comes
/// through, is the floor's picture made this much brighter, to show it is a way out.
const LADDER_LIGHT_TINT: f32 = 1.6;

/// Inside the hole: how bright the side is at the top and the bottom, and how
/// bright the model is at the bottom (it is full bright at the floor). Vertex
/// colours are linear, so these look much brighter on screen than they read:
/// 0.03 shows as about 20% of full.
const RIM_SHADE: f32 = 0.6;
const DEEPEST_SHADE: f32 = 0.02;
const MODEL_DEEPEST_SHADE: f32 = 0.03;

#[cfg(test)]
mod tests;
