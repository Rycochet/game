use bevy::prelude::*;
use shared::hall::{GateDesign, GatePlace};

use super::GateModel;
use super::profile::{HALF_OPENING, SPRING, height, jamb};

#[test]
fn every_design_stands_inside_its_outline() {
    for design in GateDesign::ALL {
        let (half, top) = GateModel::size(design);
        let frame = GateModel::frame(design);
        assert!(frame.triangles().count() > 10, "{design:?}");
        for (corners, _) in frame.triangles() {
            for p in corners {
                assert!(p.is_finite(), "{design:?}");
                assert!(
                    p.x.abs() <= half + 1e-4 && (0.0..=top + 1e-4).contains(&p.y),
                    "{design:?} {p}"
                );
            }
        }
        assert!(jamb(design) <= HALF_OPENING && height(design) > SPRING);
    }
}

#[test]
fn faces_turn_the_way_they_say() {
    for design in GateDesign::ALL {
        let model = GateModel::frame(design);
        for (i, (corners, _)) in model.triangles().enumerate() {
            let [a, b, c] = corners;
            let normal = model.normals[i * 3];
            let face = (b - a).cross(c - a);
            if face.length() > 1e-7 {
                assert!(face.dot(normal) > 0.0, "{design:?}");
            }
        }
    }
}

#[test]
fn the_front_is_filled_once() {
    // The front faces cover the outline less the opening: no gaps or overlaps.
    for design in GateDesign::ALL {
        let (half, top) = GateModel::size(design);
        let front: f32 = GateModel::frame(design)
            .triangles()
            .filter(|([a, b, c], _)| a.z > 0.0 && b.z > 0.0 && c.z > 0.0)
            .filter(|([a, b, c], _)| (b - a).cross(c - a).normalize_or_zero().z > 0.99)
            .map(|([a, b, c], _)| (b - a).cross(c - a).length() / 2.0)
            .sum();
        let opening = opening_area(design);
        let expected = 2.0 * half * top - opening;
        assert!(
            (front - expected).abs() < 0.02,
            "{design:?}: {front} against {expected}"
        );
    }
}

/// The opening's area, by the shoelace formula over its outline.
fn opening_area(design: GateDesign) -> f32 {
    let jamb = jamb(design);
    let mut ring = vec![Vec2::new(-jamb, 0.0)];
    ring.extend(
        super::profile::top(design)
            .into_iter()
            .map(|p| p + Vec2::new(0.0, SPRING)),
    );
    ring.push(Vec2::new(jamb, 0.0));
    let twice: f32 = ring
        .iter()
        .zip(ring.iter().cycle().skip(1))
        .map(|(a, b)| a.perp_dot(*b))
        .sum();
    twice.abs() / 2.0
}

#[test]
fn runes_fit_across_the_frame() {
    for design in GateDesign::ALL {
        let (half, top) = GateModel::size(design);
        let place = GatePlace::from_number(175_562_835);
        let runes = GateModel::runes(design, place.code());
        assert!(runes.triangles().count() > 0);
        for (corners, _) in runes.triangles() {
            for p in corners {
                assert!(
                    p.x.abs() < half && p.y > height(design) && p.y < top,
                    "{design:?} {p}"
                );
            }
        }
    }
}
