//! Fake mouse: moving, clicking, dragging and scrolling.

use bevy::input::ButtonState;
use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params};

use super::aim::{aim, aim_needed};
use super::{FakeInput, Step, queued};
use crate::widgets::WidgetPlaces;

#[derive(Deserialize)]
pub(super) struct MoveParams {
    x: Option<f32>,
    y: Option<f32>,
    /// A widget to point at instead (see `aim`).
    id: Option<String>,
}

/// `client.move { "x": 400, "y": 300 }`: the game's pointer goes there
/// (logical pixels), for what hovering shows; the real one doesn't move.
/// `client.move { "id": "step_pad/step_up" }` points at a widget.
/// `client.move {}` gives the pointer back to the real mouse, as a minute
/// with no fake input does.
pub(super) fn move_to(
    In(request): In<Option<Value>>,
    mut fake: ResMut<FakeInput>,
    places: WidgetPlaces,
) -> BrpResult {
    let MoveParams { x, y, id } = params(request)?;
    let step = aim(&places, id.as_deref(), x, y)?.map_or(Step::LetGo, Step::MoveTo);
    queued(&mut fake, [step])
}

#[derive(Deserialize)]
pub(super) struct ClickParams {
    x: Option<f32>,
    y: Option<f32>,
    /// A widget to click instead: its centre, or `x`, `y` from its top left.
    id: Option<String>,
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
/// `client.click { "id": "step_pad/turn_left", "hold_frames": 60 }` clicks
/// (or holds) a widget.
pub(super) fn click(
    In(request): In<Option<Value>>,
    mut fake: ResMut<FakeInput>,
    places: WidgetPlaces,
) -> BrpResult {
    let ClickParams {
        x,
        y,
        id,
        button,
        hold_frames,
        touch,
    } = params(request)?;
    let at = aim_needed(&places, id.as_deref(), x, y)?;
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
    from: Option<[f32; 2]>,
    to: Option<[f32; 2]>,
    /// Widgets to drag from and to instead (their centres; with `from` or
    /// `to` as well, that far from the widget's top left).
    from_id: Option<String>,
    to_id: Option<String>,
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
/// `"touch": true` a finger does it. `from_id` and `to_id` name widgets.
pub(super) fn drag(
    In(request): In<Option<Value>>,
    mut fake: ResMut<FakeInput>,
    places: WidgetPlaces,
) -> BrpResult {
    let DragParams {
        from,
        to,
        from_id,
        to_id,
        steps,
        button,
        touch,
    } = params(request)?;
    let end = |id: Option<&str>, at: Option<[f32; 2]>| {
        aim_needed(&places, id, at.map(|a| a[0]), at.map(|a| a[1]))
    };
    let from = end(from_id.as_deref(), from)?;
    let to = end(to_id.as_deref(), to)?;
    queued(&mut fake, drag_steps(from, to, steps, button, touch))
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
    /// A widget to scroll over instead (see `aim`).
    id: Option<String>,
    /// Wheel lines: positive scrolls up (zooms in), negative down.
    lines: f32,
}

/// `client.scroll { "lines": 3 }`: the mouse wheel, at `x`, `y` (or `id`)
/// if given.
pub(super) fn scroll(
    In(request): In<Option<Value>>,
    mut fake: ResMut<FakeInput>,
    places: WidgetPlaces,
) -> BrpResult {
    let ScrollParams { x, y, id, lines } = params(request)?;
    let mut steps: Vec<Step> = aim(&places, id.as_deref(), x, y)?
        .map(Step::MoveTo)
        .into_iter()
        .collect();
    steps.extend([Step::Wheel(lines), Step::Wait]);
    queued(&mut fake, steps)
}
