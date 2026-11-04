use super::*;

#[test]
fn a_basement_is_below_the_ground_whatever_its_index() {
    let info = |basement| LevelInfo {
        kind: super::LevelKind::Poi(PoiKind::Farm),
        ruined: false,
        building: true,
        index: 0,
        levels: 3,
        basement,
        bounds: (0, 0, 1, 1),
        arrival: (0, 0),
        entering_site: true,
    };
    assert_eq!([0, 1, 2].map(|n| info(true).storey(n)), [0, 1, -1]);
    assert_eq!([0, 1, 2].map(|n| info(false).storey(n)), [0, 1, 2]);
}

#[test]
fn cells_come_back_where_they_were() {
    let cells = [(-1, -1), (-8, -8), (-3, -6)];
    let mut mask = 0;
    for (x, y) in cells {
        let (chunk, bit) = level_chunk(x, y);
        assert_eq!(chunk, (-1, -1));
        mask |= 1 << bit;
    }
    let chunk = LevelChunk {
        chunk: (-1, -1),
        mask,
        cells: vec![Cell::Floor, Cell::Wall, Cell::Water],
        stairs: Vec::new(),
        remembered: false,
    };
    let back: Vec<_> = chunk.cells().map(|(x, y, _)| (x, y)).collect();
    // Bit order: north-west first, row by row.
    assert_eq!(back, [(-8, -8), (-3, -6), (-1, -1)]);
    assert_eq!(level_chunk(0, 0), ((0, 0), 0));
    assert_eq!(level_chunk(7, 7), ((0, 0), 63));
}
