use super::*;
use shared::quest::QuestKind;

fn painted(ready: bool) -> Vec<u8> {
    let mut pixels = vec![0; 40 * 40 * 4];
    let mut canvas = Canvas {
        pixels: &mut pixels,
        width: 40,
        height: 40,
    };
    let look = MarkLook::new(QuestKind::Normal, ready);
    sign(&mut canvas, look, (20.0, 36.0), 32.0);
    pixels
}

fn at(pixels: &[u8], x: usize, y: usize) -> [u8; 4] {
    let i = (y * 40 + x) * 4;
    [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
}

#[test]
fn an_exclamation_mark_is_its_colour_down_the_middle_and_clear_at_the_sides() {
    let pixels = painted(false);
    let yellow = MarkLook::new(QuestKind::Normal, false).rgb;
    // The bar, and the dot below it.
    assert_eq!(at(&pixels, 20, 14)[..3], yellow);
    assert_eq!(at(&pixels, 20, 32)[..3], yellow);
    // Nothing far to the side.
    assert_eq!(at(&pixels, 4, 20)[3], 0);
}

#[test]
fn a_question_mark_differs_from_an_exclamation_mark() {
    assert_ne!(painted(true), painted(false));
    // Its hook reaches out to the side, where "!" has nothing.
    let hook = (
        at(&painted(true), 28, 13)[3],
        at(&painted(false), 28, 13)[3],
    );
    assert!(hook.0 > 0 && hook.1 == 0, "{hook:?}");
}
