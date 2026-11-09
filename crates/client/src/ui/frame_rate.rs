//! The frame rate the status text shows: frames a second, averaged over the
//! last couple of seconds so it doesn't flicker.

use std::collections::VecDeque;

use bevy::prelude::*;

/// How far back the average looks, in seconds.
const WINDOW_SECONDS: f64 = 2.0;

/// When the last couple of seconds' frames began (real time, in seconds).
#[derive(Resource, Default)]
pub(super) struct FrameTimes(VecDeque<f64>);

impl FrameTimes {
    /// Notes a frame beginning at `now`, forgetting those too long ago.
    pub(super) fn note(&mut self, now: f64) {
        self.0.push_back(now);
        while self
            .0
            .front()
            .is_some_and(|&first| now - first > WINDOW_SECONDS)
        {
            self.0.pop_front();
        }
    }

    /// Frames a second over the frames noted, once there are two.
    pub(super) fn rate(&self) -> Option<f64> {
        let (first, last) = (*self.0.front()?, *self.0.back()?);
        let span = last - first;
        (span > 0.0).then(|| (self.0.len() - 1) as f64 / span)
    }
}

pub(super) fn note_frame(time: Res<Time<Real>>, mut times: ResMut<FrameTimes>) {
    times.note(time.elapsed_secs_f64());
}

#[cfg(test)]
mod tests;
