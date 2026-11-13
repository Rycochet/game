use super::super::projection::Projection;
use super::*;
use crate::quests::TrackedDots;

const SIZE: u32 = 100;
const GOLD: [u8; 3] = [240, 200, 40];

/// A north-up map, a cell to 10 pixels, centred on (50, 50).
fn under(inside: Option<((f32, f32), f32)>) -> Under {
    Under {
        projection: Projection {
            centre: (50.0, 50.0),
            middle: (50.0, 50.0),
            units_per_pixel: 0.1,
            right: Projection::NORTH_UP.0,
            down: Projection::NORTH_UP.1,
        },
        size: (SIZE, SIZE),
        full_map: inside.is_none(),
        inside,
        mark: 1.0,
        other_floor: None,
        wrap: None,
    }
}

fn painted(under: &Under, places: Vec<(i64, i64)>) -> Vec<u8> {
    let mut pixels = vec![0; (SIZE * SIZE * 4) as usize];
    let mut canvas = Canvas {
        pixels: &mut pixels,
        width: SIZE,
        height: SIZE,
    };
    let marks = QuestMarks { rgb: GOLD, places };
    draw_quest_dots(&mut canvas, under, &marks);
    pixels
}

fn pixel(pixels: &[u8], (x, y): (u32, u32)) -> [u8; 4] {
    let at = ((y * SIZE + x) * 4) as usize;
    [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
}

#[test]
fn a_dot_on_the_map_is_in_the_quests_colour() {
    let pixels = painted(&under(None), vec![(52, 50)]);
    assert_eq!(pixel(&pixels, (70, 50)), [240, 200, 40, 255]);
    assert_eq!(pixel(&pixels, (50, 50))[3], 0);
}

#[test]
fn off_the_full_map_an_arrow_points_from_its_edge() {
    // Far to the east: an arrow on the right edge, `ARROW_INSET` in.
    let pixels = painted(&under(None), vec![(500, 50)]);
    let inset = (SIZE as f32 - ARROW_INSET) as u32;
    assert_eq!(pixel(&pixels, (inset, 50)), [240, 200, 40, 255]);
    assert_eq!(pixel(&pixels, (50, 50))[3], 0);
}

#[test]
fn off_the_minimap_an_arrow_sits_inside_its_round_border() {
    let inside = Some(((50.0, 50.0), 40.0));
    let pixels = painted(&under(inside), vec![(50, 10)]);
    // North, on the circle `ARROW_INSET` inside the border.
    let y = (50.0 - (40.0 - ARROW_INSET)) as u32;
    assert_eq!(pixel(&pixels, (50, y)), [240, 200, 40, 255]);
    // Nothing at the corner, outside the circle.
    assert_eq!(pixel(&pixels, (50, 2))[3], 0);
}

#[test]
fn round_edge_is_none_inside_the_circle() {
    assert!(round_edge((55.0, 50.0), (50.0, 50.0), 20.0).is_none());
    let (place, direction) = round_edge((90.0, 50.0), (50.0, 50.0), 20.0).unwrap();
    assert_eq!(place, (70.0, 50.0));
    assert_eq!(direction, (1.0, 0.0));
}

#[test]
fn none_on_another_floor_and_placed_the_near_way_round() {
    let dots = TrackedDots {
        rgb: GOLD,
        places: vec![(98, 3)],
    };
    let mut shown = under(None);
    shown.wrap = Some(100);
    assert_eq!(
        QuestMarks::of(&dots, (1.0, 0.0), &shown).places,
        vec![(-2, 3)]
    );
    shown.other_floor = Some(true);
    assert!(QuestMarks::of(&dots, (1.0, 0.0), &shown).places.is_empty());
}
