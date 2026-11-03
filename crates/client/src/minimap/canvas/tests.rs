use super::*;

#[test]
fn polygon_distance_is_negative_inside() {
    let square = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
    assert_eq!(polygon_distance(&square, 5.0, 5.0), -5.0);
    assert_eq!(polygon_distance(&square, 13.0, 5.0), 3.0);
}

#[test]
fn half_see_through_sprites_mix_with_what_is_under_them() {
    let mut pixels = vec![0; 4];
    let mut canvas = Canvas {
        pixels: &mut pixels,
        width: 1,
        height: 1,
    };
    canvas.fill([0, 0, 0, 255]);
    let sprite = Sprite {
        size: 1,
        pixels: vec![200, 100, 0, 255],
    };
    canvas.sprite(&sprite, (0.5, 0.5), 0.5);
    assert_eq!(pixels, [100, 50, 0, 255]);
}

#[test]
fn sprites_off_the_edge_are_cut_off() {
    let mut pixels = vec![0; 16];
    let mut canvas = Canvas {
        pixels: &mut pixels,
        width: 2,
        height: 2,
    };
    let sprite = Sprite {
        size: 2,
        pixels: vec![255; 16],
    };
    canvas.sprite(&sprite, (0.0, 0.0), 1.0);
    assert_eq!(pixels[0..4], [255; 4]);
    assert_eq!(pixels[4..8], [0; 4]);
}
