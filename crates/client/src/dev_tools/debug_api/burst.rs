//! TEMPORARY (`tmp.burst`): a screenshot every frame for a while, with the
//! camera and gate lights logged, to catch the gates' light flickering.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use server::debug_api::{BrpResult, Value, json};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::view3d::{OrbitCamera, ViewCamera};

#[derive(Resource, Default)]
pub(super) struct Burst {
    left: u32,
    done: u32,
    folder: PathBuf,
    shots: bool,
    kept: Arc<Mutex<Vec<(u32, Image)>>>,
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
        kept: default(),
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
        // Write out what was kept, once all have arrived.
        let kept: Vec<(u32, Image)> = std::mem::take(&mut *burst.kept.lock().unwrap());
        for (n, image) in kept {
            if let Ok(picture) = image.try_into_dynamic() {
                let _ = picture
                    .to_rgb8()
                    .save(burst.folder.join(format!("f{n:03}.png")));
            }
        }
        return;
    }
    burst.left -= 1;
    let n = burst.done;
    burst.done += 1;
    let path = burst.folder.join(format!("f{n:03}.png"));
    if burst.shots {
        let _ = path;
        let kept = burst.kept.clone();
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>| {
                kept.lock().unwrap().push((n, captured.image.clone()));
            },
        );
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
        let depth = camera.iter().next().map(|(_, cam, _, _)| {
            (at.translation() - cam.translation()).dot(cam.forward().as_vec3())
        });
        info!(
            "burst {n}: light range {} at {:?} depth {:?}",
            light.range,
            at.translation(),
            depth
        );
    }
}
