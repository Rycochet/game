use shared::id::Id;
use shared::protocol::{Landed, ServerMessage};
use shared::vitals::Points;

use super::{Harmed, harmed};

fn hit(target: Option<Id>, harm: Points) -> ServerMessage {
    ServerMessage::AbilityLanded {
        who: target.is_none().then(|| Id::for_test(9)),
        target,
        landed: Landed::Hit { harm, left: 500 },
    }
}

#[test]
fn blows_that_harm_flash_whoever_took_them() {
    let mob = Id::for_test(1);
    let harm = Points::whole(3);
    assert_eq!(harmed(&hit(Some(mob), harm)), Some(Harmed::Other(mob)));
    assert_eq!(harmed(&hit(None, harm)), Some(Harmed::Player));
    let seen = ServerMessage::Harmed { id: mob, left: 10 };
    assert_eq!(harmed(&seen), Some(Harmed::Other(mob)));
}

#[test]
fn blows_that_do_no_harm_and_other_messages_flash_nobody() {
    assert_eq!(harmed(&hit(Some(Id::for_test(1)), Points::ZERO)), None);
    assert_eq!(harmed(&hit(None, Points::ZERO)), None);
    let fizzled = ServerMessage::AbilityLanded {
        who: None,
        target: Some(Id::for_test(1)),
        landed: Landed::Fizzled,
    };
    assert_eq!(harmed(&fizzled), None);
}
