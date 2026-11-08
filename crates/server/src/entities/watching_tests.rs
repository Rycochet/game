//! Whether something wary watches a player, for Sneaking.

use shared::creature::CreatureKind;

use super::tests::{beside_a_mob, player};
use super::watched_warily;

fn watched(kind: CreatureKind) -> bool {
    let test = beside_a_mob(kind, 2);
    let visit = test.client.visit.as_ref().unwrap();
    let character = test.client.character_id;
    watched_warily(
        &test.world,
        visit.population(),
        player(&test),
        test.client.inventory.creature,
        character.as_ref(),
    )
}

#[test]
fn only_timid_or_territorial_creatures_watch_warily() {
    assert!(watched(CreatureKind::Chicken));
    assert!(watched(CreatureKind::Wolf));
    assert!(!watched(CreatureKind::Goblin), "hostile: it would attack");
}
