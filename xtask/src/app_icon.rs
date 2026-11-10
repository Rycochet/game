//! `cargo xtask app-icon`: the game's own icon, the stylised HW, drawn from
//! the title font into `crates/client/assets/app_icon/`.
//!
//! - `icon-<size>.png`: the window's icon (native) and the web page's
//!   favicons.
//! - `apple-touch-icon.png`: for phones' home screens, as a full square.
//! - `icon.ico`: the Windows program's icon, built into it by the client's
//!   `build.rs`.
//!
//! The pictures are kept in git, so this only needs running again when the
//! design (in `svg.rs`) or the font changes.

mod ico;
mod svg;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use svg::Style;

const FONT: &str = "crates/client/assets/fonts/HallOfWorldsTitle.ttf";
const OUT: &str = "crates/client/assets/app_icon";

/// Each PNG's width in pixels and how it is laid out.
const PNGS: &[(u32, Style)] = &[
    (16, Style::Small),
    (24, Style::Small),
    (32, Style::Small),
    (48, Style::Small),
    (64, Style::Framed),
    (128, Style::Framed),
    (256, Style::Framed),
    (512, Style::Framed),
];
/// The sizes Windows picks from for the program's icon.
const ICO_SIZES: &[u32] = &[16, 24, 32, 48, 64, 128, 256];
/// The size iPhones and iPads ask for.
const APPLE_TOUCH: u32 = 180;

/// Draws every picture of the icon.
pub fn make(root: &Path) -> ExitCode {
    println!("==> App icon: {FONT} into {OUT}");
    match make_all(root) {
        Ok(count) => {
            println!("==> {count} files written");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("==> FAILED: {error}");
            ExitCode::FAILURE
        }
    }
}

fn make_all(root: &Path) -> Result<usize, String> {
    let font = fs::read(root.join(FONT)).map_err(|e| format!("{FONT}: {e}"))?;
    let out = root.join(OUT);
    fs::create_dir_all(&out).map_err(|e| format!("{OUT}: {e}"))?;
    let write = |name: &str, bytes: &[u8]| {
        fs::write(out.join(name), bytes).map_err(|e| format!("{OUT}/{name}: {e}"))
    };
    let mut for_ico = Vec::new();
    for &(size, style) in PNGS {
        let png = render(&svg::icon(&font, style)?, size)?;
        write(&format!("icon-{size}.png"), &png)?;
        if ICO_SIZES.contains(&size) {
            for_ico.push((size, png));
        }
    }
    let apple = render(&svg::icon(&font, Style::Square)?, APPLE_TOUCH)?;
    write("apple-touch-icon.png", &apple)?;
    write("icon.ico", &ico::ico(&for_ico))?;
    Ok(PNGS.len() + 2)
}

/// The SVG drawn as a PNG `size` pixels square.
pub(crate) fn render(svg: &str, size: u32) -> Result<Vec<u8>, String> {
    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default())
        .map_err(|e| e.to_string())?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).ok_or("no pixmap")?;
    let scale = size as f32 / tree.size().width();
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    pixmap.encode_png().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
