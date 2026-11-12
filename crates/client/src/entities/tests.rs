use super::moving::glide;
use super::*;

#[test]
fn others_glide_and_jump_only_when_far_behind() {
    assert_eq!(glide((0.0, 0.0), (1.0, 0.0), 0.25), (0.25, 0.0));
    assert_eq!(glide((0.0, 0.0), (1.0, 0.0), 2.0), (1.0, 0.0));
    assert_eq!(glide((0.0, 0.0), (50.0, 0.0), 0.25), (50.0, 0.0));
}

#[test]
fn every_kind_has_a_picture_that_is_there() {
    for kind in CreatureKind::ALL {
        let file = format!("{}/assets/{}", env!("CARGO_MANIFEST_DIR"), picture(kind));
        assert!(std::path::Path::new(&file).exists(), "{file}");
    }
}
