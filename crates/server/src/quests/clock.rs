//! The time quests go by: real Unix seconds, as dailies come by real days and
//! retaking waits a real hour (owner).

use std::time::{SystemTime, UNIX_EPOCH};

/// Now, in whole Unix seconds (0 if the computer's clock is before 1970).
pub(crate) fn unix_now() -> i64 {
    (SystemTime::now().duration_since(UNIX_EPOCH)).map_or(0, |since| since.as_secs() as i64)
}
