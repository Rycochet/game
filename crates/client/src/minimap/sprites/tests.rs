use super::*;

#[test]
fn shrinking_averages_and_ignores_the_colour_of_see_through_pixels() {
    let data = [
        200, 0, 0, 255, /**/ 0, 255, 0, 0, //
        100, 0, 0, 255, /**/ 0, 255, 0, 0,
    ];
    let pixels = shrink_pixels(&data, 2, (0, 0, 2), 1);
    assert_eq!(pixels, [150, 0, 0, 127]);
}

#[test]
fn a_border_goes_round_what_is_solid() {
    let mut pixels = vec![0; 3 * 3 * 4];
    pixels[16..20].copy_from_slice(&[9, 9, 9, 255]);
    let bordered = with_border(&pixels, 3);
    for (i, pixel) in bordered.chunks(4).enumerate() {
        let expected = if i == 4 {
            [9, 9, 9, 255]
        } else {
            [0, 0, 0, 255]
        };
        assert_eq!(pixel, expected);
    }
}

#[test]
fn shrinking_can_take_part_of_a_picture() {
    let data = [
        1, 1, 1, 255, /**/ 2, 2, 2, 255, //
        3, 3, 3, 255, /**/ 4, 4, 4, 255,
    ];
    assert_eq!(shrink_pixels(&data, 2, (1, 1, 1), 1), [4, 4, 4, 255]);
}
