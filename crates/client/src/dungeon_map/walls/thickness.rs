//! How thick a wall is at each height, and its shading by height.

use bevy::prelude::*;
use shared::dungeon::CELL_UNITS;

use super::super::theme::Shape;
use super::super::{CELL_SIZE, WALL_HEIGHT};
use super::{
    HALF_CELL, NATURAL_BANDS, NATURAL_HIGH_SHADE, NATURAL_LOW_SHADE, NATURAL_WALL_HALF, WALL_HALF,
};

pub(super) fn units_to_size(units: i64) -> f32 {
    units as f32 / CELL_UNITS as f32 * CELL_SIZE
}

/// Half a wall's thickness at a height.
pub fn half_thickness(shape: Shape, height: f32) -> f32 {
    match shape {
        Shape::Built => WALL_HALF,
        Shape::Natural => {
            let from_middle = 2.0 * height / WALL_HEIGHT - 1.0;
            WALL_HALF + (NATURAL_WALL_HALF - WALL_HALF) * from_middle * from_middle
        }
    }
}

/// How far a wall goes towards a side at a height, given how far it goes at
/// its narrowest (`reach`). It goes no further than the edge of its cell.
pub(super) fn reach_at(shape: Shape, reach: f32, height: f32) -> f32 {
    if reach >= HALF_CELL {
        HALF_CELL
    } else {
        reach + half_thickness(shape, height) - WALL_HALF
    }
}

/// How a wall is lit at a height, besides the light reaching it.
pub(super) fn height_shade(shape: Shape, height: f32) -> f32 {
    match shape {
        Shape::Built => 1.0,
        Shape::Natural => {
            let up = height / WALL_HEIGHT;
            NATURAL_LOW_SHADE + (NATURAL_HIGH_SHADE - NATURAL_LOW_SHADE) * up
        }
    }
}

/// The heights a face is cut at, from the floor up, between two heights.
pub(super) fn heights(shape: Shape, low: f32, high: f32) -> Vec<f32> {
    let bands = match shape {
        Shape::Built => 1,
        Shape::Natural => NATURAL_BANDS,
    };
    (0..=bands)
        .map(|i| low + (high - low) * i as f32 / bands as f32)
        .collect()
}
