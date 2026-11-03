use bevy::math::{URect, UVec2};

use super::Shelves;

#[test]
fn cells_fill_rows_then_start_new_ones() {
    let mut shelves = Shelves::new(UVec2::new(100, 100), 4);
    let a = shelves.place(UVec2::new(40, 30)).unwrap();
    let b = shelves.place(UVec2::new(40, 30)).unwrap();
    let c = shelves.place(UVec2::new(40, 30)).unwrap();
    assert_eq!(a, URect::new(0, 0, 40, 30));
    assert_eq!(b, URect::new(44, 0, 84, 30));
    // No room left along the first row.
    assert_eq!(c, URect::new(0, 34, 40, 64));
}

#[test]
fn short_pictures_get_a_row_of_their_own() {
    let mut shelves = Shelves::new(UVec2::new(200, 200), 0);
    shelves.place(UVec2::new(10, 100)).unwrap();
    // Too short for the first row: it would leave most of it empty.
    assert_eq!(shelves.place(UVec2::new(10, 20)).unwrap().min.y, 100);
    // But a nearly as tall one shares it.
    assert_eq!(shelves.place(UVec2::new(10, 90)).unwrap().min.y, 0);
}

#[test]
fn a_full_page_takes_no_more() {
    let mut shelves = Shelves::new(UVec2::new(64, 64), 2);
    assert!(shelves.place(UVec2::new(64, 64)).is_some());
    assert!(shelves.place(UVec2::new(1, 1)).is_none());
    assert!(
        Shelves::new(UVec2::new(64, 64), 0)
            .place(UVec2::new(65, 1))
            .is_none()
    );
}

#[test]
fn cells_never_overlap() {
    let mut shelves = Shelves::new(UVec2::new(1024, 1024), 8);
    let sizes = [
        (64, 64),
        (64, 128),
        (128, 64),
        (64, 256),
        (128, 128),
        (32, 64),
    ];
    let mut placed: Vec<URect> = Vec::new();
    for n in 0..400 {
        let (x, y) = sizes[n % sizes.len()];
        let Some(rect) = shelves.place(UVec2::new(x, y)) else {
            break;
        };
        assert!(rect.max.x <= 1024 && rect.max.y <= 1024);
        for other in &placed {
            assert!(
                rect.intersect(*other).is_empty(),
                "{rect:?} overlaps {other:?}"
            );
        }
        placed.push(rect);
    }
    assert!(placed.len() > 50);
}
