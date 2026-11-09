//! The audit log: one line for every role given, taken or run out, in
//! `audit.log` in the data folder (never rewritten, only added to). Accounts
//! are named by id only.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use bevy::log::{error, info};

use super::stamp::log_stamp;

#[derive(Default)]
pub(super) struct Audit {
    /// The log's file, or `None` to keep lines in `lines` (tests).
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
}

impl Audit {
    pub fn record(&mut self, now: u64, what: &str) {
        info!("Roles: {what}");
        let line = format!("{} {what}", log_stamp(now));
        let Some(path) = &self.path else {
            self.lines.push(line);
            return;
        };
        let written = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| writeln!(file, "{line}"));
        if let Err(problem) = written {
            error!("Can't add to {}: {problem} ({line})", path.display());
        }
    }
}
