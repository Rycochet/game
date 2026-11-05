use super::*;

#[test]
fn skips_files_compressed_already() {
    assert!(!worth_trying(Path::new("assets/items/ring.png")));
    assert!(!worth_trying(Path::new("game.wasm.br")));
    assert!(worth_trying(Path::new("hall-of-worlds-0123_bg.wasm")));
    assert!(worth_trying(Path::new("assets/shaders/clouds.wesl")));
    assert!(worth_trying(Path::new("LICENCE")));
}

#[test]
fn compressed_copy_sits_beside_the_file() {
    assert_eq!(
        with_br(Path::new("web/game.wasm")),
        Path::new("web/game.wasm.br")
    );
}

#[test]
fn brotli_gives_back_the_same_bytes() {
    let data: Vec<u8> = (0..50_000u32).map(|n| (n % 251) as u8).collect();
    let compressed = brotli(&data).unwrap();
    assert!(compressed.len() < data.len() / 10);
    let mut back = Vec::new();
    ::brotli::BrotliDecompress(&mut &compressed[..], &mut back).unwrap();
    assert_eq!(back, data);
}
