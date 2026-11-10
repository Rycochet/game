//! The app icon as an SVG: the title font's W large and bronze behind, its H
//! smaller and gold in front, ringed in the background colour so it stands
//! clear of the W, each with the title's dark red shadow.
//!
//! Lengths are in the font's units (1000 to the em) at the W's size, with y
//! downwards as SVG has it (the font's own y points up).

use ttf_parser::{Face, OutlineBuilder, Rect};

/// The loading screen's background, behind and around the letters.
const BACKGROUND: &str = "#0a0812";
/// The title's gold: the H and the frame.
const GOLD: &str = "#edbf61";
/// A deeper gold for the W, so the H in front stands out against it.
const BRONZE: &str = "#a87030";
/// The title's shadow.
const SHADOW: &str = "#591f0d";
/// The H's size against the W's.
const H_SCALE: f32 = 0.6;
/// How far the H's middle is above the W's.
const H_RAISE: f32 = 44.0;
/// How far the ring round the H reaches out from it.
const RING: f32 = 15.0;
/// How far the shadows fall right and down.
const SHADOW_OFFSET: f32 = 23.0;

/// How the square is laid out round the letters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Style {
    /// A rounded square with a gold frame: for 64 pixels and up.
    Framed,
    /// A rounded square without a frame, the letters filling more of it: at
    /// small sizes a frame would take room the letters need.
    Small,
    /// A full square, no frame: for phones, which round the corners
    /// themselves (`apple-touch-icon`).
    Square,
}

impl Style {
    /// How much of the square's width or height the letters fill.
    fn fill(self) -> f32 {
        match self {
            Style::Framed | Style::Square => 0.8,
            Style::Small => 0.92,
        }
    }
}

/// The icon as an SVG document, from the title font's bytes.
pub(crate) fn icon(font: &[u8], style: Style) -> Result<String, String> {
    let face = Face::parse(font, 0).map_err(|e| e.to_string())?;
    let (w, w_box) = outline(&face, 'W')?;
    let (h, h_box) = outline(&face, 'H')?;

    // The H's middle on the W's, raised a little.
    let middle = |b: &Rect| {
        (
            (b.x_min + b.x_max) as f32 / 2.0,
            -(b.y_min + b.y_max) as f32 / 2.0,
        )
    };
    let (wx, wy) = middle(&w_box);
    let (hx, hy) = middle(&h_box);
    let h_x = wx - H_SCALE * hx;
    let h_y = wy - H_RAISE - H_SCALE * hy;

    // Both letters, with the ring and the shadows, centred in the square.
    let left = (w_box.x_min as f32).min(h_x + H_SCALE * h_box.x_min as f32) - RING;
    let right = (w_box.x_max as f32).max(h_x + H_SCALE * h_box.x_max as f32) + RING;
    let top = (-w_box.y_max as f32).min(h_y - H_SCALE * h_box.y_max as f32) - RING;
    let bottom = (-w_box.y_min as f32).max(h_y - H_SCALE * h_box.y_min as f32) + RING;
    let (right, bottom) = (right + SHADOW_OFFSET, bottom + SHADOW_OFFSET);
    let side = (right - left).max(bottom - top) / style.fill();
    let x0 = (left + right - side) / 2.0;
    let y0 = (top + bottom - side) / 2.0;

    let back = match style {
        Style::Square => format!(
            r#"<rect x="{x0}" y="{y0}" width="{side}" height="{side}" fill="{BACKGROUND}"/>"#
        ),
        Style::Small => format!(
            r#"<rect x="{x0}" y="{y0}" width="{side}" height="{side}" rx="{}" fill="{BACKGROUND}"/>"#,
            side / 6.0
        ),
        Style::Framed => {
            let frame = side / 36.0;
            format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{BACKGROUND}" stroke="{GOLD}" stroke-width="{frame}"/>"#,
                x0 + frame / 2.0,
                y0 + frame / 2.0,
                side - frame,
                side - frame,
                side / 6.0 - frame / 2.0
            )
        }
    };
    // Inside the H's group lengths shrink with it, so they are made larger.
    let ring = 2.0 * RING / H_SCALE;
    let fall = SHADOW_OFFSET / H_SCALE;
    let ring_path = |at: f32| {
        format!(
            r#"<path d="{h}" transform="translate({at} {at})" fill="{BACKGROUND}" stroke="{BACKGROUND}" stroke-width="{ring}" stroke-linejoin="round"/>"#
        )
    };
    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x0} {y0} {side} {side}" width="{side}" height="{side}">{back}<path d="{w}" transform="translate({SHADOW_OFFSET} {SHADOW_OFFSET})" fill="{SHADOW}"/><path d="{w}" fill="{BRONZE}"/><g transform="translate({h_x} {h_y}) scale({H_SCALE})">{}{}<path d="{h}" transform="translate({fall} {fall})" fill="{SHADOW}"/><path d="{h}" fill="{GOLD}"/></g></svg>"#,
        ring_path(fall),
        ring_path(0.0),
    ))
}

/// A letter's outline as SVG path data (y downwards), and its box (y up).
fn outline(face: &Face, letter: char) -> Result<(String, Rect), String> {
    let glyph = face
        .glyph_index(letter)
        .ok_or(format!("no {letter} in the font"))?;
    let mut path = PathData(String::new());
    let bounds = face
        .outline_glyph(glyph, &mut path)
        .ok_or(format!("no outline for {letter}"))?;
    Ok((path.0, bounds))
}

/// Writes outline commands as SVG path data, turning y downwards.
struct PathData(String);

impl OutlineBuilder for PathData {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0 += &format!("M{x} {}", -y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0 += &format!("L{x} {}", -y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0 += &format!("Q{x1} {} {x} {}", -y1, -y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0 += &format!("C{x1} {} {x2} {} {x} {}", -y1, -y2, -y);
    }
    fn close(&mut self) {
        self.0 += "Z";
    }
}
