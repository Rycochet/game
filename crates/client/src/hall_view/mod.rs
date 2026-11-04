//! The Hall of Worlds' own looks, over the dungeon view that draws its floor:
//! the World Gates' models, with their codes in runes and water running down
//! their openings. `GateModel` is for World Gates on the world map too. See `docs/context/client/hall_view.md`.

mod gate;
mod place;
mod sight;

use bevy::prelude::*;

use shared::protocol::LevelKind;

use crate::common::{CurrentDungeon, GameView};
use crate::view3d::View3dSystems;
pub use gate::GateModel;

pub struct HallViewPlugin;

impl Plugin for HallViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<place::Gates>()
            .add_systems(
                Update,
                (place::place_gates, place::run_water)
                    .chain()
                    .run_if(in_state(GameView::Dungeon)),
            )
            .add_systems(OnExit(GameView::Dungeon), place::remove_gates)
            .add_systems(Update, sight::hall_reach.after(View3dSystems::Place));
    }
}

/// Whether the player is in the Hall of Worlds, looking at it.
fn in_hall(dungeon: &CurrentDungeon, view: GameView) -> bool {
    view == GameView::Dungeon
        && dungeon
            .info
            .as_ref()
            .is_some_and(|info| matches!(info.kind, LevelKind::Hall { .. }))
}
