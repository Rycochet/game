use super::*;
use shared::item_form::Cut;

const CUTS: [Cut; 7] = [
    Cut::Cushion,
    Cut::Emerald,
    Cut::Oval,
    Cut::Pear,
    Cut::Princess,
    Cut::Round,
    Cut::Trillion,
];

fn turn(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b - a).perp_dot(c - b)
}

#[test]
fn every_outline_goes_one_way_round_and_never_turns_back() {
    for cut in CUTS {
        let points = outline(cut);
        for i in 0..points.len() {
            let (a, b) = (points[i], points[(i + 1) % points.len()]);
            let c = points[(i + 2) % points.len()];
            assert!(turn(a, b, c) > -1e-4, "{cut:?} at {i}");
        }
    }
}

#[test]
fn every_outline_fits_the_picture() {
    for cut in CUTS {
        for point in outline(cut) {
            assert!(point.length() <= 1.2, "{cut:?} {point}");
        }
    }
}

#[test]
fn a_trillion_has_a_corner_at_the_top_and_a_pear_its_point() {
    assert!(outline(Cut::Trillion)[0].y < -0.95);
    assert_eq!(outline(Cut::Pear)[0], Vec2::new(0.0, -1.0));
    assert_eq!(TRILLION_POINTS % 3, 0);
}

#[test]
fn stones_of_different_seeds_have_different_lumps() {
    let positions = |seed| {
        let mesh = uncut_mesh(seed);
        let points = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        points.as_float3().unwrap().to_vec()
    };
    assert_eq!(positions(1), positions(1));
    assert_ne!(positions(1), positions(2));
}
