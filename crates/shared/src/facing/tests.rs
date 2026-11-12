use super::*;
use crate::creature::CreatureKind;
use crate::dungeon::{Level, Sight};
use crate::spot::{CELL_SPAN, Spot};

#[test]
fn headings_of_the_eight_ways() {
    let quarter = (TURN / 4) as u16;
    assert_eq!(Heading::of(5, 0), Some(Heading::EAST));
    assert_eq!(Heading::of(0, 5), Some(Heading::SOUTH));
    assert_eq!(Heading::of(-5, 0), Some(Heading::WEST));
    assert_eq!(Heading::of(0, -5), Some(Heading::NORTH));
    assert_eq!(Heading::of(0, 0), None);
    for (dx, dy, eighths) in [(1, 1, 1), (-1, 1, 3), (-1, -1, 5), (1, -1, 7)] {
        let way = Heading::of(dx * 7, dy * 7).unwrap();
        assert!(
            way.apart(Heading(eighths * EIGHTH)) <= 16,
            "{dx},{dy}: {way:?}"
        );
    }
    assert_eq!(Heading::EAST.turned(-i32::from(quarter)), Heading::NORTH);
}

#[test]
fn a_heading_and_its_vector_agree() {
    for steps in (0..TURN).step_by(997) {
        let heading = Heading(steps as u16);
        let (x, y) = heading.unit();
        let length = ((x * x + y * y) as f64).sqrt();
        assert!(
            (length - UNIT as f64).abs() < UNIT as f64 * 0.003,
            "{steps}: {length}"
        );
        let back = Heading::of(x, y).unwrap();
        // Within about half a degree.
        assert!(back.apart(heading) <= 100, "{steps}: back as {back:?}");
    }
}

#[test]
fn apart_goes_the_short_way() {
    assert_eq!(Heading(10).apart(Heading(65530)), 16);
    assert_eq!(Heading::EAST.apart(Heading::WEST), (TURN / 2) as u16);
}

#[test]
fn people_see_ahead_and_beside_them_only() {
    let field = Field::of(CreatureKind::Human, 0);
    let cone = Cone::new(Spot::new(0, 0), Heading::EAST, field);
    let cell = |x, y| Spot::middle_of((x, y));
    assert!(cone.sees(cell(5, 0)));
    assert!(cone.sees(cell(5, 2)), "within 60° of ahead");
    assert!(!cone.sees(cell(1, 5)), "too far round");
    assert!(!cone.sees(cell(-5, 0)), "behind");
    // The near circle: right behind, a cell away, is seen.
    assert!(cone.sees(cell(-1, 0)));
    assert!(!cone.sees(Spot::new(-CELL_SPAN - 1, 0)));
}

#[test]
fn prey_see_almost_all_round() {
    let cone = Cone::new(
        Spot::new(0, 0),
        Heading::EAST,
        Field::of(CreatureKind::Horse, 0),
    );
    assert!(cone.sees(Spot::middle_of((-5, 3))));
    assert!(
        !cone.sees(Spot::middle_of((-9, 0))),
        "a narrow blind spot behind"
    );
}

#[test]
fn observation_widens_and_lengthens() {
    let none = Field::of(CreatureKind::Human, 0);
    let full = Field::of(CreatureKind::Human, 10_000);
    assert!(full.half > none.half);
    assert_eq!((none.further, full.further), (0, 3));
    // More than 100% adds no more.
    assert_eq!(Field::of(CreatureKind::Human, 99_999), full);
    assert!(observation_skill().path().ends_with("observation"));
}

#[test]
fn sight_in_a_cone_leaves_out_what_is_behind() {
    let level = Level::test_room();
    let cone = Cone::new(
        Spot::middle_of((3, 0)),
        Heading::EAST,
        Field::of(CreatureKind::Human, 0),
    );
    let mut ahead = Sight::new(&level);
    ahead.add_view_in(&level, 3, 0, None, &cone);
    let mut round = Sight::new(&level);
    round.add_view(&level, 3, 0, None);
    assert!(ahead.is_seen(6, 0) && round.is_seen(6, 0));
    assert!(!ahead.is_seen(0, 0) && round.is_seen(0, 0));
    assert!(ahead.is_seen(2, 0), "within the near circle");
}
