use super::cut::{Plane, cut};
use super::fourfoot::Fourfoot;
use super::head::Head;
use super::leg::legs;
use super::mesh::Mesh;
use super::ring::ring;
use super::target;
use super::upright::Upright;

/// A box's six faces in `.obj` lines, its corners numbered from `first`.
fn faces(first: usize) -> String {
    let sides = [
        [0, 1, 3, 2],
        [4, 5, 7, 6],
        [0, 1, 5, 4],
        [2, 3, 7, 6],
        [0, 2, 6, 4],
        [1, 3, 7, 5],
    ];
    let face = |side: &[usize; 4]| {
        let corners: Vec<_> = side.iter().map(|at| format!("{}/1", first + at)).collect();
        format!("f {}\n", corners.join(" "))
    };
    sides.iter().map(face).collect()
}

/// A box's eight corners in `.obj` lines.
fn corners(low: [f64; 3], high: [f64; 3]) -> String {
    let mut text = String::new();
    for x in [low[0], high[0]] {
        for y in [low[1], high[1]] {
            for z in [low[2], high[2]] {
                text += &format!("v {x} {y} {z}\n");
            }
        }
    }
    text
}

/// A mesh of boxes: the skin's, then one small cube for each joint.
fn mesh(skin: &[([f64; 3], [f64; 3])], joints: &[(&str, [f64; 3])]) -> Mesh {
    let mut text = String::from("# a test mesh\n");
    for &(low, high) in skin {
        text += &corners(low, high);
    }
    for &(_, at) in joints {
        text += &corners(at.map(|v| v - 0.1), at.map(|v| v + 0.1));
    }
    text += "g body\n";
    for at in 0..skin.len() {
        text += &faces(1 + at * 8);
    }
    for (at, (name, _)) in joints.iter().enumerate() {
        text += &format!("g joint-{name}\n{}", faces(1 + (skin.len() + at) * 8));
    }
    Mesh::read(&text).expect("the test mesh reads")
}

/// A level trunk on two left legs (and their right twins), with the joints a
/// four-legged beast is measured by: 4 wide, 2 deep, 10 long, legs 3 tall.
fn beast() -> Mesh {
    let leg = |x: f64, z: f64| ([x, 0.0, z], [x + 1.0, 3.0, z + 1.0]);
    mesh(
        &[
            ([-2.0, 3.0, -1.0], [2.0, 5.0, 9.0]),
            leg(1.0, 0.0),
            leg(1.0, 7.0),
            leg(-2.0, 0.0),
            leg(-2.0, 7.0),
        ],
        &[
            ("l-upper-leg", [1.5, 4.0, 0.5]),
            ("l-shoulder", [1.5, 4.0, 7.5]),
            ("l-hand", [1.5, 1.0, 7.5]),
        ],
    )
}

#[test]
fn reads_points_skin_and_joints() {
    let mesh = beast();
    assert_eq!(mesh.skin.len(), 5 * 6);
    assert_eq!(mesh.joint("l-shoulder"), Some([1.5, 4.0, 7.5]));
    assert_eq!(mesh.joint("tail"), None);
    assert_eq!(mesh.bounds(), ([-2.0, 0.0, -1.0], [2.0, 5.0, 9.0]));
}

#[test]
fn refuses_what_is_not_a_mesh() {
    assert!(Mesh::read("v 1 2\n").is_err());
    assert!(Mesh::read("v 1 2 3\ng body\nf 1 2 3\n").is_err());
    assert!(Mesh::read("v 1 2 3\n").is_err());
}

#[test]
fn a_target_moves_the_points_it_names() {
    let mut mesh = beast();
    let before = mesh.points.clone();
    target::apply(&mut mesh, "# comment\r\n0 1 .5 -2\r\n\r\n3 0 0 1\r\n").unwrap();
    assert_eq!(
        mesh.points[0],
        [before[0][0] + 1.0, before[0][1] + 0.5, before[0][2] - 2.0]
    );
    assert_eq!(mesh.points[3][2], before[3][2] + 1.0);
    assert_eq!(mesh.points[1], before[1]);
    assert!(target::apply(&mut mesh, "99999 0 0 0").is_err());
    assert!(target::apply(&mut mesh, "0 1 2").is_err());
}

#[test]
fn a_plane_cuts_each_thing_as_its_own_outline() {
    let mesh = beast();
    // Through the legs: one outline each.
    let low = cut(&mesh, &Plane::facing([0.0, 1.0, 0.0], [0.0, 1.0, 0.0]));
    assert_eq!(low.len(), 4);
    // Across the trunk between the legs: only the trunk.
    let middle = cut(&mesh, &Plane::facing([0.0, 0.0, 4.0], [0.0, 0.0, 1.0]));
    assert_eq!(middle.len(), 1);
    assert_eq!(middle[0].span(0), (-2.0, 2.0));
    assert_eq!(middle[0].crossings(0.0), vec![3.0, 5.0]);
}

#[test]
fn a_ring_is_the_trunk_without_the_legs_beside_it() {
    let mesh = beast();
    // Across the hind legs: three outlines, and the ring is the trunk's.
    let found = ring(&mesh, &Plane::facing([0.0, 0.0, 0.5], [0.0, 0.0, 1.0])).unwrap();
    assert_eq!((found.half_width, found.belly, found.back), (2.0, 3.0, 5.0));
    // Behind the body there is nothing.
    assert!(ring(&mesh, &Plane::facing([0.0, 0.0, -3.0], [0.0, 0.0, 1.0])).is_none());
}

#[test]
fn legs_are_told_apart_and_measured() {
    let mesh = beast();
    let (fore, hind) = legs(&mesh, 1.0, 4.0);
    let (fore, hind) = (fore.unwrap(), hind.unwrap());
    assert_eq!(fore.half, (0.5, 0.5));
    assert_eq!(fore.middle, (1.5, 7.5));
    assert_eq!(hind.middle, (1.5, 0.5));
    // Level with the trunk no leg stands clear.
    assert_eq!(legs(&mesh, 4.0, 4.0), (None, None));
}

#[test]
fn a_four_legged_beast_is_measured_from_its_joints() {
    let beast = Fourfoot::measure(beast(), 5).unwrap();
    assert_eq!(beast.withers, 5.0);
    assert_eq!(beast.hips, 4.0);
    assert_eq!(beast.shoulders, 7.0);
    assert_eq!(beast.trunk.len(), 5);
    // From the rump (1.5 behind the hip joints) to the neck's base.
    assert!((beast.trunk[0].0 + 1.5).abs() < 0.01);
    for (_, ring) in &beast.trunk[1..] {
        assert_eq!((ring.half_width, ring.belly, ring.back), (2.0, 3.0, 5.0));
    }
    // Legs every twentieth of the withers' height, below the trunk.
    let (height, fore, hind) = beast.legs[0];
    assert_eq!(height, 0.25);
    assert_eq!(fore.unwrap().middle, (1.5, 7.0));
    assert_eq!(hind.unwrap().middle, (1.5, 0.0));
}

#[test]
fn a_beast_standing_on_a_slope_is_measured_level() {
    let mut tilted = beast();
    for point in &mut tilted.points {
        point[1] += 0.1 * point[2];
    }
    let level = Fourfoot::measure(beast(), 5).unwrap();
    let tilted = Fourfoot::measure(tilted, 5).unwrap();
    assert!((tilted.withers - level.withers).abs() < 1e-9);
    assert!((tilted.hips - level.hips).abs() < 1e-9);
    for (a, b) in tilted.trunk.iter().zip(&level.trunk).skip(1) {
        assert!((a.1.belly - b.1.belly).abs() < 1e-9);
        assert!((a.1.back - b.1.back).abs() < 1e-9);
    }
}

/// A person-shaped stand with a head on it: a trunk, a neck, a skull 2 across
/// and long, and a snout reaching 2 in front of it with a nose at the end, level
/// with the `head` joint (the `neck` joint is where the neck starts).
fn headed() -> Mesh {
    mesh(
        &[
            ([-2.0, 0.0, -1.0], [2.0, 8.0, 1.0]),
            ([-0.5, 8.0, -0.5], [0.5, 10.0, 0.5]),
            ([-1.0, 9.5, -1.0], [1.0, 11.5, 1.0]),
            ([-0.5, 9.5, 1.0], [0.5, 10.5, 3.0]),
            ([-0.1, 9.95, 3.0], [0.1, 10.05, 3.1]),
        ],
        &[("neck", [0.0, 8.0, 0.0]), ("head", [0.0, 10.0, 0.0])],
    )
}

#[test]
fn a_head_is_measured_from_the_back_of_the_skull_to_the_nose() {
    let head = Head::measure(&headed(), 8).unwrap();
    // The nose is a corner of a small box, so the head's line leans a little.
    assert!(head.pitch.abs() < 2.0, "{}", head.pitch);
    assert!((head.length - 4.1).abs() < 0.1, "{}", head.length);
    assert_eq!(head.rings.len(), 8);
    let (first, _) = head.rings[0];
    assert!((first + 1.0).abs() < 0.1, "the back of the skull: {first}");
    // The first ring is the back of the skull's tip.
    for &(along, ring) in &head.rings[1..] {
        // The skull is 2 across, the snout 1; nothing of the trunk below.
        // (The last is in the nose, in front of the snout.)
        if along >= 3.0 {
            continue;
        }
        let wide = if along < 0.95 { 1.0 } else { 0.5 };
        assert!((ring.half_width - wide).abs() < 0.1, "{along}: {ring:?}");
        // Down to the jaw's underside, half a unit below the head's line.
        assert!((ring.belly + 0.5).abs() < 0.1, "{along}: {ring:?}");
    }
}

/// A person of boxes standing upright: a trunk 4 wide over two legs, an arm
/// hanging beside it, and the joints a person is measured by. The hip
/// joints are 1.5 behind the trunk's front and 1 in front of its back.
fn person() -> Mesh {
    mesh(
        &[
            ([-2.0, 5.0, -1.0], [2.0, 10.0, 1.5]),
            ([0.5, 0.0, -0.5], [1.5, 5.5, 0.5]),
            ([-1.5, 0.0, -0.5], [-0.5, 5.5, 0.5]),
            ([2.5, 6.0, -0.4], [3.1, 9.5, 0.4]),
        ],
        &[
            ("l-upper-leg", [1.0, 5.5, 0.0]),
            ("l-knee", [1.0, 3.0, 0.0]),
            ("l-ankle", [1.0, 0.5, 0.0]),
            ("l-shoulder", [2.8, 9.5, 0.0]),
            ("l-elbow", [2.8, 8.0, 0.0]),
            ("l-hand", [2.8, 6.5, 0.0]),
            ("neck", [0.0, 10.0, 0.0]),
        ],
    )
}

#[test]
fn an_upright_body_is_measured_up_its_trunk_and_along_its_limbs() {
    let body = Upright::measure(&person(), 5).unwrap();
    assert!((body.height - 10.0).abs() < 0.01);
    assert!((body.crotch - 5.0).abs() < 0.01, "{}", body.crotch);
    for (got, want) in [(body.hips, 5.5), (body.shoulders, 9.5), (body.neck, 10.0)] {
        assert!((got - want).abs() < 0.01, "{got} {want}");
    }
    // The top cut may or may not catch the box's very top edge.
    assert!(body.trunk.len() >= 4, "{}", body.trunk.len());
    for &(_, ring) in &body.trunk {
        assert!((ring.half_width - 2.0).abs() < 0.01, "{ring:?}");
        // Front and back from the line up through the hip joints.
        assert!((ring.belly - 1.5).abs() < 0.01, "{ring:?}");
        assert!((ring.back - 1.0).abs() < 0.01, "{ring:?}");
    }
    let thigh = &body.limbs[0];
    assert_eq!(thigh.name, "thigh");
    assert!((thigh.length - 2.5).abs() < 0.01, "{}", thigh.length);
    for cut in &thigh.cuts {
        let (across, deep) = cut.expect("the leg is cut");
        assert!((across - 0.5).abs() < 0.01 && (deep - 0.5).abs() < 0.01);
    }
    let arm = &body.limbs[2];
    let (across, deep) = arm.cuts[2].expect("the arm is cut");
    assert!((across - 0.3).abs() < 0.01 && (deep - 0.4).abs() < 0.01);
}
