//! Building the meshes again when anyone who sees or gives light moves.

use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::dungeon::Sight;
use shared::poi::PoiKind;

use super::daylight::{OutsideLight, Spill, TownLights};
use super::mesh::View;
use super::sight_lines::SightLines;
use super::theme::Theme;
use super::{MemoryPart, SeeThroughPart, Shown, SolidPart, StairsThroughPart};
use crate::common::{
    CurrentDungeon, LightSource, MapPosition, Preferences, Remembered, SharesSight, ViewStats,
};
use crate::player::Player;
use crate::view3d::ViewCamera;

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
    (outside, mut spill, preferences, mut settings_seen): (
        Res<OutsideLight>,
        Local<Spill>,
        Res<Preferences>,
        Local<Option<(u8, bool)>>,
    ),
    (town_lights, mut window_glow, lines, mut lines_seen): (
        Res<TownLights>,
        Local<Spill>,
        Res<SightLines>,
        Local<u64>,
    ),
) {
    let camera = camera.translation;
    let video = &preferences.video;
    let settings = (video.brightness, video.remembered != Remembered::Hide);
    let looked_again = dungeon.is_changed()
        || player.is_changed()
        || sharers.iter().any(|position| position.is_changed());
    let changed = *built_for != Some(camera)
        || looked_again
        || outside.is_changed()
        || town_lights.is_changed()
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
        let views: Vec<View> = std::iter::once(&**player)
            .chain(sharers.iter().map(|position| position.into_inner()))
            .map(|position| {
                let mut sight = Sight::new(level);
                sight.add_glimpses(level, position.x, position.y, None);
                View {
                    cell: (position.x, position.y),
                    sight,
                }
            })
            .collect();
        let mut sight = Sight::new(level);
        for view in &views {
            sight.add_glimpses(level, view.cell.0, view.cell.1, None);
        }
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
            i.kind,
            PoiKind::Farm | PoiKind::HerbalistHut | PoiKind::StiltHut | PoiKind::GreenStiltHut
        )
    });
    if !is_town && (dungeon.is_changed() || outside.is_changed()) {
        *spill = Spill::work_out(level, &dungeon.stairs, outside.travel, open_air);
    }
    let mut light =
        super::daylight::light_from_outside(level, &spill, outside.light, is_town, open_air);
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
    light.raise_to(video.least_light());
    let ruined = dungeon.info.as_ref().is_some_and(|info| info.ruined);
    let theme = Theme::of(dungeon.info.as_ref().map(|info| info.kind));
    let built = super::mesh::build(
        level,
        &views,
        &drawn,
        &remembered,
        &light,
        &dungeon.stairs,
        &dungeon.buildings,
        &town_lights.0,
        &dungeon.fields,
        ruined,
        camera,
        theme.shape,
        &cut,
    );
    shown.sight = Some(sight.clone());
    dungeon.bypass_change_detection().remembered = remembered;
    shown.solid_faces = built.solid_faces;

    // An empty mesh is hidden rather than drawn with no triangles.
    let shown_if = |has_faces: bool| {
        if has_faces {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    let mut solid_meshes: Vec<Option<Mesh>> = built.solid.into_iter().map(Some).collect();
    for (part, handle, mut visibility) in &mut solid {
        let i = part.0 as usize;
        visibility.set_if_neq(shown_if(built.has_solid[i]));
        // An empty mesh is left as it was (hidden): Bevy warns about meshes
        // with no vertices.
        if !built.has_solid[i] {
            continue;
        }
        if let (Some(mut stored), Some(mesh)) = (meshes.get_mut(&handle.0), solid_meshes[i].take())
        {
            *stored = mesh;
        }
    }
    let mut memory_meshes: Vec<Option<Mesh>> = built.memory.into_iter().map(Some).collect();
    for (part, handle, mut visibility) in &mut memory {
        let i = part.0 as usize;
        visibility.set_if_neq(shown_if(built.has_memory[i] && settings.1));
        if !built.has_memory[i] {
            continue;
        }
        if let (Some(mut stored), Some(mesh)) = (meshes.get_mut(&handle.0), memory_meshes[i].take())
        {
            *stored = mesh;
        }
    }
    let mut through_meshes: Vec<Option<Mesh>> =
        built.stairs_through.into_iter().map(Some).collect();
    for (part, handle, mut visibility) in &mut stairs_through {
        let i = part.0 as usize;
        visibility.set_if_neq(shown_if(built.has_stairs_through[i]));
        if !built.has_stairs_through[i] {
            continue;
        }
        if let (Some(mut stored), Some(mesh)) =
            (meshes.get_mut(&handle.0), through_meshes[i].take())
        {
            *stored = mesh;
        }
    }
    let (see_through_mesh, mut see_through_visibility) = see_through.into_inner();
    see_through_visibility.set_if_neq(shown_if(built.has_see_through));
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
