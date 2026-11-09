use super::*;
use crate::rng::Seed;

#[test]
fn names_are_short_capitalised_and_never_three_alike() {
    for kind in CreatureKind::ALL {
        for n in 0..200 {
            let name = kind.random_name(&mut Seed::new(n).rng());
            let length = name.chars().count();
            assert!((3..=14).contains(&length), "{kind:?}: {name}");
            assert!(name.starts_with(|c: char| c.is_ascii_uppercase()), "{name}");
            assert!(!three_alike(&name), "{name}");
            // Suggested as typed, so it must already be a name as kept.
            assert_eq!(crate::name::checked(&name).as_deref(), Ok(name.as_str()));
        }
    }
}

#[test]
fn the_same_rolls_give_the_same_name_and_others_differ() {
    let name = |n| CreatureKind::Dwarf.random_name(&mut Seed::new(n).rng());
    assert_eq!(name(7), name(7));
    let different: std::collections::BTreeSet<String> = (0..50).map(name).collect();
    assert!(different.len() > 30, "{different:?}");
}

#[test]
fn three_alike_is_found_in_any_case() {
    assert!(three_alike("Ssss"));
    assert!(three_alike("Isssa"));
    assert!(!three_alike("Issa"));
}
