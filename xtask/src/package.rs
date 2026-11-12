//! `cargo xtask package`: the release build as players get it.

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use super::{cargo, copy_dir, target_dir, workspace_root};

/// Builds the release executables and copies them, with the client's
/// `assets` folder, into `package/` in cargo's target folder (usually
/// `target/package/`): the layout players get, since release builds load
/// assets from next to the executable.
pub fn run() -> ExitCode {
    let build = [
        "build",
        "--release",
        "--locked",
        "--package",
        "client",
        "--package",
        "server",
    ];
    println!("==> Build: cargo {}", build.join(" "));
    if !cargo(&build) {
        eprintln!("==> FAILED: Build");
        return ExitCode::FAILURE;
    }

    let root = workspace_root();
    let target = target_dir::target_dir(&root);
    let package = target.join("package");
    println!("==> Package: {}", package.display());
    match assemble(&root, &target.join("release"), &package) {
        Ok(()) => {
            println!("==> Packaged");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("==> FAILED: Package: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Empties `package`, then copies in the executables from `release` and the
/// assets.
fn assemble(root: &Path, release: &Path, package: &Path) -> io::Result<()> {
    if package.exists() {
        fs::remove_dir_all(package)?;
    }
    fs::create_dir_all(package)?;
    for name in ["hall-of-worlds", "hall-of-worlds-server"] {
        let file = format!("{name}{}", env::consts::EXE_SUFFIX);
        fs::copy(release.join(&file), package.join(&file))?;
    }
    copy_dir(&root.join("crates/client/assets"), &package.join("assets"))
}
