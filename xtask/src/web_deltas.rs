//! `cargo xtask web-deltas <dir> <old wasm>...`: a delta into the web build
//! in `dir` from each old build's wasm, so the server can send browsers that
//! have one of those builds only what changed. For releases: give the wasm of
//! the last few releases as they were published (a rebuild isn't the same
//! bytes, so browsers would never ask for it).

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;
use web_delta::{DELTAS, delta_name, encode, find_wasm, sha256};

pub fn run(args: &[String]) -> ExitCode {
    let [dir, old @ ..] = args else {
        eprintln!("Usage: cargo xtask web-deltas <dir> <old wasm>...");
        return ExitCode::FAILURE;
    };
    match make_deltas(Path::new(dir), old) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("==> FAILED: {error}");
            ExitCode::FAILURE
        }
    }
}

fn make_deltas(dir: &Path, old: &[String]) -> Result<(), String> {
    let wasm = find_wasm(dir)
        .map_err(|error| format!("{}: {error}", dir.display()))?
        .ok_or_else(|| format!("{}: no single *_bg.wasm", dir.display()))?;
    let new = read(&wasm)?;
    let deltas = dir.join(DELTAS);
    fs::create_dir_all(&deltas).map_err(|error| format!("{}: {error}", deltas.display()))?;
    println!("==> Deltas to {}", wasm.display());
    for old in old {
        let dictionary = read(Path::new(old))?;
        let hash = sha256(&dictionary);
        if dictionary == new {
            println!("    {old}: the same build, skipped");
            continue;
        }
        let started = Instant::now();
        let delta = encode(&dictionary, &new).map_err(|error| format!("{old}: {error}"))?;
        let target = deltas.join(delta_name(&hash));
        fs::write(&target, &delta).map_err(|error| format!("{}: {error}", target.display()))?;
        println!(
            "    {old}: {:.2} MB ({:.1} s)",
            delta.len() as f64 / 1_000_000.0,
            started.elapsed().as_secs_f32()
        );
    }
    Ok(())
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}
