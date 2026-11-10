//! A Windows `.ico` file: a short header, then one entry per size pointing at
//! that size's picture. Since Windows Vista each picture may be a whole PNG
//! file, which is what this writes.

/// An `.ico` holding the given square PNGs, each with its width in pixels
/// (256 at most).
pub(crate) fn ico(pngs: &[(u32, Vec<u8>)]) -> Vec<u8> {
    const HEADER: usize = 6;
    const ENTRY: usize = 16;
    let count = pngs.len() as u16;
    let mut file = Vec::new();
    // Reserved, 1 for an icon (2 is a cursor), how many pictures.
    for value in [0u16, 1, count] {
        file.extend(value.to_le_bytes());
    }
    let mut offset = HEADER + ENTRY * pngs.len();
    for (size, png) in pngs {
        // Width and height in one byte each, where 0 means 256.
        let side = if *size >= 256 { 0 } else { *size as u8 };
        file.extend([side, side, 0, 0]); // no palette, reserved
        file.extend(1u16.to_le_bytes()); // colour planes
        file.extend(32u16.to_le_bytes()); // bits a pixel
        file.extend((png.len() as u32).to_le_bytes());
        file.extend((offset as u32).to_le_bytes());
        offset += png.len();
    }
    for (_, png) in pngs {
        file.extend(png);
    }
    file
}
