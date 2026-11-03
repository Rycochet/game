//! `client.time_pictures` and `client.picture_times`: how long a few hundred
//! pictures take to draw, the same ones every time, so that changes to drawing can
//! be measured against each other. Start it in a fresh game: pictures already
//! drawn are kept, and the first ones wait for the graphics card's programs.

use super::ItemPictures;
use super::samples::{PIXELS_PER_CELL, looks};
use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params, refused};

/// Frames timed before asking, for how long a frame takes with nothing to draw.
const IDLE_FRAMES: usize = 60;

#[derive(Resource, Default)]
pub(super) struct Timing {
    count: usize,
    idle: Vec<f32>,
    /// Once asked: when, how many pictures had been drawn by then, and when the first
    /// of these was.
    started: Option<(f64, u64)>,
    first: Option<f64>,
    frames: Vec<f32>,
    /// Time spent building models before asking.
    building: std::time::Duration,
    result: Option<Value>,
}

impl Timing {
    fn running(&self) -> bool {
        self.result.is_none() && (self.count > 0 || self.started.is_some())
    }
}

#[derive(Deserialize)]
struct Asked {
    count: Option<usize>,
}

pub(super) fn add(app: &mut App) {
    use server::debug_api::AddDebugMethod;
    app.init_resource::<Timing>()
        .add_systems(PostUpdate, time_pictures.after(super::draw_pictures))
        .add_debug_method("client.time_pictures", start)
        .add_debug_method("client.picture_times", times);
}

/// `client.time_pictures {count?}` (300 if not given): times a second of frames with
/// nothing new, then asks for `count` pictures and times them until all are drawn.
fn start(In(request): In<Option<Value>>, mut timing: ResMut<Timing>) -> BrpResult {
    let asked: Asked = params(request)?;
    if timing.running() {
        return Err(refused("pictures are already being timed"));
    }
    *timing = Timing {
        count: asked.count.unwrap_or(300),
        ..default()
    };
    Ok(json!("started"))
}

/// `client.picture_times`: `running`, or the times of the last run.
fn times(_: In<Option<Value>>, timing: Res<Timing>) -> BrpResult {
    Ok(timing.result.clone().unwrap_or(json!("running")))
}

fn time_pictures(
    time: Res<Time<Real>>,
    mut timing: ResMut<Timing>,
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
) {
    if !timing.running() {
        return;
    }
    let now = time.elapsed_secs_f64();
    let Some((started, drawn_before)) = timing.started else {
        timing.idle.push(time.delta_secs());
        if timing.idle.len() >= IDLE_FRAMES {
            let waiting = pictures.waiting.len();
            for look in looks(1, timing.count) {
                let pixels = look.longest() as u32 * PIXELS_PER_CELL;
                pictures.picture(look, pixels, &mut images);
            }
            // Only those not drawn already.
            timing.count = pictures.waiting.len() - waiting;
            timing.started = Some((now, pictures.drawn));
            timing.building = pictures.building;
        }
        return;
    };
    timing.frames.push(time.delta_secs());
    let drawn = pictures.drawn - drawn_before;
    if drawn > 0 && timing.first.is_none() {
        timing.first = Some(now - started);
    }
    if drawn as usize >= timing.count && pictures.waiting.is_empty() {
        let total = now - started;
        let ms = |frames: &[f32]| frames.iter().sum::<f32>() * 1000.0 / frames.len() as f32;
        let worst = timing.frames.iter().fold(0.0f32, |a, &b| a.max(b)) * 1000.0;
        timing.result = Some(json!({
            "pictures": timing.count,
            "total_s": total,
            "first_picture_s": timing.first,
            "pictures_per_s": timing.count as f64 / total,
            "frames": timing.frames.len(),
            "average_frame_ms": ms(&timing.frames),
            "worst_frame_ms": worst,
            "idle_frame_ms": ms(&timing.idle),
            "building_models_s": (pictures.building - timing.building).as_secs_f64(),
        }));
    }
}
