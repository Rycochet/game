use super::*;

const SIDE: u32 = 100;

fn pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let at = ((y * SIDE + x) * 4) as usize;
    [bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]
}

fn rgb(colour: [f32; 3]) -> [u8; 4] {
    [colour[0] as u8, colour[1] as u8, colour[2] as u8, 255]
}

#[test]
fn corners_are_see_through() {
    let bytes = paint(SIDE, 1.0, 1.0);
    assert_eq!(bytes.len(), (SIDE * SIDE * 4) as usize);
    assert_eq!(pixel(&bytes, 0, 0)[3], 0);
    assert_eq!(pixel(&bytes, SIDE - 1, SIDE - 1)[3], 0);
}

#[test]
fn full_halves_are_red_and_blue() {
    let bytes = paint(SIDE, 1.0, 1.0);
    assert_eq!(pixel(&bytes, 25, 50), rgb(HEALTH));
    assert_eq!(pixel(&bytes, 75, 50), rgb(MANA));
}

#[test]
fn each_half_fills_from_the_bottom() {
    let bytes = paint(SIDE, 0.5, 0.0);
    assert_eq!(pixel(&bytes, 30, 70), rgb(HEALTH));
    assert_eq!(pixel(&bytes, 30, 30), rgb(EMPTY));
    assert_eq!(pixel(&bytes, 70, 70), rgb(EMPTY));
}

/// How bright a pixel is.
fn brightness(colour: [u8; 4]) -> u32 {
    colour[..3].iter().map(|&channel| u32::from(channel)).sum()
}

#[test]
fn borders_go_round_and_down_the_middle_lit_from_the_top_left() {
    let bytes = paint(SIDE, 1.0, 1.0);
    let plain = brightness(rgb(BORDER));
    // Round the edge: the top left faces the light, the bottom right away.
    let (top_left, bottom_right) = (pixel(&bytes, 17, 17), pixel(&bytes, 82, 82));
    assert!(brightness(top_left) > plain && brightness(bottom_right) < plain);
    // Down the middle each half stands up: Health's side of the line faces
    // away from the light, Mana's side towards it.
    assert!(brightness(pixel(&bytes, 49, 50)) < plain);
    assert!(brightness(pixel(&bytes, 50, 50)) > plain);
    assert_ne!(pixel(&bytes, 1, 50), rgb(EMPTY));
}
