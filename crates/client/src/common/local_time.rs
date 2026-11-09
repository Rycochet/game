//! Times as the player's clock shows them: when a character was made and last
//! played. Native clients ask the system for the local time zone (UTC if it
//! won't say); a browser knows its own.

/// The months' short names.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// A moment (seconds since 1970) as the date and time here: "9 Nov 2026, 14:05".
pub fn local_date_time(seconds: u64) -> String {
    let (year, month, day, hour, minute) = parts(seconds);
    let month = MONTHS.get(month as usize).copied().unwrap_or("?");
    format!("{day} {month} {year}, {hour:02}:{minute:02}")
}

/// Year, month from 0, day, hour and minute, here.
#[cfg(not(target_arch = "wasm32"))]
fn parts(seconds: u64) -> (i32, u8, u8, u8, u8) {
    use std::sync::OnceLock;
    use time::{OffsetDateTime, UtcOffset};
    // Asked once: some systems refuse once there are threads, but say the same.
    static HERE: OnceLock<UtcOffset> = OnceLock::new();
    let here = *HERE.get_or_init(|| UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
    let seconds = i64::try_from(seconds).unwrap_or(0);
    let at = OffsetDateTime::from_unix_timestamp(seconds)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .to_offset(here);
    let month = u8::from(at.month()) - 1;
    (at.year(), month, at.day(), at.hour(), at.minute())
}

#[cfg(target_arch = "wasm32")]
fn parts(seconds: u64) -> (i32, u8, u8, u8, u8) {
    let at = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(seconds as f64 * 1000.0));
    (
        at.get_full_year() as i32,
        at.get_month() as u8,
        at.get_date() as u8,
        at.get_hours() as u8,
        at.get_minutes() as u8,
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
