use super::super::geometry::{MIDDLE, centre};
use super::*;

#[test]
fn a_press_on_a_triangle_takes_its_dot_and_beside_them_turns() {
    let draft = Draft::default();
    assert_eq!(
        press_at(centre(Row::Mind, 0), &draft),
        Holding::Dot(Row::Mind)
    );
    // Far out in a corner, by none of them.
    assert!(matches!(
        press_at(Vec2::new(0.98, 0.02), &draft),
        Holding::Turn { .. }
    ));
}

#[test]
fn holding_a_dot_moves_it_and_chooses_its_triangle() {
    let mut draft = Draft::default();
    let corner = corners(Row::Spirit, 0)[1];
    follow(Holding::Dot(Row::Spirit), corner, &mut draft);
    assert_eq!(draft.chosen, Row::Spirit);
    assert_eq!(draft.build.dots[Row::Spirit as usize].0, [0, 1000, 0]);
    assert!(draft.build.is_valid());
}

#[test]
fn holding_beside_them_turns_by_how_far_the_pointer_goes_round() {
    let mut draft = Draft::default();
    // In the gaps between the triangles, a sixth of a turn either side of Body.
    let at = |degrees: f32| {
        let radians = degrees.to_radians();
        MIDDLE + Vec2::new(radians.sin(), -radians.cos()) * 0.45
    };
    let holding = press_at(at(60.0), &draft);
    assert!(matches!(holding, Holding::Turn { .. }));
    follow(holding, at(180.0), &mut draft);
    assert_eq!(draft.build.turn, 120);
    follow(holding, at(300.0), &mut draft);
    assert_eq!(draft.build.turn, 240);
}
