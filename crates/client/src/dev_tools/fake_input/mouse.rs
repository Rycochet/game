//! Fake mouse: moving, clicking, dragging and scrolling.

use bevy::input::ButtonState;
use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params};

use super::{FakeInput, Step, queued};

#[derive(Deserialize)]
pub(super) struct At {
    x: f32,
    y: f32,
}

/// `client.move { "x": 400, "y": 300 }`: the mouse cursor goes there
/// (logical pixels), for what hovering shows.
pub(super) fn move_to(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let At { x, y } = params(request)?;
    queued(&mut fake, [Step::MoveTo(Vec2::new(x, y))])
}

#[derive(Deserialize)]
pub(super) struct ClickParams {
    x: f32,
    y: f32,
    /// `Left` (the default), `Right` or `Middle`.
    #[serde(default = "left")]
    button: MouseButton,
    /// Frames between pressing and letting go: long enough, a hold.
    #[serde(default = "one")]
    hold_frames: u32,
    /// A finger's tap instead of the mouse, through `touch`.
    #[serde(default)]
    touch: bool,
}

pub(super) fn left() -> MouseButton {
    MouseButton::Left
}

pub(super) fn one() -> u32 {
    1
}

/// `client.click { "x": 400, "y": 300 }`: moves there, presses and lets go
/// (`button`, `hold_frames`), or taps there with `"touch": true`.
pub(super) fn click(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let ClickParams {
        x,
        y,
        button,
        hold_frames,
        touch,
    } = params(request)?;
    let at = Vec2::new(x, y);
    let held = (1..hold_frames).map(|_| Step::Wait);
    if touch {
        let steps = [Step::Finger(TouchPhase::Started, at)]
            .into_iter()
            .chain(held)
            .chain([Step::Finger(TouchPhase::Ended, at), Step::Wait]);
        queued(&mut fake, steps)
    } else {
        let steps = [
            Step::MoveTo(at),
            Step::Wait,
            Step::Mouse(button, ButtonState::Pressed),
        ]
        .into_iter()
        .chain(held)
        .chain([Step::Mouse(button, ButtonState::Released), Step::Wait]);
        queued(&mut fake, steps)
    }
}

#[derive(Deserialize)]
pub(super) struct DragParams {
    from: [f32; 2],
    to: [f32; 2],
    /// Frames the move takes.
    #[serde(default = "ten")]
    steps: u32,
    #[serde(default = "left")]
    button: MouseButton,
    /// A finger instead of the mouse.
    #[serde(default)]
    touch: bool,
}

pub(super) fn ten() -> u32 {
    10
}

/// `client.drag { "from": [100, 100], "to": [300, 200] }`: presses at
/// `from`, moves to `to` over `steps` frames and lets go; with
/// `"touch": true` a finger does it.
pub(super) fn drag(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let DragParams {
        from,
        to,
        steps,
        button,
        touch,
    } = params(request)?;
    queued(
        &mut fake,
        drag_steps(Vec2::from(from), Vec2::from(to), steps, button, touch),
    )
}

/// The steps of a drag.
pub(super) fn drag_steps(
    from: Vec2,
    to: Vec2,
    frames: u32,
    button: MouseButton,
    touch: bool,
) -> Vec<Step> {
    let frames = frames.max(1);
    let along = (1..=frames).map(|frame| from.lerp(to, frame as f32 / frames as f32));
    let mut steps = Vec::new();
    if touch {
        steps.push(Step::Finger(TouchPhase::Started, from));
        steps.extend(along.map(|at| Step::Finger(TouchPhase::Moved, at)));
        steps.push(Step::Finger(TouchPhase::Ended, to));
    } else {
        steps.extend([
            Step::MoveTo(from),
            Step::Wait,
            Step::Mouse(button, ButtonState::Pressed),
        ]);
        steps.extend(along.map(Step::MoveTo));
        steps.push(Step::Mouse(button, ButtonState::Released));
    }
    steps.push(Step::Wait);
    steps
}

#[derive(Deserialize)]
pub(super) struct ScrollParams {
    /// Where the cursor is while scrolling, if not where it is now.
    x: Option<f32>,
    y: Option<f32>,
    /// Wheel lines: positive scrolls up (zooms in), negative down.
    lines: f32,
}

/// `client.scroll { "lines": 3 }`: the mouse wheel, at `x`, `y` if given.
pub(super) fn scroll(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let ScrollParams { x, y, lines } = params(request)?;
    let mut steps: Vec<Step> = x
        .zip(y)
        .map(|(x, y)| Step::MoveTo(Vec2::new(x, y)))
        .into_iter()
        .collect();
    steps.extend([Step::Wheel(lines), Step::Wait]);
    queued(&mut fake, steps)
}
