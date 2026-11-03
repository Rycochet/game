use super::*;

#[test]
fn lexend_replaces_whatever_font_was_the_default() {
    let mut app = App::new();
    let mut fonts = Assets::<Font>::default();
    fonts
        .insert(AssetId::default(), Font::from_bytes(vec![1, 2, 3]))
        .unwrap();
    app.insert_resource(fonts);

    app.add_plugins(FontsPlugin);

    let fonts = app.world().resource::<Assets<Font>>();
    let font = fonts.get(AssetId::default()).unwrap();
    assert_eq!(font.data.data(), DEFAULT_FONT_BYTES);
}
