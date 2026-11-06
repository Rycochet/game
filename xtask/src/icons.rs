//! `cargo xtask icons`: the game-icons.net icons the game uses, made into PNGs.
//!
//! Every icon is kept as an SVG (white on black) in `spare_assets/game-icons/`,
//! which is never shipped. Those listed in `xtask/icons.txt`, one
//! `<author>/<name>` a line, are drawn white on a transparent background,
//! `SIZE` pixels square, into `crates/client/assets/icons/<author>/<name>.png`,
//! and PNGs no longer listed are removed. `check` (part of `lint`) fails when
//! the list and the PNGs differ.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// How many pixels across each icon is drawn: sharp at the interface's
/// largest sizes, and small to download.
const SIZE: u32 = 128;
const LIST: &str = "xtask/icons.txt";
const SOURCE: &str = "spare_assets/game-icons";
const OUT: &str = "crates/client/assets/icons";
/// The black square behind every icon, left out to make it transparent.
const BACKGROUND: &str = r#"<path d="M0 0h512v512H0z"/>"#;

/// Draws every listed icon and removes the PNGs no longer listed.
pub fn make(root: &Path) -> ExitCode {
    println!("==> Icons: {LIST} into {OUT}");
    match make_all(root) {
        Ok(count) => {
            println!("==> {count} icons drawn");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("==> FAILED: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Fails if an icon is listed without its PNG, or a PNG isn't listed.
pub fn check(root: &Path) -> ExitCode {
    println!("==> Icons");
    let (missing, unlisted) = match (listed(root), drawn(root)) {
        (Ok(listed), Ok(drawn)) => (
            listed
                .iter()
                .filter(|name| !drawn.contains(name))
                .cloned()
                .collect::<Vec<_>>(),
            drawn
                .iter()
                .filter(|name| !listed.contains(name))
                .cloned()
                .collect::<Vec<_>>(),
        ),
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("==> FAILED: {error}");
            return ExitCode::FAILURE;
        }
    };
    if missing.is_empty() && unlisted.is_empty() {
        println!("==> Every listed icon is drawn");
        return ExitCode::SUCCESS;
    }
    for name in &missing {
        eprintln!("listed but not drawn: {name}");
    }
    for name in &unlisted {
        eprintln!("drawn but not listed: {name}");
    }
    eprintln!("==> FAILED: run `cargo xtask icons`");
    ExitCode::FAILURE
}

fn make_all(root: &Path) -> Result<usize, String> {
    let names = listed(root)?;
    for name in drawn(root)? {
        if !names.contains(&name) {
            let file = root.join(OUT).join(format!("{name}.png"));
            fs::remove_file(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        }
    }
    for name in &names {
        let svg_file = root.join(SOURCE).join(format!("{name}.svg"));
        let svg =
            fs::read_to_string(&svg_file).map_err(|e| format!("{}: {e}", svg_file.display()))?;
        let png = render(&svg).map_err(|e| format!("{name}: {e}"))?;
        let file = root.join(OUT).join(format!("{name}.png"));
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder).map_err(|e| format!("{}: {e}", folder.display()))?;
        }
        fs::write(&file, png).map_err(|e| format!("{}: {e}", file.display()))?;
    }
    Ok(names.len())
}

/// The icon in white on transparent, as a PNG `SIZE` pixels square.
pub(crate) fn render(svg: &str) -> Result<Vec<u8>, String> {
    if !svg.contains(BACKGROUND) {
        return Err("no black square behind it to leave out".into());
    }
    let svg = svg.replacen(BACKGROUND, "", 1);
    let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())
        .map_err(|e| e.to_string())?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(SIZE, SIZE).ok_or("no pixmap")?;
    let size = tree.size();
    let scale = resvg::tiny_skia::Transform::from_scale(
        SIZE as f32 / size.width(),
        SIZE as f32 / size.height(),
    );
    resvg::render(&tree, scale, &mut pixmap.as_mut());
    pixmap.encode_png().map_err(|e| e.to_string())
}

/// The names in the list, without blank lines or `#` notes.
fn listed(root: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(root.join(LIST)).map_err(|e| format!("{LIST}: {e}"))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(String::from)
        .collect())
}

/// The `<author>/<name>` of every PNG drawn so far.
fn drawn(root: &Path) -> Result<Vec<String>, String> {
    let out = root.join(OUT);
    let mut names = Vec::new();
    let Ok(authors) = fs::read_dir(&out) else {
        return Ok(names);
    };
    for author in authors.flatten().filter(|entry| entry.path().is_dir()) {
        let files = fs::read_dir(author.path()).map_err(|e| e.to_string())?;
        for file in files.flatten() {
            let path = file.path();
            if path.extension().is_some_and(|ext| ext == "png")
                && let Some(stem) = path.file_stem()
            {
                let author = author.file_name();
                names.push(format!(
                    "{}/{}",
                    author.to_string_lossy(),
                    stem.to_string_lossy()
                ));
            }
        }
    }
    Ok(names)
}

#[cfg(test)]
mod tests;
