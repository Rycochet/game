//! `cargo xtask shapes`: creatures' shapes measured from MakeHuman's meshes,
//! as tables of figures for the shared body (`shared::body`).
//!
//! The source files are in `spare_assets/makehuman/` and never shipped: the
//! base mesh of a person (`base.obj`), and targets that reshape it into an
//! animal (`README.md` there has their sources and licences). Each target is
//! applied to the base mesh, and the skin is cut by planes along the body:
//! every cut gives a ring (half width, belly, back). The game never reads a
//! mesh: the figures are copied by hand into the shared body's code. Why and
//! what for: `docs/context/creature-shapes.md`.
//!
//! The readers are our own (MakeHuman's code is AGPL, and none of it is
//! used); both file formats are plain text.

mod check;
mod cut;
mod fourfoot;
mod leg;
mod level;
mod mesh;
mod ring;
mod table;
mod target;
#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

pub use check::check;
use fourfoot::Fourfoot;
use mesh::Mesh;

pub const USAGE: &str = "\
Usage: cargo xtask shapes [rings=10]

  Prints, for each four-legged beast in spare_assets/makehuman/ (cow, dog,
  horse), `rings` rings along its trunk and its legs' thickness at levels up
  from the ground, in thousandths of its height at the withers.";

const SOURCE: &str = "spare_assets/makehuman";
const BASE: &str = "base.obj";
/// How many rings along a trunk, unless `rings=` says otherwise: as many as the
/// shared body has.
const RINGS: usize = shared::body::RINGS;

/// The four-legged beasts measured: a name and its target, under `SOURCE`.
const FOURFEET: [(&str, &str); 3] = [
    ("Cow", "animal04/jaldmic_vaca_cow.target"),
    ("Dog", "animal04/jaldmic_perro_dog.target"),
    ("Horse", "animal04/jaldmic_horse.target"),
];

pub fn run(root: &Path, args: &[String]) -> ExitCode {
    let rings = match args {
        [] => Some(RINGS),
        [arg] => arg
            .strip_prefix("rings=")
            .and_then(|count| count.parse().ok())
            .filter(|count| *count >= 2),
        _ => None,
    };
    let Some(rings) = rings else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    match measure_all(root, rings) {
        Ok(tables) => {
            println!("{}", tables.join("\n"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("==> FAILED: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Every beast's tables.
fn measure_all(root: &Path, rings: usize) -> Result<Vec<String>, String> {
    let mut tables = Vec::new();
    for (name, target) in FOURFEET {
        let beast = measure(root, target, rings)?;
        tables.push(table::fourfoot(name, target, &beast));
    }
    Ok(tables)
}

/// One beast measured: its target applied to the base mesh, read afresh, as a
/// target moves its points.
fn measure(root: &Path, target: &str, rings: usize) -> Result<Fourfoot, String> {
    let folder = root.join(SOURCE);
    let read = |file: &str| {
        let path = folder.join(file);
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))
    };
    let mut mesh = Mesh::read(&read(BASE)?).map_err(|error| format!("{BASE}: {error}"))?;
    target::apply(&mut mesh, &read(target)?).map_err(|error| format!("{target}: {error}"))?;
    Fourfoot::measure(mesh, rings).map_err(|error| format!("{target}: {error}"))
}
