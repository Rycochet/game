//! Delta downloads of the web client: a new build of a file compressed
//! against an old build a browser already has, in the `dcz` encoding of
//! Compression Dictionary Transport (RFC 9842). The browser names the old
//! build it has by its SHA-256 (`Available-Dictionary`), and what it gets
//! back is a 40-byte header (a zstd skippable frame holding that hash)
//! followed by a zstd frame that uses the old build as its prefix.
//!
//! Used by the server, which makes deltas from the builds it served before,
//! and by `cargo xtask web-deltas`, which makes them from published releases.

use sha2::{Digest, Sha256};
use std::io;
use std::path::{Path, PathBuf};
use zstd_safe::{CCtx, CParameter, DCtx, DParameter};

/// A SHA-256 hash: how an old build is named.
pub type Hash = [u8; 32];

/// The folder in a web build that holds deltas made for it beforehand
/// (`cargo xtask web-deltas`), each named `<hash of the old build>.dcz`.
pub const DELTAS: &str = "deltas";

/// The name of a delta from the build with this hash.
pub fn delta_name(from: &Hash) -> String {
    format!("{}.dcz", hex(from))
}

/// The web build's wasm in `dir` (Trunk's `<name>-<hash>_bg.wasm`), if
/// there is exactly one.
pub fn find_wasm(dir: &Path) -> io::Result<Option<PathBuf>> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path.file_name().and_then(|name| name.to_str());
        if name.is_some_and(|name| name.ends_with("_bg.wasm")) {
            found.push(path);
        }
    }
    Ok(if found.len() == 1 { found.pop() } else { None })
}

/// How every `dcz` answer starts, before the dictionary's hash.
const MAGIC: [u8; 8] = [0x5e, 0x2a, 0x4d, 0x18, 0x20, 0x00, 0x00, 0x00];
const HEADER: usize = MAGIC.len() + 32;

/// How hard zstd works: a delta is made once and sent to many browsers.
const LEVEL: i32 = 19;

/// The largest window browsers take: 1.25 times the dictionary, but at
/// least 8 MiB and at most 128 MiB.
fn window_log(dictionary_len: usize) -> u32 {
    let limit = (dictionary_len as u64 * 5 / 4).clamp(8 << 20, 128 << 20);
    63 - limit.leading_zeros()
}

/// The SHA-256 of `data`.
pub fn sha256(data: &[u8]) -> Hash {
    Sha256::digest(data).into()
}

/// `hash` as lower-case hex, for file names.
pub fn hex(hash: &Hash) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The hash named by 64 hex digits, if `text` is that.
pub fn from_hex(text: &str) -> Option<Hash> {
    if text.len() != 64 || !text.is_ascii() {
        return None;
    }
    let mut hash = [0; 32];
    for (index, byte) in hash.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(hash)
}

/// `data` as a `dcz` delta from `dictionary`.
pub fn encode(dictionary: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let mut context = CCtx::create();
    for parameter in [
        CParameter::CompressionLevel(LEVEL),
        CParameter::WindowLog(window_log(dictionary.len())),
        // Finds the long runs the two builds share, however far apart.
        CParameter::EnableLongDistanceMatching(true),
    ] {
        context.set_parameter(parameter).map_err(error)?;
    }
    context.ref_prefix(dictionary).map_err(error)?;
    let mut frame = Vec::with_capacity(zstd_safe::compress_bound(data.len()));
    context.compress2(&mut frame, data).map_err(error)?;

    let mut delta = Vec::with_capacity(HEADER + frame.len());
    delta.extend_from_slice(&MAGIC);
    delta.extend_from_slice(&sha256(dictionary));
    delta.extend_from_slice(&frame);
    Ok(delta)
}

/// What a `dcz` delta from `dictionary` holds, as a browser would read it.
pub fn decode(dictionary: &[u8], delta: &[u8]) -> Result<Vec<u8>, String> {
    if delta.len() < HEADER || delta[..MAGIC.len()] != MAGIC {
        return Err("not a dcz delta".into());
    }
    if delta[MAGIC.len()..HEADER] != sha256(dictionary) {
        return Err("made from another dictionary".into());
    }
    let frame = &delta[HEADER..];
    let size = zstd_safe::get_frame_content_size(frame)
        .ok()
        .flatten()
        .ok_or("no size in the frame")?;
    let mut context = DCtx::create();
    context
        .set_parameter(DParameter::WindowLogMax(window_log(dictionary.len())))
        .map_err(error)?;
    context.ref_prefix(dictionary).map_err(error)?;
    let mut data = Vec::with_capacity(size as usize);
    context.decompress(&mut data, frame).map_err(error)?;
    Ok(data)
}

fn error(code: zstd_safe::ErrorCode) -> String {
    zstd_safe::get_error_name(code).to_string()
}

#[cfg(test)]
mod tests;
