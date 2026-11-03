//! Waiting for frames, and how much fake input is left.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params};

use super::{FakeInput, Step, one, queued};

#[derive(Deserialize)]
pub(super) struct WaitParams {
    #[serde(default = "one")]
    frames: u32,
}

/// `client.wait { "frames": 30 }`: answers once that many frames have gone
/// by after everything already queued (through `cargo xtask api`).
pub(super) fn wait(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let WaitParams { frames } = params(request)?;
    let frames = frames.clamp(1, 100_000);
    queued(&mut fake, (0..frames).map(|_| Step::Wait))
}

/// `client.input`: how many steps of fake input are still to be played, and
/// the frame count, for waiting on them.
pub(super) fn input(_: In<Option<Value>>, fake: Res<FakeInput>) -> BrpResult {
    Ok(json!({ "pending": fake.steps.len(), "frame": fake.frame }))
}
