use super::*;
use crate::connections::tests::{TestClient, answers};

/// A world map point and second lightning flashes in, at the world's time
/// now (its clock stopped, so the weather stays put).
fn flash(world: &ServerWorld) -> (i64, i64, i64) {
    for y in (-3000..3000).step_by(40) {
        for x in (0..world.width).step_by(40) {
            for second in 0..40 {
                if world.lightning_at(x, y, second) {
                    return (x, y, second);
                }
            }
        }
    }
    panic!("no storm anywhere");
}

#[test]
fn only_those_under_a_flash_are_told() {
    let mut test = TestClient::new();
    test.world.server_clock.pause();
    let (x, y, second) = flash(&test.world);
    answers(&test.end);

    test.client.character.world_position = (x, y);
    tell(std::slice::from_mut(&mut test.client), &test.world, second);
    assert_eq!(answers(&test.end), [ServerMessage::Lightning]);

    // Far from the storm, nothing.
    test.client.character.world_position = (x, y + 20_000);
    tell(std::slice::from_mut(&mut test.client), &test.world, second);
    assert_eq!(answers(&test.end), []);
}

#[test]
fn none_in_the_hall() {
    let mut test = TestClient::new();
    test.world.server_clock.pause();
    let (x, y, second) = flash(&test.world);
    answers(&test.end);
    test.send(&[shared::protocol::ClientMessage::DevToHall]);
    test.client.character.world_position = (x, y);
    tell(std::slice::from_mut(&mut test.client), &test.world, second);
    assert_eq!(answers(&test.end), []);
}
