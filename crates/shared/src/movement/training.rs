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

/// Practice for Weight Training per step while carrying more than half of
/// what can be carried (`sheet::Load::Burdened`), on any map: about 50% after
/// 10 hours of such walking. Twice as much while overloaded.
pub const WEIGHT_PRACTICE: u32 = 17;

/// Practice for Jumping per jump: slow to learn (owner), about 50% after
/// 3,000 jumps and 100% after 25,000.
pub const JUMP_PRACTICE: u32 = 40;

/// Practice for Sneaking per step in the wilderness at a sneaking pace while
/// something wary (timid or territorial, and not out to attack) watches: about
/// 50% after 10 hours of it.
pub const SNEAK_PRACTICE: u32 = 17;

/// Practice for a terrain's Pioneering skill per step over it (off roads and
/// water), on either map: about 50% after 10 hours there.
pub const PIONEER_PRACTICE: u32 = 17;

/// Practice for Swimming per step wading through shallows on a level: about
/// 50% after 10 hours of it.
pub const SWIM_PRACTICE: u32 = 17;

/// Practice for Swimming per world map step across a river: simple, and not
/// very effective (owner).
pub const RIVER_SWIM_PRACTICE: u32 = 4;

/// What a step teaches Weight Training under this load: nothing when light.
pub fn weight_practice(load: crate::sheet::Load) -> u32 {
    match load {
        crate::sheet::Load::Light => 0,
        crate::sheet::Load::Burdened => WEIGHT_PRACTICE,
        crate::sheet::Load::Overloaded => 2 * WEIGHT_PRACTICE,
    }
}

/// What a step climbing or descending `elevations` teaches Climbing: nothing on
/// ground about flat (a single elevation, 25 m, or less), more the steeper.
pub fn climb_practice(elevations: u32) -> u32 {
    (elevations.saturating_sub(1) * CLIMB_PRACTICE_PER_ELEVATION).min(MOST_CLIMB_PRACTICE)
}

#[cfg(test)]
mod tests;
