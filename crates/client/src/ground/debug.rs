//! Looking at the bags (`dev_tools`).

use bevy::prelude::*;
use shared::ground::Load;

use super::{BagAssets, Bags, set_bag};
use crate::common::MapPosition;
use crate::player::Player;

/// For looking at the bags (`dev_tools`).
#[derive(Message, Clone, Copy, Debug)]
pub enum GroundDebug {
    /// A row of every look, starting from the player's cell and going east.
    ShowLooks,
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn run_debug(
    mut asked: MessageReader<GroundDebug>,
    assets: Res<BagAssets>,
    player: Single<&MapPosition, With<Player>>,
    mut bags: ResMut<Bags>,
    mut commands: Commands,
) {
    for GroundDebug::ShowLooks in asked.read() {
        for (index, load) in Load::ALL.into_iter().enumerate() {
            let cell = (player.x + index as i64, player.y);
            set_bag(&mut bags, &mut commands, &assets, cell, load);
        }
    }
}
