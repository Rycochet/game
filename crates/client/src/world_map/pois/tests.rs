use super::*;

#[test]
fn pois_are_shown_only_as_far_as_the_server_sends() {
    const { assert!(RANGE <= shared::protocol::WORLD_MAP_SIGHT) };
}
