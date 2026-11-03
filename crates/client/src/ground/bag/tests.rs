use super::*;

#[test]
fn every_bag_stays_within_its_cell_and_on_the_floor() {
    const HALF_CELL: f32 = 12.0;
    for load in Load::ALL {
        for variety in 0..VARIETIES {
            let model = hover_outline(&bag(load, variety));
            for position in &model.positions {
                assert!(position.iter().all(|axis| axis.is_finite()));
                assert!(position[0].abs() < HALF_CELL, "{load:?}: {position:?}");
                assert!(position[2].abs() < HALF_CELL, "{load:?}: {position:?}");
                assert!(position[1] > -1.0, "{load:?}: {position:?}");
            }
        }
    }
}

#[test]
fn fuller_bags_are_taller() {
    for pair in Load::ALL.windows(2) {
        assert!(size_of(pair[0]).height < size_of(pair[1]).height);
    }
}

#[test]
fn faces_look_outwards() {
    let model = bag(Load::Full, 0);
    // Half way round the belly, the normal points away from the middle.
    let ring = RINGS / 4;
    let point = ring * (SEGMENTS + 1);
    let (position, normal) = (model.positions[point], model.normals[point]);
    assert!(position[0] * normal[0] + position[2] * normal[2] > 0.0);
}

#[test]
fn varieties_differ() {
    assert_ne!(bag(Load::Half, 0).positions, bag(Load::Half, 1).positions);
}

#[test]
fn every_index_is_a_point() {
    for model in [bag(Load::Flat, 2), star(5.0, 1.0, 0.5)] {
        let points = model.positions.len() as u32;
        assert!(model.indices.iter().all(|&index| index < points));
        assert_eq!(model.indices.len() % 3, 0);
    }
}
