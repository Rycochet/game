//! Squeezing through a gap narrower than oneself (owner, 2026-11-10).

/// The slowest a squeeze makes a move: 4 times.
pub const MOST_SQUEEZE_PER_MILLE: u32 = 4000;

/// How much slower a creature of `radius` moves with `clearance` units round
/// its middle, in thousandths (1000: not at all), or `None` if it can't get
/// through. A gap is half as wide as the room either side of the middle, so a
/// creature gets through a gap down to half its own width (`clearance` down to
/// half its `radius`), its speed divided by (wanted / available)², at most 4
/// times slower.
pub fn squeeze_per_mille(radius: i64, clearance: i64) -> Option<u32> {
    if clearance >= radius {
        return Some(1000);
    }
    if 2 * clearance < radius {
        return None;
    }
    let (radius, clearance) = (radius as u64, clearance as u64);
    Some((1000 * radius * radius / (clearance * clearance)) as u32)
}
