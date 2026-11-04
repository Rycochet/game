//! World time as the World Time panel writes and reads it: `Day 1 06:30`.

const MINUTES_PER_DAY: u64 = 24 * MINUTES_PER_HOUR;
const MINUTES_PER_HOUR: u64 = 60;

/// `minutes` since the world began as a day (counting from 1) and a time of day.
pub(super) fn time_text(minutes: f64) -> String {
    let whole = minutes.max(0.0).floor() as u64;
    let day = whole / MINUTES_PER_DAY + 1;
    let hour = whole % MINUTES_PER_DAY / MINUTES_PER_HOUR;
    let minute = whole % MINUTES_PER_HOUR;
    format!("Day {day} {hour:02}:{minute:02}")
}

/// The minutes since the world began that `text` names, or `None` if it can't be
/// read. A time of day alone (`06:30`) is on the day `now` is in.
pub(super) fn read_time(text: &str, now: f64) -> Option<f64> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let (day, clock) = match words.as_slice() {
        [clock] => (now.max(0.0) as u64 / MINUTES_PER_DAY + 1, *clock),
        [word, day, clock] if word.eq_ignore_ascii_case("day") => {
            // A day of `u32` keeps the minutes well inside what `u64` holds.
            (u64::from(day.parse::<u32>().ok()?), *clock)
        }
        _ => return None,
    };
    let (hour, minute) = clock.split_once(':')?;
    let hour: u64 = hour.parse().ok()?;
    let minute_digits = minute.len();
    let minute: u64 = minute.parse().ok()?;
    if day == 0 || hour >= 24 || minute >= MINUTES_PER_HOUR || minute_digits != 2 {
        return None;
    }
    Some(((day - 1) * MINUTES_PER_DAY + hour * MINUTES_PER_HOUR + minute) as f64)
}
