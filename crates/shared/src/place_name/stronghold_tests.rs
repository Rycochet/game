use std::collections::BTreeSet;

use super::*;

/// Names of one kind for many seeds, at a middling height.
fn names(kind: PoiKind) -> Vec<String> {
    (0..2000)
        .map(|seed| place_name(kind, Seed::new(seed), 300).unwrap())
        .collect()
}

#[test]
fn castles_and_forts_have_their_kind_in_front_and_nothing_after() {
    for (kind, word) in [(PoiKind::Castle, "Castle"), (PoiKind::Fort, "Fort")] {
        for name in names(kind) {
            let words: Vec<&str> = name.split(' ').collect();
            assert_eq!(words.len(), 2, "{name}");
            assert_eq!(words[0], word, "{name}");
            assert!(crate::name::checked(&name).is_ok(), "{name}");
        }
    }
}

#[test]
fn castles_have_more_names_than_towns() {
    let distinct = |names: Vec<String>| {
        let bases: BTreeSet<String> = names
            .into_iter()
            .map(|name| name.split(' ').next_back().unwrap().to_owned())
            .collect();
        bases.len()
    };
    let castles = distinct(names(PoiKind::Castle));
    let towns = distinct(names(PoiKind::City));
    assert!(castles > towns, "castles {castles}, towns {towns}");
}

#[test]
fn no_word_twice() {
    for name in names(PoiKind::Fort) {
        let base = name.split(' ').next_back().unwrap().to_lowercase();
        let half = base.len() / 2;
        assert!(
            base.len() % 2 == 1 || base[..half] != base[half..],
            "{name}"
        );
    }
}

#[test]
fn golden_stronghold_names() {
    let found: Vec<String> = (1..=6)
        .map(|seed| {
            let kind = [PoiKind::Castle, PoiKind::Fort][seed as usize % 2];
            place_name(kind, Seed::new(seed), seed as i32 * 200).unwrap()
        })
        .collect();
    let expected = [
        "Fort Chipstone",
        "Castle Stagbury",
        "Fort Redlea",
        "Castle Vormont",
        "Fort Redridge",
        "Castle Bramfell",
    ];
    assert_eq!(found, expected);
}
