//! How bright a flash is, by how long since it struck.

/// A flash lasts this long: a few frames, like a token's red flash
/// (`view3d::FLASH_SECONDS`).
const FLASH_SECONDS: f32 = 0.1;

/// With Reduce Motion, a flash is this bright at most, and fades in and out
/// over these times rather than going on and off at once.
const GENTLE_MOST: f32 = 0.35;
const GENTLE_IN: f32 = 0.2;
const GENTLE_OUT: f32 = 0.4;

/// How bright a flash is `seconds` after it struck (0 to 1), or `None` once
/// it is over. Asked with the time before each frame is added, so a full
/// flash is always seen for at least a frame.
pub(super) fn strength(seconds: f32, gentle: bool) -> Option<f32> {
    if !gentle {
        return (seconds < FLASH_SECONDS).then_some(1.0);
    }
    if seconds < GENTLE_IN {
        Some(GENTLE_MOST * seconds / GENTLE_IN)
    } else if seconds < GENTLE_IN + GENTLE_OUT {
        Some(GENTLE_MOST * (1.0 - (seconds - GENTLE_IN) / GENTLE_OUT))
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
