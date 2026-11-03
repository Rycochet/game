use super::*;

#[test]
fn every_category_has_a_kind() {
    for category in ItemCategory::ALL {
        assert!(
            ItemKind::ALL.iter().any(|kind| kind.category() == category),
            "{category:?}"
        );
    }
}

#[test]
fn bags_are_containers_and_blades_are_weapons() {
    for kind in ItemKind::ALL {
        if kind.def().container.is_some() {
            assert_eq!(kind.category(), ItemCategory::Containers, "{kind:?}");
        }
        if kind.blade_family().is_some() {
            assert_eq!(kind.category(), ItemCategory::Weapons, "{kind:?}");
        }
    }
}
