use std::collections::BTreeSet;

use super::*;
use crate::gender::Gender;
use crate::rng::Seed;

/// Peoples' names must already be names as kept (`name::checked`), short and
/// capitalised, never a letter three times running.
fn assert_name(name: &str) {
    let length = name.chars().count();
    assert!((2..=16).contains(&length), "{name}");
    assert!(name.starts_with(|c: char| c.is_ascii_uppercase()), "{name}");
    assert!(!crate::creature::three_alike(name), "{name}");
    assert_eq!(crate::name::checked(name).as_deref(), Ok(name), "{name}");
}

#[test]
fn a_household_shares_its_family_name_and_members_differ() {
    let household = Household::new(CreatureKind::Human, Seed::new(5));
    let members: Vec<Person> = (0..20).map(|n| household.member(n)).collect();
    for member in &members {
        assert_eq!(member.family, household.family);
        assert_eq!(member.kind, CreatureKind::Human);
        assert_name(&member.given);
    }
    let given: BTreeSet<&str> = members.iter().map(|m| m.given.as_str()).collect();
    assert!(given.len() > 12, "{given:?}");
    assert_eq!(
        household.member(3),
        Household::new(CreatureKind::Human, Seed::new(5)).member(3)
    );
}

#[test]
fn every_people_has_family_names_and_animals_none() {
    for kind in CreatureKind::ALL {
        let families: BTreeSet<Option<String>> = (0..100)
            .map(|n| Household::new(kind, Seed::new(n)).family)
            .collect();
        if is_people(kind) {
            assert!(families.len() > 30, "{kind:?}: {families:?}");
            for family in families {
                assert_name(&family.expect("a people's family name"));
            }
        } else {
            assert_eq!(families, BTreeSet::from([None]), "{kind:?}");
        }
    }
}

#[test]
fn members_genders_are_rolled_like_mobs() {
    let household = Household::new(CreatureKind::Dwarf, Seed::new(9));
    let count = |gender| {
        (0..1000)
            .filter(|&n| household.member(n).gender == gender)
            .count()
    };
    let (male, female, neutral) = (
        count(Gender::Male),
        count(Gender::Female),
        count(Gender::Neutral),
    );
    assert!((380..520).contains(&male), "{male}");
    assert!((380..520).contains(&female), "{female}");
    assert!((50..150).contains(&neutral), "{neutral}");
}

#[test]
fn pets_and_bosses_have_names_and_every_value_turns_up() {
    let pets: Vec<Pet> = (0..200).map(|n| Pet::new(Seed::new(n))).collect();
    for pet in &pets {
        assert_name(&pet.name);
    }
    for kind in PetKind::ALL {
        assert!(pets.iter().any(|pet| pet.kind == kind), "{kind:?}");
    }
    for kind in CreatureKind::ALL {
        for n in 0..50 {
            assert_name(&Boss::new(kind, Seed::new(n)).name);
        }
    }
    let epithets: BTreeSet<Epithet> = (0..400)
        .map(|n| Boss::new(CreatureKind::Wolf, Seed::new(n)).epithet)
        .collect();
    assert_eq!(epithets.len(), Epithet::ALL.len());
}

/// Golden values: changing a list, a derived seed's purpose or the order of
/// draws renames people in every world. Only change them on purpose.
#[test]
fn golden_names() {
    let household = Household::new(CreatureKind::Human, Seed::new(42));
    let person = household.member(0);
    assert_eq!(
        (
            person.given.as_str(),
            person.family.as_deref(),
            person.gender
        ),
        GOLDEN_PERSON
    );
    let dwarf = Household::new(CreatureKind::Dwarf, Seed::new(42));
    assert_eq!(dwarf.family.as_deref(), GOLDEN_DWARF_FAMILY);
    let pet = Pet::new(Seed::new(42));
    assert_eq!((pet.kind, pet.name.as_str()), GOLDEN_PET);
    let boss = Boss::new(CreatureKind::Wolf, Seed::new(42));
    assert_eq!((boss.name.as_str(), boss.epithet), GOLDEN_WOLF);
    let boss = Boss::new(CreatureKind::Orc, Seed::new(42));
    assert_eq!((boss.name.as_str(), boss.epithet), GOLDEN_ORC);
}

const GOLDEN_PERSON: (&str, Option<&str>, Gender) = ("Oseric", Some("Langwick"), Gender::Male);
const GOLDEN_DWARF_FAMILY: Option<&str> = Some("Rockhand");
const GOLDEN_PET: (PetKind, &str) = (PetKind::Chicken, "Mo");
const GOLDEN_WOLF: (&str, Epithet) = ("Graak", Epithet::Old);
const GOLDEN_ORC: (&str, Epithet) = ("Groaash", Epithet::Old);
