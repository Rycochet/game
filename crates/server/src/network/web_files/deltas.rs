//! What the web client's deltas are made from and kept as: the current
//! build's wasm, deltas made for it beforehand (in its `deltas` folder, by
//! `cargo xtask web-deltas`) and, with a history folder, deltas from the
//! builds this server served before, made on a thread of their own so
//! starting up never waits for them.

use axum::body::Bytes;
use axum::http::HeaderValue;
use bevy::log::{info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;
use web_delta::{DELTAS, Hash, encode, find_wasm, from_hex, hex, sha256};

use super::super::WEB_PATH;
use super::WebClient;
use super::history::{self, delta_path, write_whole};

/// The web client's files, and the deltas there are for its wasm.
pub(in crate::network) struct WebFiles {
    pub(super) files: PathBuf,
    /// The current wasm's file name, if there is one wasm.
    wasm: Option<String>,
    /// What browsers are told to keep the wasm as (`Use-As-Dictionary`).
    pub(super) use_as_dictionary: HeaderValue,
    /// Each delta to the current wasm, by the hash of the build it is from.
    deltas: RwLock<HashMap<Hash, Bytes>>,
}

impl WebFiles {
    /// Finds the wasm and the deltas made for it, and starts making those
    /// from the builds in the history folder.
    pub(in crate::network) fn start(client: WebClient) -> Arc<Self> {
        let wasm = find_wasm(&client.files).unwrap_or_else(|error| {
            warn!("Can't read {}: {error}", client.files.display());
            None
        });
        let pattern = format!("match=\"{WEB_PATH}*_bg.wasm\"");
        let web = Arc::new(Self {
            files: client.files.clone(),
            wasm: wasm
                .as_ref()
                .and_then(|wasm| wasm.file_name()?.to_str().map(Into::into)),
            use_as_dictionary: HeaderValue::from_str(&pattern).expect("plain text"),
            deltas: RwLock::default(),
        });
        let Some(wasm) = wasm else { return web };
        web.load_made_beforehand(&client.files.join(DELTAS));
        if let Some(history) = client.history {
            let making = web.clone();
            thread::Builder::new()
                .name("web deltas".into())
                .spawn(move || making.make_from_history(&history, &wasm))
                .expect("a thread");
        }
        web
    }

    /// The current wasm's file name.
    pub(super) fn wasm(&self) -> Option<&str> {
        self.wasm.as_deref()
    }

    /// The delta from the build with this hash, if there is one.
    pub(super) fn delta_from(&self, hash: &Hash) -> Option<Bytes> {
        self.deltas
            .read()
            .expect("never poisoned")
            .get(hash)
            .cloned()
    }

    fn add(&self, from: Hash, delta: Vec<u8>) {
        let mut deltas = self.deltas.write().expect("never poisoned");
        deltas.insert(from, Bytes::from(delta));
    }

    fn load_made_beforehand(&self, folder: &Path) {
        let Ok(entries) = fs::read_dir(folder) else {
            return;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("");
            if let (Some(from), Ok(delta)) = (from_hex(stem), fs::read(&path)) {
                self.add(from, delta);
            }
        }
    }

    fn make_from_history(&self, history: &Path, wasm: &Path) {
        if let Err(error) = self.try_making(history, wasm) {
            warn!(
                "Can't make web client deltas in {}: {error}",
                history.display()
            );
        }
    }

    fn try_making(&self, history: &Path, wasm: &Path) -> Result<(), String> {
        let current = fs::read(wasm).map_err(|error| error.to_string())?;
        let hash = sha256(&current);
        let older = history::keep(history, &current, &hash).map_err(|error| error.to_string())?;
        for (from, old) in older {
            if self.delta_from(&from).is_some() {
                continue;
            }
            let kept = delta_path(history, &from, &hash);
            let delta = match fs::read(&kept) {
                Ok(delta) => delta,
                Err(_) => {
                    let started = Instant::now();
                    let dictionary = fs::read(&old).map_err(|error| error.to_string())?;
                    let delta = encode(&dictionary, &current)?;
                    write_whole(&kept, &delta).map_err(|error| error.to_string())?;
                    info!(
                        "Made a web client delta from build {}: {} bytes, in {:.0} s",
                        &hex(&from)[..12],
                        delta.len(),
                        started.elapsed().as_secs_f32()
                    );
                    delta
                }
            };
            self.add(from, delta);
        }
        Ok(())
    }
}
