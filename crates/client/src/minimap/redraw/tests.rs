use super::*;
use crate::common::GameView;

fn from(centre: (f64, f64)) -> DrawnFrom {
    DrawnFrom {
        view: GameView::WorldMap,
        floor: None,
        open: false,
        centre,
        turn: (1.0, 0.0),
        scale: 1.0,
        size: (4, 4),
    }
}

#[test]
fn a_change_paints_it_all_and_otherwise_the_clock_comes_before_filling() {
    let now = from((1.0, 2.0));
    let last = from((1.0, 3.0));
    assert_eq!(Redraw::of(None, &now, (false, false, false)), Redraw::All);
    assert_eq!(
        Redraw::of(Some(&last), &now, (false, false, false)),
        Redraw::All
    );
    assert_eq!(
        Redraw::of(Some(&now), &now, (true, true, true)),
        Redraw::All
    );
    assert_eq!(
        Redraw::of(Some(&now), &now, (true, false, true)),
        Redraw::Clock
    );
    assert_eq!(
        Redraw::of(Some(&now), &now, (true, false, false)),
        Redraw::Filling
    );
    assert_eq!(
        Redraw::of(Some(&now), &now, (false, false, false)),
        Redraw::No
    );
}

#[test]
fn kept_land_comes_back_only_for_the_same_drawing() {
    let mut kept = KeptLand::default();
    let land = [1, 2, 3, 4];
    kept.keep(from((0.0, 0.0)), &land);
    let mut pixels = [0; 4];
    assert!(!kept.restore(&from((0.0, 1.0)), &mut pixels));
    assert_eq!(pixels, [0; 4]);
    assert!(kept.restore(&from((0.0, 0.0)), &mut pixels));
    assert_eq!(pixels, land);
    kept.forget();
    assert!(!kept.restore(&from((0.0, 0.0)), &mut pixels));
}
