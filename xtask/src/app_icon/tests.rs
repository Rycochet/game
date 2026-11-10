use super::*;

fn font() -> Vec<u8> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    fs::read(root.join(FONT)).unwrap()
}

#[test]
fn every_style_draws() {
    for style in [Style::Framed, Style::Small, Style::Square] {
        let png = render(&svg::icon(&font(), style).unwrap(), 32).unwrap();
        assert!(png.starts_with(b"\x89PNG"), "{style:?}");
    }
}

#[test]
fn the_square_is_filled_to_its_corners() {
    let svg = svg::icon(&font(), Style::Square).unwrap();
    let tree = resvg::usvg::Tree::from_str(&svg, &Default::default()).unwrap();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(16, 16).unwrap();
    let scale = 16.0 / tree.size().width();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    assert_eq!(pixmap.pixel(0, 0).unwrap().alpha(), 255);
}

#[test]
fn an_ico_points_at_each_png() {
    let pngs = [(16, vec![1, 2, 3]), (256, vec![4, 5])];
    let file = ico::ico(&pngs);
    assert_eq!(&file[..6], &[0, 0, 1, 0, 2, 0]);
    // The first entry: 16 square, 3 bytes, after the header and two entries.
    assert_eq!(file[6], 16);
    assert_eq!(&file[14..18], &3u32.to_le_bytes());
    assert_eq!(&file[18..22], &38u32.to_le_bytes());
    // 256 is written as 0.
    assert_eq!(file[22], 0);
    assert_eq!(&file[38..], &[1, 2, 3, 4, 5]);
}
