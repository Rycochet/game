use super::*;
use crate::poi::PoiKind;

const WIDTH: i64 = 10_000;

#[test]
fn directions_go_round_the_compass_with_north_up() {
    let from = (500, 500);
    let way = |dx: i64, dy: i64| Direction::between(from, (500 + dx, 500 + dy), WIDTH);
    assert_eq!(way(0, 0), Direction::Here);
    assert_eq!(way(0, -10), Direction::North);
    assert_eq!(way(10, -10), Direction::NorthEast);
    assert_eq!(way(10, 0), Direction::East);
    assert_eq!(way(10, 10), Direction::SouthEast);
    assert_eq!(way(0, 10), Direction::South);
    assert_eq!(way(-10, 10), Direction::SouthWest);
    assert_eq!(way(-10, 0), Direction::West);
    assert_eq!(way(-10, -10), Direction::NorthWest);
    // Either side of 22.5°: tan is 0.4142...
    assert_eq!(way(1000, -414), Direction::East);
    assert_eq!(way(1000, -415), Direction::NorthEast);
    assert_eq!(way(-414, 1000), Direction::South);
    assert_eq!(way(-415, 1000), Direction::SouthWest);
}

#[test]
fn ways_and_distances_go_the_near_way_round() {
    // 60 points east of the seam, the place 60 points west of it.
    let from = (60, 0);
    let to = (WIDTH - 60, 0);
    assert_eq!(Direction::between(from, to, WIDTH), Direction::West);
    assert_eq!(Direction::between(to, from, WIDTH), Direction::East);
    assert_eq!(DistanceBand::between(from, to, WIDTH), DistanceBand::Short);
    // Huge positions don't overflow.
    let far = (i64::MAX / 4, i64::MAX / 4);
    assert_eq!(DistanceBand::between((0, 0), far, WIDTH), DistanceBand::Far);
}

#[test]
fn distance_bands_change_at_their_limits() {
    let band = |dx: i64| DistanceBand::between((0, 0), (dx, 0), WIDTH);
    assert_eq!(band(0), DistanceBand::Near);
    assert_eq!(band(NEAR_WITHIN), DistanceBand::Near);
    assert_eq!(band(NEAR_WITHIN + 1), DistanceBand::Short);
    assert_eq!(band(SHORT_WITHIN), DistanceBand::Short);
    assert_eq!(band(SHORT_WITHIN + 1), DistanceBand::Day);
    assert_eq!(band(DAY_WITHIN), DistanceBand::Day);
    assert_eq!(band(DAY_WITHIN + 1), DistanceBand::Far);
    // Straight, not by steps: 50 each way is about 71.
    assert_eq!(
        DistanceBand::between((0, 0), (50, 50), WIDTH),
        DistanceBand::Short
    );
}

#[test]
fn places_from_pois_and_ruins() {
    assert_eq!(PlaceKind::of_poi(PoiKind::IceCave), PlaceKind::Cave);
    assert_eq!(PlaceKind::of_poi(PoiKind::Mausoleum), PlaceKind::Crypt);
    assert_eq!(PlaceKind::of_poi(PoiKind::City), PlaceKind::Settlement);
    // Every place kind but the wilds and a quest's cellar has a poi kind.
    for kind in PlaceKind::ALL {
        let found = PoiKind::ALL
            .iter()
            .any(|&poi| PlaceKind::of_poi(poi) == kind);
        let made_up = matches!(kind, PlaceKind::Wilds | PlaceKind::Cellar);
        assert_eq!(found, !made_up, "{kind:?}");
    }
    assert_eq!(
        PlaceDescription::new(PlaceKind::Tower, true).condition,
        Condition::Ruined
    );
    assert_eq!(
        PlaceDescription::new(PlaceKind::Ruin, true).condition,
        Condition::Whole
    );
}
