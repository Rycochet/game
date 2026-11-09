//! Setting aside saves from a version this one can't read (`save_format.ron`).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{FORMAT_FILE, SAVE_FOLDERS, read, write};
use crate::storage::{OLDEST_SAVE_FORMAT, SAVE_FORMAT};

/// `save_format.ron`.
#[derive(Serialize, Deserialize)]
struct FormatSave {
    format: u32,
}

/// Run before anything opens a data folder. If its saves are from before
/// `OLDEST_SAVE_FORMAT`, moves them into a dated folder beside them, for the owner
/// to delete: nothing is ever deleted here. Folders without a format file are from
/// before there was one. An error if they can't be moved, as the server would then
/// be mixing old saves with new.
pub(crate) fn set_aside_old_saves(data: &Path) -> Result<(), String> {
    let file = data.join(FORMAT_FILE);
    // One that makes no sense says nothing, the same as there being none.
    let format = read::<FormatSave>(&file)
        .unwrap_or(None)
        .map(|save| save.format);
    if format.is_none_or(|format| format < OLDEST_SAVE_FORMAT) {
        let found: Vec<_> = SAVE_FOLDERS
            .iter()
            .filter(|folder| data.join(folder).exists())
            .collect();
        if !found.is_empty() {
            let aside = old_saves_folder(data);
            fs::create_dir_all(&aside)
                .map_err(|error| format!("can't make {}: {error}", aside.display()))?;
            for folder in found {
                fs::rename(data.join(folder), aside.join(folder))
                    .map_err(|error| format!("can't move {folder} aside: {error}"))?;
            }
            // Logging may not have started yet, and a host needs to see this.
            eprintln!(
                "Saves from an older version can't be used, so they were moved to {}",
                aside.display()
            );
        }
    }
    // Never lowered: a newer server's saves are refused later, with a reason.
    if format.is_none_or(|format| format < SAVE_FORMAT) {
        let format = FormatSave {
            format: SAVE_FORMAT,
        };
        write(&file, &format)?;
    }
    Ok(())
}

/// `old-saves-<today>` in the data folder, numbered if there already is one.
fn old_saves_folder(data: &Path) -> PathBuf {
    let today = OffsetDateTime::now_utc().date();
    let name = format!(
        "old-saves-{}-{:02}-{:02}",
        today.year(),
        today.month() as u8,
        today.day()
    );
    let mut folder = data.join(&name);
    let mut number = 1;
    while folder.exists() {
        number += 1;
        folder = data.join(format!("{name}-{number}"));
    }
    folder
}
