use super::*;

#[test]
fn heavier_never_looks_emptier() {
    let mut last = Load::Flat;
    for grams in (0..30_000).step_by(50) {
        let load = Load::of_weight(grams);
        assert!(load >= last);
        last = load;
    }
    assert_eq!(Load::of_weight(0), Load::Flat);
    assert_eq!(last, Load::Bulging);
}

#[test]
fn anyone_in_a_cells_middle_reaches_all_of_it() {
    use crate::creature::Size;
    use crate::spot::{HALF_SPAN, Spot};
    let middle = Spot::middle_of((3, -2));
    let corner = middle.offset(HALF_SPAN - 1, HALF_SPAN - 1);
    for size in Size::ALL {
        assert!(in_reach(size, middle, corner), "{size:?}");
    }
    // Not the next cell's middle, for anyone up to Medium.
    let next = Spot::middle_of((4, -2));
    assert!(!in_reach(Size::Medium, middle, next));
}

#[test]
fn the_nearest_bag_in_reach() {
    use crate::creature::Size;
    use crate::spot::Spot;
    let at = Spot::new(0, 0);
    let bags = [Spot::new(200, 0), Spot::new(0, -100), Spot::new(900, 0)];
    assert_eq!(
        nearest_in_reach(Size::Medium, at, bags),
        Some(Spot::new(0, -100))
    );
    assert_eq!(
        nearest_in_reach(Size::Medium, at, [Spot::new(900, 0)]),
        None
    );
}
