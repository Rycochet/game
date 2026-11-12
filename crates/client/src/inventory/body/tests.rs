use super::*;

#[test]
fn the_clothes_are_good_markup_with_a_slot_for_each_layer() {
    CLOTHES.assert_good();
    for layer in ["skin", "middle", "outer", "over"] {
        assert!(layer_named(layer).is_some());
        assert!(CLOTHES.source.contains(&format!(r#"id="{layer}""#)));
    }
    assert_eq!(layer_named("hand0"), None);
}

#[test]
fn held_has_a_column_not_a_row() {
    let rows: Vec<_> = parts(CreatureKind::Human).collect();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|part| part.location != Location::Held));
}
