use shared::creature::CreatureKind;

use super::*;

#[test]
fn it_says_what_the_kind_is_and_how_much_it_can_carry() {
    let carried = Inventory {
        creature: CreatureKind::Kobold,
        ..Inventory::default()
    };
    let words = describe(&Sheet::new(CreatureKind::Kobold), &carried, None);
    assert!(words.starts_with("Kobold, Small"), "{words}");
    let strength = Sheet::new(CreatureKind::Kobold).attributes.strength;
    assert!(
        words.contains(&format!("Body: Strength {strength}, Agility ")),
        "{words}"
    );
    assert!(words.contains("Spirit: Presence "), "{words}");
    assert!(!words.contains("Sneaking"), "skills are on the Skills page");
    // Abilities have a page of their own.
    assert!(!words.contains("Night Vision"), "{words}");
    assert!(words.contains("Can carry "), "{words}");
    assert!(!words.contains("Light"), "{words}");
    assert!(words.contains("Speed 100%"));
    assert!(words.contains("Height: Average"));
    assert!(!words.contains("Health"), "{words}");
}

#[test]
fn it_says_where_health_and_mana_come_from() {
    let mut sheet = Sheet::new(CreatureKind::Human);
    sheet.attributes.toughness = 100;
    sheet.attributes.will = 60;
    sheet.vitals.health.practice = 100_000;
    let told = |most| shared::vitals::VitalNow {
        value: shared::vitals::Points::whole(1),
        most,
        refills_in: 0,
    };
    // 20 from Toughness and 2 from practice: the 3 left are from playing.
    let lines = vitals_lines(&sheet, Some((told(25), told(12))));
    assert_eq!(lines[0], "Health 25 (Toughness 20, +3 play, +2 practice)");
    assert_eq!(lines[1], "Mana 12 (Will 12)");
}
