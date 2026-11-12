//! Every value is worded in every language, and the English reads right.

use super::*;
use crate::lang::{Lang, Language, Selector};
use crate::rng::Seed;

fn assert_worded(lang: &Lang, values: impl IntoIterator<Item = impl Selector>) {
    for value in values {
        let id = format!("{}-{}", value.fact(), value.key());
        let words = lang.phrase(&value);
        assert!(
            words.as_deref().is_ok_and(|words| !words.is_empty()),
            "{id}: {words:?}"
        );
    }
}

#[test]
fn every_value_has_words_in_every_language() {
    for language in Language::ALL {
        let lang = Lang::new(language);
        assert_worded(&lang, Direction::ALL);
        assert_worded(&lang, DistanceBand::ALL);
        assert_worded(&lang, Epithet::ALL);
        assert_worded(&lang, PetKind::ALL);
        assert_worded(&lang, PlaceKind::ALL);
        assert_worded(&lang, [Condition::Whole, Condition::Ruined]);
    }
}

#[test]
fn english_names_and_places() {
    let lang = Lang::new(Language::English);
    let person = Person {
        kind: CreatureKind::Human,
        gender: crate::gender::Gender::Female,
        given: "Wenna".into(),
        family: Some("Ashford".into()),
    };
    assert_eq!(person.full_name(&lang).unwrap(), "Wenna Ashford");
    let alone = Person {
        family: None,
        ..person
    };
    assert_eq!(alone.full_name(&lang).unwrap(), "Wenna");
    let boss = Boss {
        name: "Skarnak".into(),
        epithet: Epithet::OneEyed,
        ..Boss::new(CreatureKind::Wolf, Seed::new(1))
    };
    assert_eq!(boss.full_name(&lang).unwrap(), "Skarnak the One-Eyed");
    let tower = PlaceDescription::new(PlaceKind::Tower, true);
    assert_eq!(tower.describe(&lang).unwrap(), "ruined tower");
    let hut = PlaceDescription::new(PlaceKind::HerbalistHut, false).describe(&lang);
    assert_eq!(lang.rules().indefinite(&hut.unwrap()), "a herbalist's hut");
    assert_eq!(lang.phrase(&Direction::NorthEast).unwrap(), "north-east");
    assert_eq!(
        lang.phrase(&DistanceBand::Day).unwrap(),
        "most of a day away"
    );
}
