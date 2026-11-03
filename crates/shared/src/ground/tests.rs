use super::*;

#[test]
fn heavier_never_looks_emptier() {
    let mut last = Load::Flat;
    for grams in (0..30_000).step_by(50) {
        let load = Load::of_weight(grams);
        assert!(load >= last);
        last = load;
    }
    assert_eq!(Load::of_weight(0), Load::Flat);
    assert_eq!(last, Load::Bulging);
}
