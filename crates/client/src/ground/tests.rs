use super::spawn::spot_hash;
use super::*;

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    fn starts<M>(system: impl IntoSystem<(), (), M>) {
        IntoSystem::into_system(system).initialize(&mut World::new());
    }
    starts(make_assets);
    starts(read_server);
    starts(find_hovered);
    starts(change_of_mind);
    starts(follow_click);
    starts(open_on_arrival);
    starts(show_bags);
}

#[test]
fn bags_side_by_side_are_told_apart() {
    let spot = |x: i64| shared::spot::Spot::new(x * 100, 3);
    let hashes: std::collections::BTreeSet<u32> = (0..8).map(|x| spot_hash(spot(x)) % 8).collect();
    assert!(hashes.len() > 1);
}
