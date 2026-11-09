//! Reading and writing `roles.ron`, keeping the last `BACKUPS` copies in
//! `roles-backups/` as it changes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::file::RolesFile;
use super::stamp::file_stamp;

/// How many earlier copies are kept.
pub(super) const BACKUPS: usize = 10;

/// Where the roles are kept, in a data folder.
#[derive(Clone, Debug)]
pub(super) struct Disk {
    pub data: PathBuf,
}

impl Disk {
    pub fn file(&self) -> PathBuf {
        self.data.join("roles.ron")
    }

    fn backups(&self) -> PathBuf {
        self.data.join("roles-backups")
    }

    /// The file, or `None` if there is none yet. An error if it can't be
    /// read or isn't a roles file: the server mustn't start without its roles.
    pub fn read(&self) -> Result<Option<RolesFile>, String> {
        let path = self.file();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("can't read {}: {error}", path.display())),
        };
        ron::from_str(&text)
            .map(Some)
            .map_err(|error| format!("{} isn't a roles file: {error}", path.display()))
    }

    /// Writes the roles, first copying what was there into the backups (keeping
    /// the newest `BACKUPS`), through a `.new` file so a crash leaves one whole.
    pub fn write(&self, file: &RolesFile, now: u64) -> Result<(), String> {
        let path = self.file();
        let fail = |error: io::Error| format!("can't write {}: {error}", path.display());
        let pretty = ron::ser::PrettyConfig::default().depth_limit(3);
        let text = ron::ser::to_string_pretty(file, pretty).map_err(io::Error::other);
        let text = text.map_err(fail)?;
        if path.exists() {
            self.back_up(&path, now).map_err(fail)?;
        } else {
            fs::create_dir_all(&self.data).map_err(fail)?;
        }
        let new = path.with_extension("ron.new");
        fs::write(&new, text).map_err(fail)?;
        fs::rename(&new, &path).map_err(fail)
    }

    fn back_up(&self, path: &Path, now: u64) -> io::Result<()> {
        let folder = self.backups();
        fs::create_dir_all(&folder)?;
        let name = format!("roles-{}", file_stamp(now));
        let mut copy = folder.join(format!("{name}.ron"));
        let mut number = 1;
        while copy.exists() {
            number += 1;
            copy = folder.join(format!("{name}-{number}.ron"));
        }
        fs::copy(path, &copy)?;
        // Named by time, so the oldest sort first.
        let mut kept: Vec<PathBuf> = fs::read_dir(&folder)?
            .filter_map(|entry| Some(entry.ok()?.path()))
            .filter(|path| path.extension().is_some_and(|e| e == "ron"))
            .collect();
        kept.sort();
        let extra = kept.len().saturating_sub(BACKUPS);
        for old in &kept[..extra] {
            fs::remove_file(old)?;
        }
        Ok(())
    }
}
