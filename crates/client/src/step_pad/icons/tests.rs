use super::*;

/// How much of the pixel at (x, y) is drawn, 0 to 255.
fn alpha(image: &Image, x: usize, y: usize) -> u8 {
    let bytes = image.data.as_ref().expect("drawn on the CPU");
    bytes[(y * SIZE + x) * 4 + 3]
}

/// How many pixels are drawn in the given rows, across the whole picture.
fn drawn_in_rows(image: &Image, rows: std::ops::Range<usize>) -> usize {
    rows.flat_map(|y| (0..SIZE).map(move |x| (x, y)))
        .filter(|&(x, y)| alpha(image, x, y) > 128)
        .count()
}

#[test]
fn up_and_down_ladders_are_narrow_at_opposite_ends() {
    let up = ladder(true);
    let down = ladder(false);
    // Widest across, between the rails' outsides, in a band of rows.
    let width = |image: &Image, y: usize| {
        let drawn: Vec<usize> = (0..SIZE).filter(|&x| alpha(image, x, y) > 128).collect();
        drawn.last().expect("rails") - drawn.first().expect("rails")
    };
    assert!(
        width(&up, 10) < width(&up, 54),
        "Up narrows towards the top"
    );
    assert!(
        width(&down, 54) < width(&down, 10),
        "Down narrows towards the bottom"
    );
}

#[test]
fn a_ladders_rungs_crowd_together_at_its_far_end() {
    let up = ladder(true);
    // Rows with a rung drawn across the middle, between the rails.
    let rung_rows: Vec<usize> = (0..SIZE).filter(|&y| alpha(&up, 32, y) > 128).collect();
    let mut starts: Vec<usize> = rung_rows
        .iter()
        .copied()
        .filter(|&y| y == 0 || !rung_rows.contains(&(y - 1)))
        .collect();
    starts.sort();
    assert_eq!(starts.len(), 5, "five rungs: {starts:?}");
    let gaps: Vec<usize> = starts.windows(2).map(|pair| pair[1] - pair[0]).collect();
    assert!(gaps.windows(2).all(|pair| pair[0] <= pair[1]), "{gaps:?}");
    assert!(gaps.first() < gaps.last(), "{gaps:?}");
}

#[test]
fn arrows_point_their_way() {
    let up = arrow(Pointing::Up);
    assert!(drawn_in_rows(&up, 12..20) > 0);
    assert_eq!(drawn_in_rows(&up, 56..64), 0);
    let down = arrow(Pointing::Down);
    assert_eq!(drawn_in_rows(&down, 0..8), 0);
    // Left and right are the same, mirrored.
    let (left, right) = (arrow(Pointing::Left), arrow(Pointing::Right));
    for y in 0..SIZE {
        for x in 0..SIZE {
            // Within a sample: points right on an edge may round either way.
            let (a, b) = (alpha(&left, x, y), alpha(&right, SIZE - 1 - x, y));
            assert!(a.abs_diff(b) <= 16, "at ({x}, {y}): {a} and {b}");
        }
    }
    assert!(alpha(&left, 14, 32) > 128, "the tip is on the left");
}

#[test]
fn the_keypad_has_twelve_keys_less_one() {
    let pad = keypad();
    // Down the middle of the first column: four keys, and the frame twice.
    let column: Vec<bool> = (0..SIZE).map(|y| alpha(&pad, 19, y) > 128).collect();
    let runs = column.windows(2).filter(|pair| !pair[0] && pair[1]).count();
    assert_eq!(runs, 6);
    // The wide 0 fills the gap between the first two columns.
    assert!(alpha(&pad, 25, 52) > 128);
    assert!(alpha(&pad, 25, 14) < 128);
}
