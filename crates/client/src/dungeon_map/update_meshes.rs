//! Building the meshes again when anyone who sees or gives light moves.

use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::dungeon::Sight;
use shared::facing::{Field, Heading};
use shared::poi::PoiKind;
use shared::protocol::LevelKind;

use super::daylight::{OutsideLight, Spill, TownLights};
use super::door_swing::DoorSwings;
use super::mesh::{LightGrid, View, build_posed};
use super::put_meshes::put_meshes;
use super::sight_lines::SightLines;
use super::theme::Theme;
use super::viewers;
use super::wild_ground::GroundTint;
use super::{MemoryPart, SeeThroughPart, Shown, SolidPart, StairsThroughPart};
use crate::common::{
    CurrentDungeon, DrawnGround, LightSource, Lightning, MapPosition, PlayerCone, Preferences,
    Remembered, SharesSight, ViewStats,
};
use crate::player::Player;
use crate::view3d::ViewCamera;

/// The Hall of Worlds' own light, everywhere in it.
const HALL_LIGHT: f32 = 0.85;
/// When anyone who sees or gives light moves, the level or the light outside
/// changes, or the camera moves (turning, zooming or following the player): works out what
/// is seen (and so remembered from now on) and how it is lit, and rebuilds
/// the meshes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn update_meshes(
    mut dungeon: ResMut<CurrentDungeon>,
    player: Single<Ref<MapPosition>, With<Player>>,
    sharers: Query<Ref<MapPosition>, (With<SharesSight>, Without<Player>)>,
    lights: Query<(Ref<MapPosition>, Ref<LightSource>)>,
    camera: Single<&Transform, With<ViewCamera>>,
    mut solid: Query<
        (&SolidPart, &Mesh3d, &mut Visibility),
        (Without<SeeThroughPart>, Without<MemoryPart>),
    >,
    mut memory: Query<
        (&MemoryPart, &Mesh3d, &mut Visibility),
        (Without<SeeThroughPart>, Without<SolidPart>),
    >,
    see_through: Single<(&Mesh3d, &mut Visibility), With<SeeThroughPart>>,
    mut stairs_through: Query<
        (&StairsThroughPart, &Mesh3d, &mut Visibility),
        (
            Without<SeeThroughPart>,
            Without<SolidPart>,
            Without<MemoryPart>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stats: ResMut<ViewStats>,
    mut shown: ResMut<Shown>,
    // The camera position last built for.
    mut built_for: Local<Option<Vec3>>,
    // What the viewers saw when last worked out, kept while only the camera
    // moves.
    mut seen: Local<Option<(Vec<View>, Sight)>>,
    // Together, as Bevy limits how many things a system can ask for.
    // With the settings: Brightness and whether remembered places show, as
    // last built.
    (outside, mut spill, preferences, mut settings_seen, swings): (
        Res<OutsideLight>,
        Local<Spill>,
        Res<Preferences>,
        Local<Option<(u8, bool)>>,
        Res<DoorSwings>,
    ),
    // With the player's field of view, and the way it faced when last worked out.
    (
        town_lights,
        mut window_glow,
        lines,
        mut lines_seen,
        tint,
        lightning,
        heights,
        cone,
        mut cone_seen,
    ): (
        Res<TownLights>,
        Local<Spill>,
        Res<SightLines>,
        Local<u64>,
        Res<GroundTint>,
        Res<Lightning>,
        Res<DrawnGround>,
        Res<PlayerCone>,
        Local<Option<(Heading, Field)>>,
    ),
) {
    let camera = camera.translation;
    let graphics = &preferences.graphics;
    let settings = (graphics.brightness, graphics.remembered != Remembered::Hide);
    let facing = cone.0.map(|cone| (cone.heading, cone.field));
    let turned = *cone_seen != facing;
    *cone_seen = facing;
    let looked_again = dungeon.is_changed()
        || turned
        || player.is_changed()
        || sharers.iter().any(|position| position.is_changed());
    let changed = *built_for != Some(camera)
        || looked_again
        || outside.is_changed()
        || lightning.is_changed()
        || town_lights.is_changed()
        || tint.is_changed()
        || heights.is_changed()
        || swings.is_changed()
        || *lines_seen != lines.changes
        || *settings_seen != Some(settings)
        || lights
            .iter()
            .any(|(position, light)| position.is_changed() || light.is_changed());
    if !changed {
        return;
    }
    *built_for = Some(camera);
    *lines_seen = lines.changes;
    *settings_seen = Some(settings);

    let start = Instant::now();
    let level = &dungeon.level;
    if looked_again || seen.is_none() {
        let player_at = (player.x, player.y);
        let sharing = sharers.iter().map(|position| (position.x, position.y));
        let (views, sight) = viewers::views(level, player_at, facing, sharing);
        *seen = Some((views, sight));
        shown.looks += 1;
    }
    let Some((views, sight)) = &*seen else {
        return;
    };
    // Drawn as in sight: what the game's sight says, and anything the real
    // lines reach into, which can be a little more. The mask cuts them to shape.
    let mut drawn = sight.clone();
    // Each viewer sees what its own lines reach too, or a wall they reach would
    // be drawn as memory, which the mask hides inside its edge.
    let views: Vec<View> = views
        .iter()
        .enumerate()
        .map(|(i, view)| {
            let mut sight = view.sight.clone();
            if let Some(touched) = lines.touched_by.get(i) {
                super::sight_lines::add_cells(level, touched, &mut sight);
            }
            View {
                cell: view.cell,
                sight,
            }
        })
        .collect();
    // What needs a remembered copy for the mask to show: drawn as in sight, but
    // not all of it is.
    let mut cut = Sight::new(level);
    // What is in sight is remembered from now on (only cells the server has
    // sent: the rest are wall here until it does). Not a change that needs
    // anything redrawn, so it doesn't mark the level changed.
    let mut remembered = dungeon.remembered.clone();
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if lines.touched.as_ref().is_some_and(|t| t.is_seen(x, y)) {
                drawn.insert(x, y);
            }
            if drawn.is_seen(x, y)
                && (lines.crossed.as_ref().is_some_and(|c| c.is_seen(x, y))
                    || lines.touched.as_ref().is_none_or(|t| !t.is_seen(x, y)))
            {
                cut.insert(x, y);
            }
            if drawn.is_seen(x, y) && dungeon.known.is_seen(x, y) {
                remembered.insert(x, y);
            }
        }
    }
    // The light from outside, with what spills in kept unless it has changed.
    let is_town = dungeon.is_town();
    // All in the open apart from their buildings, so their water is too.
    let open_air = dungeon.info.as_ref().is_some_and(|i| {
        matches!(
            i.kind.poi(),
            Some(
                PoiKind::Farm | PoiKind::HerbalistHut | PoiKind::StiltHut | PoiKind::GreenStiltHut
            )
        )
    });
    if !is_town && (dungeon.is_changed() || outside.is_changed()) {
        *spill = Spill::work_out(level, &dungeon.stairs, outside.travel, open_air);
    }
    // The Hall of Worlds has no sky, and gives its own even light.
    let in_hall = dungeon
        .info
        .as_ref()
        .is_some_and(|i| matches!(i.kind, LevelKind::Hall { .. }));
    let mut light = if in_hall {
        LightGrid::filled(level, Vec3::splat(HALL_LIGHT))
    } else {
        let flash = lightning.0;
        super::daylight::light_from_outside(level, &spill, &outside, flash, is_town, open_air)
    };
    if is_town {
        if dungeon.is_changed() || town_lights.is_changed() {
            *window_glow = Spill::from_windows(level, &dungeon.buildings, &town_lights.0);
        }
        light.add(super::daylight::WINDOW_LIGHT, |x, y| window_glow.at(x, y));
    }
    for (position, source) in &lights {
        let colour = LinearRgba::from(source.colour);
        light.add_light(
            level,
            (position.x, position.y),
            source.radius,
            Vec3::new(colour.red, colour.green, colour.blue),
        );
    }
    light.raise_to(graphics.least_light());
    let ruined = dungeon.info.as_ref().is_some_and(|info| info.ruined);
    let theme = Theme::of_level(dungeon.info.as_ref().map(|info| info.kind));
    let looks = (
        &dungeon.buildings[..],
        &town_lights.0,
        &dungeon.fields[..],
        ruined,
    );
    let (seen_now, place) = (
        (&views[..], &drawn, &remembered, &cut),
        (camera, theme.shape),
    );
    let (stairs, wild) = (&dungeon.stairs, tint.wild(&heights));
    let built = build_posed(
        level,
        seen_now,
        &light,
        stairs,
        looks,
        place,
        wild,
        &swings.drawn,
    );
    shown.sight = Some(sight.clone());
    dungeon.bypass_change_detection().remembered = remembered;
    (shown.solid_faces, shown.outlined) = (built.solid_faces, Some(built.outlined));

    let solid = solid
        .iter_mut()
        .map(|(part, mesh, shown)| (part.0 as usize, mesh, shown));
    put_meshes(&mut meshes, built.solid, &built.has_solid, true, solid);
    let memory = memory
        .iter_mut()
        .map(|(part, mesh, shown)| (part.0 as usize, mesh, shown));
    let remembered_shown = settings.1 && !in_hall;
    put_meshes(
        &mut meshes,
        built.memory,
        &built.has_memory,
        remembered_shown,
        memory,
    );
    let through = stairs_through.iter_mut();
    let through = through.map(|(part, mesh, shown)| (part.0 as usize, mesh, shown));
    put_meshes(
        &mut meshes,
        built.stairs_through,
        &built.has_stairs_through,
        true,
        through,
    );
    let (see_through_mesh, mut see_through_visibility) = see_through.into_inner();
    see_through_visibility.set_if_neq(if built.has_see_through {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if built.has_see_through
        && let Some(mut stored) = meshes.get_mut(&see_through_mesh.0)
    {
        *stored = built.see_through;
    }
    let ms = start.elapsed().as_secs_f32() * 1000.0;
    let place = dungeon.describe();
    let at_door = dungeon
        .door_facing(player.x, player.y)
        .map_or(String::new(), |b| {
            format!(", at the door of: {}", b.kind.name())
        });
    let outlines = lines.cost_ms;
    stats.summary =
        format!("{place}{at_door}: sight, light and meshes {ms:.2} ms, lines {outlines:.2} ms");
}
