//! How each rune is drawn: straight strokes on a small grid, as runes were
//! made for carving. Everything that shows a rune (gates, the interface)
//! draws it from these, never from a font, so they are always the same.

/// A rune's grid, in steps: `x` across from the left, `y` down from the top.
pub const GLYPH_WIDTH: u8 = 4;
pub const GLYPH_HEIGHT: u8 = 6;

/// One stroke, from `(x, y)` to `(x, y)` on the grid.
pub type Stroke = [u8; 4];

/// Each of `RUNES`' strokes, in the same order.
pub const RUNE_STROKES: [&[Stroke]; 32] = [
    // ᚠ feoh: two branches up to the right.
    &[[1, 0, 1, 6], [1, 2, 3, 0], [1, 4, 3, 2]],
    // ᚢ ur
    &[[1, 0, 1, 6], [1, 0, 3, 2], [3, 2, 3, 6]],
    // ᚦ þorn
    &[[1, 0, 1, 6], [1, 2, 3, 3], [3, 3, 1, 4]],
    // ᚩ os: two branches down to the right.
    &[[1, 0, 1, 6], [1, 0, 3, 2], [1, 2, 3, 4]],
    // ᚱ rad
    &[[1, 0, 1, 6], [1, 0, 3, 1], [3, 1, 1, 3], [1, 3, 3, 6]],
    // ᚳ cen: a square hook at the top.
    &[[1, 0, 1, 6], [1, 0, 3, 0], [3, 0, 3, 2]],
    // ᚷ gyfu
    &[[0, 0, 4, 6], [4, 0, 0, 6]],
    // ᚹ wynn
    &[[1, 0, 1, 6], [1, 0, 3, 1], [3, 1, 1, 2]],
    // ᚻ hægl
    &[[0, 0, 0, 6], [4, 0, 4, 6], [0, 2, 4, 3], [0, 3, 4, 4]],
    // ᚾ nyd
    &[[2, 0, 2, 6], [0, 2, 4, 4]],
    // ᛁ is
    &[[2, 0, 2, 6]],
    // ᛄ ger: a small diamond on its stem.
    &[
        [2, 0, 2, 6],
        [2, 1, 4, 3],
        [4, 3, 2, 5],
        [2, 5, 0, 3],
        [0, 3, 2, 1],
    ],
    // ᛇ eoh
    &[[2, 0, 2, 6], [2, 0, 4, 1], [2, 6, 0, 5]],
    // ᛈ peorð
    &[[1, 0, 1, 6], [1, 1, 3, 0], [1, 5, 3, 6]],
    // ᛉ eolh
    &[[2, 0, 2, 6], [2, 2, 0, 0], [2, 2, 4, 0]],
    // ᛋ sigel
    &[[3, 0, 1, 2], [1, 2, 3, 4], [3, 4, 1, 6]],
    // ᛏ tiw
    &[[2, 0, 2, 6], [2, 0, 0, 2], [2, 0, 4, 2]],
    // ᛒ beorc
    &[
        [1, 0, 1, 6],
        [1, 0, 3, 1],
        [3, 1, 1, 3],
        [1, 3, 3, 5],
        [3, 5, 1, 6],
    ],
    // ᛖ eh
    &[[0, 0, 0, 6], [4, 0, 4, 6], [0, 0, 2, 2], [2, 2, 4, 0]],
    // ᛗ mann
    &[[0, 0, 0, 6], [4, 0, 4, 6], [0, 0, 4, 3], [4, 0, 0, 3]],
    // ᛚ lagu
    &[[1, 0, 1, 6], [1, 0, 3, 2]],
    // ᛝ ing: a large diamond.
    &[[2, 0, 4, 3], [4, 3, 2, 6], [2, 6, 0, 3], [0, 3, 2, 0]],
    // ᛟ ēþel
    &[[2, 0, 4, 2], [2, 0, 0, 2], [0, 2, 4, 6], [4, 2, 0, 6]],
    // ᛞ dæg
    &[[0, 0, 0, 6], [4, 0, 4, 6], [0, 0, 4, 6], [4, 0, 0, 6]],
    // ᚪ ac: as os, each branch ending in a hook down.
    &[
        [1, 0, 1, 6],
        [1, 0, 3, 2],
        [3, 2, 3, 3],
        [1, 2, 3, 4],
        [3, 4, 3, 5],
    ],
    // ᚣ yr: ur with a stem inside.
    &[[1, 0, 1, 6], [1, 0, 4, 2], [4, 2, 4, 6], [2, 1, 2, 6]],
    // ᛡ ior: a cross on its stem.
    &[[2, 0, 2, 6], [0, 1, 4, 3], [4, 1, 0, 3]],
    // ᛠ ear
    &[
        [2, 0, 2, 6],
        [2, 0, 0, 2],
        [2, 0, 4, 2],
        [0, 2, 0, 6],
        [4, 2, 4, 6],
    ],
    // ᚸ gar
    &[[0, 0, 4, 6], [4, 0, 0, 6], [2, 0, 2, 6]],
    // ᛣ calc
    &[[2, 0, 2, 6], [2, 4, 0, 6], [2, 4, 4, 6]],
    // ᛢ cweorð: forks at both ends.
    &[
        [2, 0, 2, 6],
        [2, 2, 0, 0],
        [2, 2, 4, 0],
        [2, 4, 0, 6],
        [2, 4, 4, 6],
    ],
    // ᛥ stan
    &[
        [2, 0, 2, 6],
        [0, 1, 2, 0],
        [2, 0, 4, 1],
        [0, 5, 2, 6],
        [2, 6, 4, 5],
    ],
];

/// The separator between a code's groups (᛫): a dot half way up, drawn as a
/// stroke with no length.
pub const SEPARATOR_STROKES: &[Stroke] = &[[2, 3, 2, 3]];
