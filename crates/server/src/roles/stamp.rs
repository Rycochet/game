//! Times written into the audit log and backup names, in UTC.

use time::OffsetDateTime;

fn at(seconds: u64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(seconds as i64).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

/// `2026-11-09 10:04:05 UTC`, for the audit log.
pub(crate) fn log_stamp(seconds: u64) -> String {
    let t = at(seconds);
    format!(
        "{}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        t.year(),
        t.month() as u8,
        t.day(),
        t.hour(),
        t.minute(),
        t.second()
    )
}

/// `2026-11-09T10-04-05`, for file names (sorting by time).
pub(super) fn file_stamp(seconds: u64) -> String {
    let t = at(seconds);
    format!(
        "{}-{:02}-{:02}T{:02}-{:02}-{:02}",
        t.year(),
        t.month() as u8,
        t.day(),
        t.hour(),
        t.minute(),
        t.second()
    )
}
