//! Reading positions, cells and tokens from environment variables.

use bevy::prelude::*;
use shared::dungeon::Cell;

use crate::common::{Attitude, Token, TokenShape, TokenSize};

/// `shape,attitude` or `shape,attitude,completed`, all numbers.
pub(super) fn parse_token(text: &str) -> Option<Token> {
    let mut numbers = text.split(',').map(|n| n.trim().parse::<usize>().ok());
    let shape = numbers.next()??;
    let attitude = numbers.next()??;
    let completed = numbers.next().flatten() == Some(1);
    token_of(shape, attitude, completed, numbers.next().flatten())
}

/// A token from indexes into `TokenShape::ALL`, `Attitude::ALL` and
/// `TokenSize::ALL` (`Default` if none).
pub(super) fn token_of(
    shape: usize,
    attitude: usize,
    completed: bool,
    size: Option<usize>,
) -> Option<Token> {
    Some(Token {
        shape: *TokenShape::ALL.get(shape)?,
        attitude: *Attitude::ALL.get(attitude)?,
        completed,
        size: size
            .and_then(|n| TokenSize::ALL.get(n).copied())
            .unwrap_or_default(),
        ..default()
    })
}

/// `dx,dy,kind`, as in `HOW_DEBUG_CELLS`.
pub(super) fn parse_cell(text: &str) -> Option<(i64, i64, Cell)> {
    let (position, kind) = text.rsplit_once(',')?;
    let (dx, dy) = parse_pos(position)?;
    Some((dx, dy, cell_kind(kind)?))
}

/// A kind of cell by the name `HOW_DEBUG_CELLS` uses.
fn cell_kind(name: &str) -> Option<Cell> {
    Some(match name.trim() {
        "floor" => Cell::Floor,
        "wall" => Cell::Wall,
        "door" => Cell::Door,
        "secret" => Cell::SecretDoor,
        "window" => Cell::Window,
        "water" => Cell::Water,
        "lava" => Cell::Lava,
        "hole" => Cell::Hole,
        "ground" => Cell::Ground,
        "covered" => Cell::CoveredGround,
        _ => return None,
    })
}

pub(super) fn parse_pos(text: &str) -> Option<(i64, i64)> {
    let (x, y) = text.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}
