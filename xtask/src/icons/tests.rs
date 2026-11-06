use super::*;

/// A white square in the middle of a black one, as game-icons.net draws them.
const SAMPLE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><path d="M0 0h512v512H0z"/><path fill="#fff" d="M128 128h256v256H128z"/></svg>"##;

#[test]
fn the_background_is_left_out_and_the_icon_is_white() {
    let png = render(SAMPLE).expect("draws");
    let pixmap = resvg::tiny_skia::Pixmap::decode_png(&png).expect("a png");
    assert_eq!((pixmap.width(), pixmap.height()), (SIZE, SIZE));
    let corner = pixmap.pixel(0, 0).expect("in it");
    assert_eq!(corner.alpha(), 0, "the black square is gone");
    let middle = pixmap.pixel(SIZE / 2, SIZE / 2).expect("in it");
    assert_eq!(
        (middle.red(), middle.green(), middle.blue(), middle.alpha()),
        (255, 255, 255, 255)
    );
}

#[test]
fn an_icon_without_the_usual_background_is_refused() {
    assert!(render(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8"/>"#).is_err());
}
