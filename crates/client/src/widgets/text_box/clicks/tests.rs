use super::*;

#[test]
fn quick_clicks_in_one_place_count_up_to_three_and_start_again() {
    let a = Entity::from_raw_u32(1).expect("an entity");
    let at = Vec2::new(10.0, 10.0);
    let first = clicks_in_a_row(None, (a, at, 0.0));
    assert_eq!(first, 1);
    let second = clicks_in_a_row(Some((a, at, 0.0, first)), (a, at, 0.2));
    assert_eq!(second, 2);
    let third = clicks_in_a_row(Some((a, at, 0.2, second)), (a, at, 0.4));
    assert_eq!(third, 3);
    assert_eq!(clicks_in_a_row(Some((a, at, 0.4, third)), (a, at, 0.6)), 1);
}

#[test]
fn slow_far_or_other_clicks_start_again() {
    let (a, b) = (
        Entity::from_raw_u32(1).expect("an entity"),
        Entity::from_raw_u32(2).expect("an entity"),
    );
    let at = Vec2::ZERO;
    assert_eq!(clicks_in_a_row(Some((a, at, 0.0, 1)), (a, at, 1.0)), 1);
    assert_eq!(
        clicks_in_a_row(Some((a, at, 0.0, 1)), (a, Vec2::X * 20.0, 0.1)),
        1
    );
    assert_eq!(clicks_in_a_row(Some((a, at, 0.0, 1)), (b, at, 0.1)), 1);
}
