use super::parts::{Affix, ENDINGS, STARTS, affixes};
use super::*;

const TOWNS: [PoiKind; 4] = [
    PoiKind::Hamlet,
    PoiKind::Village,
    PoiKind::Town,
    PoiKind::City,
];

/// Names for many seeds of one kind and height.
fn names(kind: PoiKind, metres: i32) -> Vec<String> {
    (0..2000)
        .map(|seed| town_name(kind, Seed::new(seed), metres).unwrap())
        .collect()
}

#[test]
fn only_towns_have_names() {
    for kind in PoiKind::ALL {
        assert_eq!(
            town_name(kind, Seed::new(1), 0).is_some(),
            kind.is_town(),
            "{kind:?}"
        );
    }
}

#[test]
fn the_same_seed_gives_the_same_name() {
    for kind in TOWNS {
        for seed in 0..50 {
            let seed = Seed::new(seed);
            assert_eq!(town_name(kind, seed, 300), town_name(kind, seed, 300));
        }
    }
}

#[test]
fn golden_names() {
    let found: Vec<String> = (1..=8)
        .map(|seed| {
            town_name(TOWNS[seed as usize % 4], Seed::new(seed), seed as i32 * 150).unwrap()
        })
        .collect();
    let expected = [
        "Bradcrag",
        "Bexham Market",
        "Marbrook Town",
        "Chipridge End",
        "Oakstead",
        "Morfell",
        "Wyncote",
        "Goldcombe",
    ];
    assert_eq!(found, expected);
}

#[test]
fn names_are_plain_letters() {
    for kind in TOWNS {
        for name in names(kind, 300) {
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '\''),
                "{name}"
            );
            assert!(crate::name::checked(&name).is_ok(), "{name}");
        }
    }
}

/// The extra word a name has, if any, and whether it came before.
fn affix_of(kind: PoiKind, name: &str) -> Vec<Affix> {
    affixes(kind)
        .unwrap()
        .iter()
        .copied()
        .filter(|affix| match affix {
            Affix::Before(word) => name.starts_with(&format!("{word} ")),
            Affix::After(word) => name.ends_with(&format!(" {word}")),
        })
        .collect()
}

#[test]
fn at_most_one_extra_word_from_its_own_size() {
    for kind in TOWNS {
        let mut with = 0;
        for name in names(kind, 300) {
            let found = affix_of(kind, &name);
            let words = name.split(' ').count();
            if words == 1 {
                assert!(found.is_empty(), "{name}");
                continue;
            }
            with += 1;
            assert_eq!(found.len(), 1, "{name}: {found:?}");
            let before = matches!(found[0], Affix::Before(_));
            let after = affixes(kind).unwrap().iter().any(
                |affix| matches!(affix, Affix::After(word) if name.ends_with(&format!(" {word}"))),
            );
            assert!(!(before && after), "both: {name}");
        }
        // About half have one.
        assert!((800..1200).contains(&with), "{kind:?}: {with}");
    }
}

/// How many names end with an ending of a band.
fn in_band(names: &[String], band: Altitude) -> usize {
    names
        .iter()
        .filter(|name| {
            let base = name.split(' ').find(|word| {
                STARTS.iter().any(|start| word.starts_with(start))
                    && ENDINGS.iter().any(|(ending, _)| word.ends_with(ending))
            });
            base.is_some_and(|base| {
                ENDINGS
                    .iter()
                    .any(|(ending, own)| *own == band && base.ends_with(ending))
            })
        })
        .count()
}

#[test]
fn height_makes_its_endings_likelier() {
    let low = names(PoiKind::Village, 0);
    let high = names(PoiKind::Village, 2000);
    assert!(in_band(&low, Altitude::Low) > 2 * in_band(&high, Altitude::Low));
    assert!(in_band(&high, Altitude::High) > 2 * in_band(&low, Altitude::High));
    // Still possible elsewhere.
    assert!(in_band(&low, Altitude::High) > 0);
}

#[test]
fn bands_by_metres() {
    assert_eq!(Altitude::of_metres(-5), Altitude::Low);
    assert_eq!(Altitude::of_metres(LOW_BELOW - 1), Altitude::Low);
    assert_eq!(Altitude::of_metres(LOW_BELOW), Altitude::Middle);
    assert_eq!(Altitude::of_metres(HIGH_FROM - 1), Altitude::Middle);
    assert_eq!(Altitude::of_metres(HIGH_FROM), Altitude::High);
}

#[test]
fn names_are_varied() {
    let found = names(PoiKind::Town, 300);
    let distinct: std::collections::BTreeSet<&String> = found.iter().collect();
    assert!(distinct.len() > 1500, "{}", distinct.len());
}

#[test]
fn joins_read_well() {
    for name in names(PoiKind::Hamlet, 300) {
        let lower = name.to_lowercase();
        let letters: Vec<char> = lower.chars().collect();
        assert!(
            !letters
                .windows(3)
                .any(|three| three[0] == three[1] && three[1] == three[2]),
            "{name}"
        );
    }
}
