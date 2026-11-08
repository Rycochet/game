//! How much moving teaches: practice for Climbing and Running from world map
//! steps. The server gives it, at most once per skill every 5 seconds
//! (`skill::Windows`), so these are per 5 seconds of travel. Settings for
//! balancing: a test prints how long each takes to reach 25%, 50% and 100%.

/// Practice per elevation (25 m) a step climbs or descends beyond the first.
pub const CLIMB_PRACTICE_PER_ELEVATION: u32 = 6;

/// The most one step can teach Climbing, for the steepest cliffs.
pub const MOST_CLIMB_PRACTICE: u32 = 40;

/// Practice for Running per world map step: about 50% after 10 hours of world
/// map travel, 100% after 80.
pub const RUN_PRACTICE: u32 = 17;

/// What a step climbing or descending `elevations` teaches Climbing: nothing on
/// ground about flat (a single elevation, 25 m, or less), more the steeper.
pub fn climb_practice(elevations: u32) -> u32 {
    (elevations.saturating_sub(1) * CLIMB_PRACTICE_PER_ELEVATION).min(MOST_CLIMB_PRACTICE)
}

#[cfg(test)]
mod tests;
