use super::*;
use crate::connections::tests::{TestClient, answers};
use shared::protocol::ClientMessage;

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

/// The test client, on its own, told of lightning for `salt`.
fn tell_one(test: &mut TestClient, salt: i64, natural: bool) -> Vec<ServerMessage> {
    tell(
        std::slice::from_mut(&mut test.client),
        &mut test.world,
        salt,
        natural,
    );
    answers(&test.end)
}

#[test]
fn those_under_a_flash_are_told_with_a_bolt_clear_of_them() {
    let mut test = TestClient::new();
    test.world.server_clock.pause();
    let (x, y, second) = flash(&test.world);
    answers(&test.end);

    test.client.character.world_position = (x, y);
    let told = tell_one(&mut test, second, true);
    let [
        ServerMessage::Lightning {
            flash: true,
            strike,
        },
    ] = told[..]
    else {
        panic!("{told:?}");
    };
    if let Some(strike) = strike {
        let (cx, cy) = (x * SCALE + SCALE / 2, y * SCALE + SCALE / 2);
        assert!((strike.x - cx).abs().max((strike.y - cy).abs()) > 16);
    }

    // Far from the storm, nothing.
    test.client.character.world_position = (x, y + 20_000);
    assert_eq!(tell_one(&mut test, second, true), []);
}

#[test]
fn asking_brings_lightning_whatever_the_weather_but_not_in_the_hall() {
    let mut test = TestClient::new();
    test.world.server_clock.pause();
    answers(&test.end);
    test.send(&[ClientMessage::DevLightning]);
    let told = tell_one(&mut test, 1234, false);
    assert!(
        matches!(told[..], [ServerMessage::Lightning { flash: true, .. }]),
        "{told:?}"
    );
    // Asked once, told once.
    assert_eq!(tell_one(&mut test, 1235, false), []);

    test.send(&[ClientMessage::DevToHall]);
    answers(&test.end);
    test.send(&[ClientMessage::DevLightning]);
    assert_eq!(tell_one(&mut test, 1236, false), []);
}
