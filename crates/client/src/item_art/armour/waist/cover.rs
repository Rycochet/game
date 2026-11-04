//! Covering a surface as a waist piece's construction is made: lames or rows standing
//! over each other, rings, scales, quilting or riveted cloth; then what its style adds
//! to plate, and a rolled edge of its trim.

use shared::armour::waist::WaistForm;
use shared::armour::{Construction, Style};

use crate::item_art::armour::surface::{Surface, bands, lines, patch, scales, studs};
use crate::item_art::armour::{Model, Stuff};

/// How far each lame or row stands out over the one below.
const STEP: f32 = 1.4;

/// Thin stripes across a surface between rows, of this stuff: lacing.
fn stripes(model: &mut Model, stuff: Stuff, surface: Surface, rows: usize, lift: f32) {
    for row in 1..rows {
        let v = row as f32 / rows as f32;
        let lift = lift + (rows - row) as f32 * STEP;
        patch(
            model,
            stuff,
            surface,
            (0.0, 1.0),
            (v - 0.02, v + 0.02),
            lift,
            0.8,
        );
    }
}

/// The surface covered as the piece is made, in `rows` rows (lames, hoops, rows of
/// plates); `(wide, long)` is about its size in millimetres, for scales and studs.
pub(super) fn cover(
    model: &mut Model,
    form: &WaistForm,
    surface: Surface,
    rows: usize,
    (wide, long): (f32, f32),
) {
    use Construction as C;
    let rows = rows.max(1);
    let columns = ((wide / 60.0) as usize).max(2);
    match form.construction {
        C::Plate | C::Laminar | C::Splint if rows > 1 => {
            bands(model, Stuff::Main, surface, rows, STEP, 1.6)
        }
        C::Lamellar => {
            bands(model, Stuff::Main, surface, rows, STEP, 2.0);
            stripes(model, Stuff::Cloth, surface, rows, 1.2);
        }
        // Rows of plates with mail between them.
        C::MailAndPlate => {
            bands(model, Stuff::Main, surface, rows, STEP, 1.8);
            lines(model, surface, rows * 3, 0.4);
        }
        C::Scale => {
            let rows = rows.max((long / 45.0) as usize);
            scales(
                model,
                Stuff::Main,
                surface,
                rows,
                (wide / 40.0) as usize + 2,
            );
        }
        C::Mail | C::RingMail => {
            patch(
                model,
                Stuff::Main,
                surface,
                (0.0, 1.0),
                (0.0, 1.0),
                0.0,
                2.5,
            );
            lines(model, surface, ((long / 22.0) as usize).max(3), 1.4);
        }
        C::PlatesInCloth => {
            patch(
                model,
                Stuff::Cloth,
                surface,
                (0.0, 1.0),
                (0.0, 1.0),
                0.0,
                3.0,
            );
            let down = ((long / 70.0) as usize).max(2);
            studs(model, Stuff::Rivet, surface, (down, columns * 2), 1.6, 3.0);
        }
        C::Padded => {
            patch(
                model,
                Stuff::Main,
                surface,
                (0.0, 1.0),
                (0.0, 1.0),
                0.0,
                6.0,
            );
            lines(model, surface, ((long / 40.0) as usize).max(3), 3.1);
        }
        // Leather, and single plates.
        _ => {
            patch(
                model,
                Stuff::Main,
                surface,
                (0.0, 1.0),
                (0.0, 1.0),
                0.0,
                3.0,
            );
            if form.studded {
                let down = ((long / 50.0) as usize).max(2);
                studs(model, Stuff::Rivet, surface, (down, columns * 2), 1.8, 3.0);
            }
        }
    }
    add_style(model, form, surface, rows, columns);
    if form.trim.is_some() {
        let lift = rows as f32 * STEP + 0.5;
        patch(
            model,
            Stuff::Trim,
            surface,
            (0.0, 1.0),
            (0.95, 1.0),
            lift,
            2.4,
        );
    }
}

/// What a style adds over plate: Gothic's ridges, Maximilian's flutes, Greenwich's
/// etched bands, Dwarven rivets.
fn add_style(model: &mut Model, form: &WaistForm, surface: Surface, rows: usize, columns: usize) {
    if form.construction != Construction::Plate {
        return;
    }
    let lift = rows as f32 * STEP + 1.2;
    let strip = |model: &mut Model, stuff: Stuff, u: f32, width: f32, thick: f32| {
        let (u0, u1) = ((u - width / 2.0).max(0.0), (u + width / 2.0).min(1.0));
        patch(model, stuff, surface, (u0, u1), (0.02, 0.97), lift, thick);
    };
    match form.style {
        Style::Gothic => {
            for at in [0.3, 0.5, 0.7] {
                strip(model, Stuff::Main, at, 0.04, 3.5);
            }
        }
        Style::Maximilian => {
            let flutes = (columns * 3).max(5);
            for flute in 0..flutes {
                let at = (flute as f32 + 0.5) / flutes as f32;
                strip(model, Stuff::Main, at, 0.35 / flutes as f32, 3.0);
            }
        }
        Style::Greenwich => strip(model, Stuff::Etch, 0.5, 0.22, 0.6),
        Style::Dwarven => studs(model, Stuff::Rivet, surface, (rows, columns * 2), lift, 3.2),
        _ => {}
    }
}
