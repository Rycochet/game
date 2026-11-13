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
mod head;
mod leg;
mod level;
mod mesh;
mod ring;
mod table;
mod target;
#[cfg(test)]
mod tests;
mod upright;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

pub use check::check;
use fourfoot::Fourfoot;
use head::Head;
use mesh::Mesh;
use upright::Upright;

pub const USAGE: &str = "\
Usage: cargo xtask shapes [rings=10]

  Prints, for the base person in spare_assets/makehuman/, `rings` rings up
  its trunk and its limbs' thickness, in thousandths of its height; for each
  four-legged beast (cow, dog, horse), `rings` rings along its trunk and its
  legs' thickness at levels up from the ground, in thousandths of its height
  at the withers; and the heads' rings.";

const SOURCE: &str = "spare_assets/makehuman";
const BASE: &str = "base.obj";
/// How many rings along a trunk, unless `rings=` says otherwise: as many as the
/// shared body has.
const RINGS: usize = shared::body::RINGS;

/// How many rings along a head: as many as the shared body has.
const HEAD_RINGS: usize = shared::body::HEAD_RINGS;

/// The heads measured: a name and its target, under `SOURCE`. The wolf's,
/// cow's and dog's are on a person; the horse's is its whole body's.
const HEADS: [(&str, &str); 4] = [
    ("Wolf", "animal03/jaldmic_wolf_headv3.target"),
    ("Cow", "animal03/jaldmic_vaca_cow_head.target"),
    ("Dog", "animal03/jaldmic_perro_dog_head.target"),
    ("Horse", "animal04/jaldmic_horse.target"),
];

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

/// Every body's tables.
fn measure_all(root: &Path, rings: usize) -> Result<Vec<String>, String> {
    let person = measure_upright(root, rings)?;
    let mut tables = vec![table::upright("Person", BASE, &person)];
    for (name, target) in FOURFEET {
        let beast = measure(root, target, rings)?;
        tables.push(table::fourfoot(name, target, &beast));
    }
    for (name, target) in HEADS {
        let head = measure_head(root, target)?;
        tables.push(table::head(name, target, &head));
    }
    Ok(tables)
}

/// A mesh with a target applied to it.
fn reshaped(root: &Path, target: &str) -> Result<Mesh, String> {
    let folder = root.join(SOURCE);
    let read = |file: &str| {
        let path = folder.join(file);
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))
    };
    let mut mesh = Mesh::read(&read(BASE)?).map_err(|error| format!("{BASE}: {error}"))?;
    target::apply(&mut mesh, &read(target)?).map_err(|error| format!("{target}: {error}"))?;
    Ok(mesh)
}

/// The base mesh itself, a person standing upright, measured with `rings`
/// rings up its trunk.
fn measure_upright(root: &Path, rings: usize) -> Result<Upright, String> {
    let path = root.join(SOURCE).join(BASE);
    let text = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mesh = Mesh::read(&text).map_err(|error| format!("{BASE}: {error}"))?;
    Upright::measure(&mesh, rings).map_err(|error| format!("{BASE}: {error}"))
}

/// One head measured, `HEAD_RINGS` rings along it.
fn measure_head(root: &Path, target: &str) -> Result<Head, String> {
    Head::measure(&reshaped(root, target)?, HEAD_RINGS)
        .map_err(|error| format!("{target}: {error}"))
}

/// One beast measured: its target applied to the base mesh, read afresh, as a
/// target moves its points.
fn measure(root: &Path, target: &str, rings: usize) -> Result<Fourfoot, String> {
    Fourfoot::measure(reshaped(root, target)?, rings).map_err(|error| format!("{target}: {error}"))
}
