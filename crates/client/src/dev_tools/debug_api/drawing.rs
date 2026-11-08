//! How stairs, fields, herbs and the player's token are drawn.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params, refused};
use shared::site::StairsStyle;

use super::super::{Forced, token_of};
use super::done;
use crate::common::{HerbGrowthShift, Token};
use crate::player::Player;

#[derive(Deserialize)]
pub(super) struct StyleParams {
    /// An index into `StairsStyle::ALL` (0 spiral ... 6 knotted rope), or
    /// none to draw stairs as they are.
    style: Option<usize>,
}

/// `client.stairs_style { "style": 2 }` (`HOW_DEBUG_STAIRS`).
pub(super) fn stairs_style(
    In(request): In<Option<Value>>,
    mut forced: ResMut<Forced>,
) -> BrpResult {
    let StyleParams { style } = params(request)?;
    forced.stairs = match style {
        Some(index) => Some(*StairsStyle::ALL.get(index).ok_or_else(|| {
            refused(format!(
                "styles go from 0 to {}",
                StairsStyle::ALL.len() - 1
            ))
        })?),
        None => None,
    };
    done()
}

#[derive(Deserialize)]
pub(super) struct StageParams {
    /// A field's `variety`, or none to draw fields as they are.
    stage: Option<u32>,
}

/// `client.fields_stage { "stage": 4 }` (`HOW_DEBUG_FIELDS`).
pub(super) fn fields_stage(
    In(request): In<Option<Value>>,
    mut forced: ResMut<Forced>,
) -> BrpResult {
    let StageParams { stage } = params(request)?;
    forced.fields = stage;
    done()
}

#[derive(Deserialize)]
pub(super) struct StagesParams {
    stages: u32,
}

/// `client.herbs_shift { "stages": 3 }`: herbs drawn this many game days on
/// (`HOW_DEBUG_HERBS`).
pub(super) fn herbs_shift(
    In(request): In<Option<Value>>,
    mut shift: ResMut<HerbGrowthShift>,
) -> BrpResult {
    let StagesParams { stages } = params(request)?;
    shift.0 = stages;
    done()
}

#[derive(Deserialize)]
pub(super) struct TokenParams {
    /// Indexes into `TokenShape::ALL`, `Attitude::ALL` and `TokenSize::ALL`.
    shape: usize,
    attitude: usize,
    #[serde(default)]
    completed: bool,
    size: Option<usize>,
}

/// `client.token { "shape": 3, "attitude": 3 }`: the player's token
/// (`HOW_DEBUG_TOKEN`).
pub(super) fn token(
    In(request): In<Option<Value>>,
    mut token: Single<&mut Token, With<Player>>,
) -> BrpResult {
    let TokenParams {
        shape,
        attitude,
        completed,
        size,
    } = params(request)?;
    **token = token_of(shape, attitude, completed, size)
        .ok_or_else(|| refused("no such shape, attitude or size"))?;
    done()
}

#[derive(Deserialize)]
pub(super) struct HeightStepParams {
    step: f32,
}

/// `tmp.height_step { "step": 0.25 }`: cells drawn per 16th of an elevation
/// in the wilderness, for choosing it.
pub(super) fn height_step(
    In(request): In<Option<Value>>,
    mut heights: ResMut<crate::common::WildCellHeights>,
    player: Single<(&crate::common::MapPosition, &crate::common::SmoothPosition), With<Player>>,
) -> BrpResult {
    let HeightStepParams { step } = params(request)?;
    if step >= 0.0 {
        heights.drawn_step = step;
    }
    let (cell, smooth) = *player;
    let around: Vec<Vec<f32>> = (-1..=1)
        .map(|dy| {
            (-1..=1)
                .map(|dx| heights.drawn(cell.x + dx, cell.y + dy))
                .collect()
        })
        .collect();
    let between = heights.drawn_between(smooth.x as f32, smooth.y as f32);
    Ok(
        server::debug_api::json!({"cell": [cell.x, cell.y], "smooth": [smooth.x, smooth.y], "around": around, "between": between}),
    )
}
