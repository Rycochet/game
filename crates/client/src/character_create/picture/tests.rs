use super::*;
use shared::creature::CreatureKind;

fn at(pixels: &[u8], point: Vec2) -> [u8; 4] {
    let x = (point.x * PIXELS as f32) as u32;
    let y = (point.y * PIXELS as f32) as u32;
    let index = ((y * PIXELS + x) * 4) as usize;
    pixels[index..index + 4].try_into().unwrap()
}

fn drawn_for(kind: CreatureKind) -> Vec<u8> {
    let build = Build::CENTRED;
    let attributes = Attributes::for_kind(kind, shared::gender::Gender::Neutral, &build);
    let bonus = kind.def().bonus();
    paint(&Drawn {
        build: &build,
        attributes: &attributes,
        bonus: &bonus,
        chosen: Row::Body,
    })
}

#[test]
fn the_middle_is_clear_and_each_triangle_is_drawn() {
    let pixels = drawn_for(CreatureKind::Human);
    assert_eq!(at(&pixels, Vec2::new(0.5, 0.5))[3], 0);
    assert_eq!(at(&pixels, Vec2::new(0.01, 0.01))[3], 0);
    for row in Row::ALL {
        let corners = corners(row, 0);
        // Between its middle and a corner, past the chart: the triangle's own colour.
        let inside = super::super::geometry::centre(row, 0) * 0.1 + corners[0] * 0.9;
        assert_eq!(at(&pixels, inside)[3], 255, "{row:?}");
    }
}

#[test]
fn a_bonus_shows_in_its_own_colour_and_none_still_shows_a_little() {
    let minotaur = drawn_for(CreatureKind::Minotaur);
    let human = drawn_for(CreatureKind::Human);
    // Towards the strength corner, past the dot's ring: a minotaur's bonus (30)
    // reaches it, a human's (6) doesn't.
    let middle = super::super::geometry::centre(Row::Body, 0);
    let strength = corners(Row::Body, 0)[0];
    let point = middle + (strength - middle).normalize() * 0.03;
    assert_ne!(at(&minotaur, point), at(&human, point));
    // With no bonus, a little round shape is left at the middle.
    let bonus_colour = at(&human, middle);
    assert!(bonus_colour[0] > bonus_colour[2], "{bonus_colour:?}");
}

#[test]
fn blending_over_nothing_gives_the_colour() {
    let mut pixel = [0, 0, 0, 0];
    blend(&mut pixel, [10, 20, 30, 255], 1.0);
    assert_eq!(pixel, [10, 20, 30, 255]);
    blend(&mut pixel, [200, 200, 200, 255], 0.0);
    assert_eq!(pixel, [10, 20, 30, 255]);
}
