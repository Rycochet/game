use super::*;

#[test]
fn bands_by_drop() {
    assert_eq!(band_of(0), 0, "level water is still");
    assert_eq!(band_of(1), 1);
    assert_eq!(band_of(2), 2);
    assert_eq!(band_of(3), 2);
    assert_eq!(band_of(4), 3);
    assert_eq!(band_of(40), 3);
}

#[test]
fn packed_flows_decode_as_the_shader_does() {
    for (band, way, degrees) in [
        (1, Vec2::X, 0.0),
        (2, Vec2::Y, 90.0),
        (3, Vec2::NEG_X, 180.0),
        (1, Vec2::NEG_Y, 270.0),
    ] {
        let value = packed(band, way);
        let decoded = (value / 512.0).floor();
        assert_eq!(decoded as usize, band);
        assert!((value - decoded * 512.0 - degrees).abs() < 0.01, "{value}");
    }
}
