use super::*;

fn plain(rgb: [u8; 3]) -> Choice {
    Choice {
        name: "Plain",
        colour: Some(rgb),
        pattern: Pattern::Plain,
    }
}

#[test]
fn colours_are_swatches_and_words_are_not() {
    let (mut pictures, mut images) = (PatternPictures::default(), Assets::<Image>::default());
    let red = swatch(&plain([200, 0, 0]), &mut pictures, &mut images);
    assert_eq!(red, Some(Swatch::Colour(Color::srgb_u8(200, 0, 0))));
    let tall = Choice {
        name: "Tall",
        colour: None,
        pattern: Pattern::Plain,
    };
    assert_eq!(swatch(&tall, &mut pictures, &mut images), None);
}

#[test]
fn a_pattern_is_painted_once_and_shared() {
    let (mut pictures, mut images) = (PatternPictures::default(), Assets::<Image>::default());
    let speckled = Choice {
        pattern: Pattern::Speckled([0, 0, 0]),
        ..plain([255, 255, 255])
    };
    let first = swatch(&speckled, &mut pictures, &mut images);
    let again = swatch(&speckled, &mut pictures, &mut images);
    assert!(matches!(first, Some(Swatch::Picture(_))));
    assert_eq!(first, again);
    assert_eq!(images.len(), 1);
}

#[test]
fn specks_show_over_the_base() {
    let picture = pattern_picture([255, 255, 255], Pattern::Speckled([0, 0, 0]));
    let bytes = picture.data.unwrap();
    let dark = bytes.chunks(4).filter(|pixel| pixel[0] == 0).count();
    let side = PATTERN_PIXELS as usize;
    assert!(dark > 4 && dark < side * side / 2, "{dark}");
}
