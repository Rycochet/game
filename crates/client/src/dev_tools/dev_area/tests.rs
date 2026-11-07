use super::*;

#[test]
fn reads_every_choice() {
    let area = parse_dev_area("64, moat, Wolf, 5, test").unwrap();
    assert_eq!(area.size, 64);
    assert_eq!(area.layout, DevLayout::Moat);
    assert_eq!(
        area.mobs,
        DevMobs::Kind {
            kind: CreatureKind::Wolf,
            count: 5
        }
    );
    assert_eq!(area.seed, DevArea::seed_from_text("test"));
}

#[test]
fn missing_choices_are_as_the_panel_starts() {
    let area = parse_dev_area("").unwrap();
    assert_eq!(area.size, 32);
    assert_eq!(area.layout, DevLayout::Empty);
    assert_eq!(area.mobs, DevMobs::Default);
    assert_eq!(area.seed, 1);
    assert_eq!(parse_dev_area("16,cover,none").unwrap().mobs, DevMobs::None);
}

#[test]
fn wrong_names_say_what_would_do() {
    let why = parse_dev_area("16,maze").unwrap_err();
    assert!(why.contains("Moat"), "{why}");
    let why = parse_dev_area("16,empty,dragon").unwrap_err();
    assert!(why.contains("Wolf"), "{why}");
    assert!(parse_dev_area("big").is_err());
}

#[test]
fn reads_biomes_and_kinds_of_place() {
    use shared::climate::Biome;
    use shared::poi::PoiKind;
    let forest = parse_dev_area("128,boreal forest").unwrap();
    assert_eq!(forest.layout, DevLayout::Biome(Biome::BorealForest));
    let tower = parse_dev_area("64,StoneTower,default").unwrap();
    assert_eq!(tower.layout, DevLayout::Building(PoiKind::StoneTower));
}
