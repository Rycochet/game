use shared::item::Quality;

use super::SkillRing;

#[test]
fn a_ring_fills_to_100_in_fifths_of_quality() {
    assert_eq!(SkillRing::of(0.0, false).fill, 0.0);
    assert_eq!(SkillRing::of(0.5, false).fill, 0.02, "a sliver at first");
    for (value, quality) in [
        (10.0, Quality::Common),
        (20.4, Quality::Uncommon),
        (45.0, Quality::Rare),
        (79.9, Quality::Epic),
        (80.0, Quality::Legendary),
        (100.0, Quality::Legendary),
    ] {
        let ring = SkillRing::of(value, false);
        assert_eq!(ring.quality, quality, "{value}");
        assert!(
            (ring.fill - (value / 100.0).max(0.02)).abs() < 1e-6,
            "{value}"
        );
    }
    assert_eq!(SkillRing::of(150.0, false).fill, 1.0);
}

#[test]
fn only_a_specialised_skill_fills_its_second_ring_from_100_to_120() {
    assert_eq!(SkillRing::of(110.0, false).special, 0.0);
    assert_eq!(SkillRing::of(90.0, true).special, 0.0);
    assert!((SkillRing::of(110.0, true).special - 0.5).abs() < 1e-6);
    assert_eq!(SkillRing::of(130.0, true).special, 1.0);
}
