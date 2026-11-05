//! `cargo xtask compress-web <dir>`: a brotli copy (`<file>.br`) beside each
//! file of a web build that compresses well, for the server to send browsers
//! that take brotli (most do) instead of the file itself.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

/// Files smaller than this aren't worth a second copy.
const SMALLEST: usize = 1024;

/// Kinds of file that are compressed already, so brotli gains nothing.
const ALREADY_COMPRESSED: &[&str] = &[
    "br", "gz", "zst", "dcz", "png", "jpg", "jpeg", "webp", "avif", "ktx2", "basis", "ogg", "mp3",
    "woff2",
];

pub fn run(args: &[String]) -> ExitCode {
    let [dir] = args else {
        eprintln!("Usage: cargo xtask compress-web <dir>");
        return ExitCode::FAILURE;
    };
    println!("==> Compress web files in {dir}");
    match compress_dir(Path::new(dir)) {
        Ok((before, after)) => {
            println!(
                "==> Compressed {} into {}",
                megabytes(before),
                megabytes(after)
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("==> FAILED: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Compresses every file in `dir` (and the folders in it) that is worth it,
/// returning the sizes of those files before and after.
fn compress_dir(dir: &Path) -> io::Result<(usize, usize)> {
    let (mut before, mut after) = (0, 0);
    for file in files_in(dir)? {
        if !worth_trying(&file) {
            continue;
        }
        let data = fs::read(&file)?;
        if data.len() < SMALLEST {
            continue;
        }
        let started = Instant::now();
        let compressed = brotli(&data)?;
        // Only kept when it saves at least a tenth.
        if compressed.len() * 10 > data.len() * 9 {
            continue;
        }
        println!(
            "    {}: {} to {} ({:.1} s)",
            file.display(),
            megabytes(data.len()),
            megabytes(compressed.len()),
            started.elapsed().as_secs_f32()
        );
        fs::write(with_br(&file), &compressed)?;
        before += data.len();
        after += compressed.len();
    }
    Ok((before, after))
}

/// Every file in `dir` and the folders in it.
fn files_in(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            files.extend(files_in(&entry.path())?);
        } else {
            files.push(entry.path());
        }
    }
    Ok(files)
}

/// Whether a file's kind might compress.
fn worth_trying(file: &Path) -> bool {
    let extension = file
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    !extension.is_some_and(|extension| ALREADY_COMPRESSED.contains(&extension.as_str()))
}

/// `file` with `.br` added to its name.
fn with_br(file: &Path) -> PathBuf {
    let mut name = file.as_os_str().to_owned();
    name.push(".br");
    PathBuf::from(name)
}

/// The best (slowest) brotli there is: done once per build, read by every
/// browser.
fn brotli(data: &[u8]) -> io::Result<Vec<u8>> {
    let params = brotli::enc::BrotliEncoderParams {
        quality: 11,
        // The largest window browsers take (16 MiB).
        lgwin: 24,
        size_hint: data.len(),
        ..Default::default()
    };
    let mut compressed = Vec::new();
    brotli::BrotliCompress(&mut &data[..], &mut compressed, &params)?;
    Ok(compressed)
}

fn megabytes(bytes: usize) -> String {
    format!("{:.2} MB", bytes as f64 / 1_000_000.0)
}

#[cfg(test)]
mod tests;
