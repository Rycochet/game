use super::*;

/// A north-up view a pixel per grid point, centred on `centre`, 40 pixels
/// across.
fn view(centre: (f64, f64)) -> Projection {
    Projection {
        centre,
        middle: (20.0, 20.0),
        units_per_pixel: 1.0,
        right: Projection::NORTH_UP.0,
        down: Projection::NORTH_UP.1,
    }
}

fn layer() -> Layer {
    Layer::new(40, 40)
}

fn cover(layer: &Layer, x: u32, y: u32) -> f32 {
    layer.cover[(y * layer.width + x) as usize]
}

#[test]
fn a_line_covers_its_pixels_once_however_many_pieces_overlap() {
    let mut layer = layer();
    let points: Vec<_> = (0..10).map(|x| (x, 0)).collect();
    layer.stroke(&view((5.0, 0.0)), &points, 1000, &|_| true, |_| 2.0);
    // Along the middle of it, fully covered, and never more.
    assert_eq!(cover(&layer, 20, 20), 1.0);
    assert!(layer.cover.iter().all(|&c| c <= 1.0));
    // Well away from it, nothing.
    assert_eq!(cover(&layer, 20, 30), 0.0);
}

#[test]
fn lines_are_left_out_where_the_land_isnt_known() {
    let mut layer = layer();
    let points: Vec<_> = (0..10).map(|x| (x, 0)).collect();
    layer.stroke(&view((5.0, 0.0)), &points, 1000, &|(x, _)| x < 5, |_| 2.0);
    assert!(cover(&layer, 17, 20) > 0.9);
    assert_eq!(cover(&layer, 24, 20), 0.0);
}

#[test]
fn a_line_is_drawn_the_nearer_way_round_the_planet() {
    // A road whose points carry on past the planet's edge (1000 wide), seen
    // from just inside the other side of the seam.
    let mut layer = layer();
    let points: Vec<_> = (1000..1010).map(|x| (x, 0)).collect();
    layer.stroke(&view((5.0, 0.0)), &points, 1000, &|_| true, |_| 2.0);
    assert_eq!(cover(&layer, 20, 20), 1.0);
}
