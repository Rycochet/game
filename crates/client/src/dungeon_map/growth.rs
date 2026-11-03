//! Building crops and herbs again when what is seen changes.

use bevy::prelude::*;

use super::{CropPart, HerbPart, Shown};
use crate::common::{CurrentDungeon, GameClock, HerbGrowthShift, WorldReturn};

/// Builds the crops again when what is seen changes: never for the camera, as
/// fields can hold many more faces than everything else put together.
pub(super) fn update_crops(
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    part: Single<(&Mesh3d, &mut Visibility), With<CropPart>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut built: Local<u64>,
) {
    let Some(sight) = shown.sight.as_ref().filter(|_| *built != shown.looks) else {
        return;
    };
    *built = shown.looks;
    let ruined = dungeon.info.as_ref().is_some_and(|info| info.ruined);
    let crops = super::field::build_crops(&dungeon.fields, sight, &dungeon.remembered, ruined);
    let (handle, mut visibility) = part.into_inner();
    let any = !crops.indices.is_empty();
    visibility.set_if_neq(if any {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if any && let Some(mut stored) = meshes.get_mut(&handle.0) {
        *stored = crops.into_mesh();
    }
}

/// Builds what grows in a herbalist's patch again when what is seen changes,
/// or an hour passes: some tile may move on a stage in any of them.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_herbs(
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    clock: Res<GameClock>,
    shift: Res<HerbGrowthShift>,
    place: Res<WorldReturn>,
    part: Single<(&Mesh3d, &mut Visibility), With<HerbPart>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut built: Local<(u64, u32)>,
) {
    let grown = super::herbs::growth(clock.hour, shift.0);
    let now = (shown.looks, grown);
    let Some(sight) = shown.sight.as_ref().filter(|_| *built != now) else {
        return;
    };
    *built = now;
    let at = (place.0.x, place.0.y);
    let plants = super::herbs::build_plants(&dungeon.level, sight, &dungeon.remembered, at, grown);
    let (handle, mut visibility) = part.into_inner();
    let any = !plants.indices.is_empty();
    visibility.set_if_neq(if any {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if any && let Some(mut stored) = meshes.get_mut(&handle.0) {
        *stored = plants.into_mesh();
    }
}
