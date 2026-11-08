//! Bolts of lightning in the wilderness, where the server's spots (in its
//! cells) are the level's own: drawn on outside ground the player can see.
//! Other sites aren't laid out on the world map, so they only flash.

use bevy::prelude::*;

use super::CELL_SIZE;
use super::parts::{DungeonScene, Shown};
use crate::common::{BoltStruck, CurrentDungeon};
use crate::lightning::spawn_bolt;

/// How tall a bolt is, and how wide its main stroke, against a cell.
const BOLT_HEIGHT: f32 = CELL_SIZE * 10.0;
const BOLT_WIDTH: f32 = CELL_SIZE * 0.3;

/// Draws each bolt that strikes outside ground in sight in the wilderness.
pub(super) fn strike(
    mut struck: MessageReader<BoltStruck>,
    mut commands: Commands,
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for BoltStruck(strike) in struck.read() {
        let (x, y) = (strike.x, strike.y);
        let seen = shown
            .sight
            .as_ref()
            .is_some_and(|sight| sight.is_seen(x, y));
        if !dungeon.is_wilderness() || !seen || !dungeon.level.cell(x, y).is_outside() {
            continue;
        }
        let at = Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE);
        let bolt = spawn_bolt(
            &mut commands,
            &mut meshes,
            &mut materials,
            strike.seed,
            at,
            BOLT_HEIGHT,
            BOLT_WIDTH,
        );
        commands.entity(bolt).insert(DungeonScene);
    }
}
