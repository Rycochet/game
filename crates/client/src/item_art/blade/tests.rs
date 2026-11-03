use super::super::{BladeLook, OUTLINE_PIXELS, VIEW_WIDTH};
use shared::blade::{Family, roll_blade};
use shared::item::Quality;
use shared::rng::Seed;

use super::*;

fn looks() -> impl Iterator<Item = BladeLook> {
    Family::ALL.into_iter().flat_map(|family| {
        (0..200).map(move |number| {
            let mut rng = Seed::new(number).derive("blade art").rng();
            let form = roll_blade(&mut rng, family, Quality::default());
            let shape = form.shape();
            BladeLook {
                form,
                cells: (shape.width, shape.height),
            }
        })
    })
}

#[test]
fn every_weapon_stays_inside_its_picture_with_room_for_its_edge() {
    for look in looks() {
        let (wide, long) = (look.cells.0 as f32, look.cells.1 as f32);
        let per_cell = VIEW_WIDTH / wide.max(long);
        let edge = OUTLINE_PIXELS / PIXELS_PER_CELL * per_cell;
        for (_, triangles) in fitted(&look) {
            for corner in triangles.iter().flatten() {
                assert!(corner.is_finite(), "{:?}", look.form);
                assert!(
                    corner.x.abs() + edge <= wide * per_cell / 2.0,
                    "{:?}",
                    look.form
                );
                assert!(
                    corner.y.abs() + edge <= long * per_cell / 2.0,
                    "{:?}",
                    look.form
                );
            }
        }
    }
}

#[test]
fn a_long_weapon_keeps_some_width_to_be_seen() {
    for look in looks().filter(|look| look.cells.1 >= 6) {
        let stuff = fitted(&look);
        let corners = stuff
            .iter()
            .flat_map(|(_, triangles)| triangles.iter().flatten());
        let (low, high) = corners.fold((f32::MAX, f32::MIN), |(low, high), corner| {
            (low.min(corner.x), high.max(corner.x))
        });
        let cell = VIEW_WIDTH / look.cells.1 as f32;
        assert!(high - low > cell * 0.15, "{:?}", look.form);
    }
}
