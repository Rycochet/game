//! Slides the bands on set items' edges as the view changes: turning, tilting and
//! zooming the camera, and walking. They follow the way it changes, not how far, at
//! a speed that eases and is capped, as fast changes would make them flash.

use bevy::prelude::*;

use super::{OrbitCamera, ViewCamera};
use crate::common::{BandShift, SmoothPosition};
use crate::player::Player;

/// How much the bands want to move, in pixels on the screen, for a radian of turning
/// or tilting, a doubling of distance (near enough), and a cell walked.
const PIXELS_PER_RADIAN: f32 = 400.0;
const PIXELS_PER_ZOOM: f32 = 400.0;
const PIXELS_PER_CELL: f32 = 30.0;
/// The fastest they go, in pixels a second.
const MOST_SPEED: f32 = 200.0;
/// How quickly their speed follows what is wanted, per second.
const EASING: f32 = 5.0;
/// Slower than this, in pixels a second, is stopped.
const STOPPED: f32 = 0.5;

/// The view as last seen, and how fast the bands are going.
#[derive(Default)]
pub(super) struct Seen {
    view: Option<(f32, f32, f32, Vec2)>,
    speed: f32,
}

/// How fast the bands go next, given how fast they go now and the change in view
/// wanted, in pixels a second.
fn eased(speed: f32, wanted: f32, seconds: f32) -> f32 {
    let wanted = wanted.clamp(-MOST_SPEED, MOST_SPEED);
    let speed = speed + (wanted - speed) * (seconds * EASING).min(1.0);
    if wanted == 0.0 && speed.abs() < STOPPED {
        0.0
    } else {
        speed
    }
}

pub(super) fn drift_bands(
    time: Res<Time<Real>>,
    camera: Single<&OrbitCamera, With<ViewCamera>>,
    player: Single<&SmoothPosition, With<Player>>,
    mut shift: ResMut<BandShift>,
    mut seen: Local<Seen>,
) {
    let seconds = time.delta_secs();
    let at = Vec2::new(player.x as f32, player.y as f32);
    let now = (camera.yaw, camera.pitch, camera.distance, at);
    let Some((yaw, pitch, distance, from)) = seen.view.replace(now) else {
        return;
    };
    if seconds <= 0.0 {
        return;
    }
    // Along the bands' own diagonal on the screen: right and down.
    let (right, towards) = camera.ground_axes();
    let walked = (at - from).dot(right + towards);
    let zoomed = (camera.distance / distance.max(1.0)).ln();
    let change = (camera.yaw - yaw + camera.pitch - pitch) * PIXELS_PER_RADIAN
        + zoomed * PIXELS_PER_ZOOM
        + walked * PIXELS_PER_CELL;
    seen.speed = eased(seen.speed, change / seconds, seconds);
    if seen.speed != 0.0 {
        shift.0 += seen.speed * seconds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bands_go_the_way_of_the_change_no_faster_than_the_cap() {
        let mut speed = 0.0;
        for _ in 0..200 {
            speed = eased(speed, 100_000.0, 0.016);
        }
        assert!(speed > 0.0);
        assert!(speed <= MOST_SPEED);
        for _ in 0..400 {
            speed = eased(speed, -100_000.0, 0.016);
        }
        assert!(speed < 0.0);
        assert!(speed >= -MOST_SPEED);
    }

    #[test]
    fn the_bands_ease_to_a_stop_after_the_view_stops() {
        let first = eased(MOST_SPEED, 0.0, 0.016);
        assert!(first > 0.0);
        assert!(first < MOST_SPEED, "not all at once");
        let mut speed = first;
        for _ in 0..500 {
            speed = eased(speed, 0.0, 0.016);
        }
        assert_eq!(speed, 0.0);
    }

    /// Bevy only finds two parameters clashing when a system starts.
    #[test]
    fn it_asks_for_nothing_twice() {
        IntoSystem::into_system(drift_bands).initialize(&mut World::new());
    }
}
