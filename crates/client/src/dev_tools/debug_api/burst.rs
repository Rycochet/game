//! TEMPORARY (`tmp.burst`): a screenshot every frame for a while, with the
//! camera and gate lights logged, to catch the gates' light flickering.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use server::debug_api::{BrpResult, Value, json};
use std::path::PathBuf;

use crate::view3d::{OrbitCamera, ViewCamera};

#[derive(Resource, Default)]
pub(super) struct Burst {
    left: u32,
    done: u32,
    folder: PathBuf,
    shots: bool,
}

pub(super) fn burst(In(request): In<Option<Value>>, mut burst: ResMut<Burst>) -> BrpResult {
    let request = request.unwrap_or_default();
    let frames = request.get("frames").and_then(Value::as_u64).unwrap_or(30) as u32;
    let folder = PathBuf::from(
        request
            .get("folder")
            .and_then(Value::as_str)
            .unwrap_or("target/shots/burst"),
    );
    let folder = std::path::absolute(folder).unwrap();
    let _ = std::fs::create_dir_all(&folder);
    *burst = Burst {
        left: frames,
        done: 0,
        folder,
        shots: request
            .get("shots")
            .and_then(Value::as_bool)
            .unwrap_or(true),
    };
    Ok(json!("ok"))
}

pub(super) fn shoot(
    mut commands: Commands,
    mut burst: ResMut<Burst>,
    camera: Query<
        (
            &OrbitCamera,
            &GlobalTransform,
            &Transform,
            Option<&bevy::light::cluster::Clusters>,
        ),
        With<ViewCamera>,
    >,
    lights: Query<(&PointLight, &GlobalTransform, &ViewVisibility)>,
) {
    if burst.left == 0 {
        return;
    }
    burst.left -= 1;
    let n = burst.done;
    burst.done += 1;
    let path = burst.folder.join(format!("f{n:03}.png"));
    if burst.shots {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
    for (orbit, global, local, clusters) in &camera {
        info!(
            "burst {n}: distance {} global {:?} local {:?} clusters far {:?} last {:?} dims {:?}",
            orbit.distance,
            global.translation(),
            local.translation,
            clusters.map(|c| c.far),
            clusters.and_then(|c| c.last_frame_farthest_z),
            clusters.map(|c| c.dimensions),
        );
    }
    for (light, at, seen) in &lights {
        if !seen.get() {
            continue;
        }
        info!(
            "burst {n}: light range {} at {:?} seen {}",
            light.range,
            at.translation(),
            seen.get()
        );
    }
}
