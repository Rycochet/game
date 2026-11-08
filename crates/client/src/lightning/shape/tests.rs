use super::*;

#[test]
fn the_main_stroke_runs_from_the_top_to_the_origin() {
    for seed in 0..50 {
        let pieces = segments(seed);
        assert_eq!(pieces[0].from, Vec3::Y);
        let last = pieces[STEPS - 1];
        assert!(last.to.length() < 1e-5, "{seed}: {:?}", last.to);
        // Each piece goes down, and joins the one before.
        for pair in pieces[..STEPS].windows(2) {
            assert_eq!(pair[0].to, pair[1].from);
            assert!(pair[1].to.y < pair[1].from.y);
        }
    }
}

#[test]
fn forks_are_thinner_stay_above_ground_and_vary() {
    let mut lengths = std::collections::BTreeSet::new();
    for seed in 0..50 {
        let pieces = segments(seed);
        assert!(pieces.len() > STEPS);
        for fork in &pieces[STEPS..] {
            assert_eq!(fork.width, FORK_WIDTH);
            assert!(fork.to.y > 0.0);
        }
        lengths.insert(pieces.len());
    }
    assert!(lengths.len() > 3);
}

#[test]
fn the_same_seed_gives_the_same_bolt() {
    assert_eq!(segments(7), segments(7));
    assert_ne!(segments(7), segments(8));
}
