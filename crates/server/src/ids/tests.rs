use std::collections::BTreeSet;

use super::*;

#[test]
fn ids_are_version_6_and_all_different() {
    let ids: BTreeSet<_> = (0..10_000).map(|_| new_id()).collect();
    assert_eq!(ids.len(), 10_000);
    assert!(ids.iter().all(|id| id.0.get_version_num() == 6));
}

#[test]
fn a_save_reads_as_text() {
    let id = new_id();
    let text = ron::to_string(&id).unwrap();
    assert!(text.contains('-'), "{text}");
    assert_eq!(ron::from_str::<Id>(&text).unwrap(), id);
}
