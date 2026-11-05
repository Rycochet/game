use super::*;

#[test]
fn hashed_names_are_kept() {
    assert_eq!(
        how_long_to_keep("/hall-of-worlds-3f2a9c01b7d4e865.js"),
        KEEP
    );
    assert_eq!(
        how_long_to_keep("/hall-of-worlds-3f2a9c01b7d4e865_bg.wasm"),
        KEEP
    );
}

#[test]
fn other_names_are_checked() {
    assert_eq!(how_long_to_keep("/"), CHECK);
    assert_eq!(how_long_to_keep("/index.html"), CHECK);
    assert_eq!(how_long_to_keep("/assets/fonts/Lexend-Variable.ttf"), CHECK);
    assert_eq!(how_long_to_keep("/assets/items/deadbeef.png"), CHECK);
}
