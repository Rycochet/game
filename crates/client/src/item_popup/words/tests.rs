use shared::creature::CreatureKind;
use shared::id::Id;
use shared::item::{Item, ItemKind, Quality};
use shared::item_form::{ItemOrder, make_item};
use shared::lang::{Lang, Language};
use shared::rng::Seed;

use super::*;

const MINUS: char = '\u{2212}';

#[test]
fn numbers_keep_one_place_at_most() {
    assert_eq!(number(40.0), "40");
    assert_eq!(number(12.34), "12.3");
    assert_eq!(number(0.6), "0.6");
    assert_eq!(signed(40_000), "+40");
    assert_eq!(signed(-8_000), format!("{MINUS}8"));
    assert_eq!(signed(-600), format!("{MINUS}0.6"));
}

#[test]
fn an_apple_restores_health_until_moving_or_hurt() {
    let lang = Lang::new(Language::English);
    let apple = Item::new(Id::for_test(1), ItemKind::Apple);
    assert_eq!(
        bonus_lines(&apple, &lang),
        vec!["Restores 1 Health a second for 30 s; stops on moving or being hurt"]
    );
}

#[test]
fn armour_gives_toughness_and_costs_agility_and_mana() {
    let lang = Lang::new(Language::English);
    let order = ItemOrder::plain(ItemKind::TorsoArmour, Quality::Common);
    let human = CreatureKind::Human;
    let next = &mut || Id::for_test(2);
    let piece = make_item(&order, human, Id::for_test(1), Seed::new(3), None, next);
    let lines = bonus_lines(&piece, &lang);
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert!(lines[0].starts_with('+') && lines[0].ends_with(" Toughness"));
    assert!(lines[1].starts_with(MINUS) && lines[1].ends_with(" Agility"));
    assert!(lines[2].starts_with(MINUS) && lines[2].ends_with(" most Mana"));
}

#[test]
fn a_ring_gives_nothing() {
    let lang = Lang::new(Language::English);
    let ring = Item::new(Id::for_test(1), ItemKind::Ring);
    assert!(bonus_lines(&ring, &lang).is_empty());
}

#[test]
fn a_hair_shirt_costs_five_percent_will() {
    use shared::item_form::ItemForm;
    use shared::material::Material;
    use shared::underclothes::{UnderPiece, roll};
    let lang = Lang::new(Language::English);
    let form = (0..5000u64)
        .map(|seed| {
            roll(
                &mut Seed::new(seed).rng(),
                UnderPiece::Shirt,
                Quality::Common,
            )
        })
        .find(|form| form.material == Material::Haircloth)
        .expect("a hair shirt");
    let mut shirt = Item::new(Id::for_test(1), UnderPiece::Shirt.kind());
    shirt.form = Some(ItemForm::Under(form));
    assert_eq!(bonus_lines(&shirt, &lang), vec![format!("{MINUS}5% Will")]);
}
