use shared::creature::CreatureKind;

use super::*;

#[test]
fn it_says_what_the_kind_is_and_how_much_it_can_carry() {
    let carried = Inventory {
        creature: CreatureKind::Kobold,
        ..Inventory::default()
    };
    let words = describe(&Sheet::new(CreatureKind::Kobold), &carried);
    assert!(words.starts_with("Kobold, Small"), "{words}");
    assert!(words.contains("Strength 7"));
    assert!(words.contains("Stealth 3"));
    assert!(words.contains("NightVision"));
    assert!(words.contains("Can carry "), "{words}");
    assert!(!words.contains("Light"), "{words}");
    assert!(words.contains("Speed 100%"));
    assert!(words.contains("Height: Average"));
}
