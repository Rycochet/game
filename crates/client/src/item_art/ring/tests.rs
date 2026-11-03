use super::*;
use crate::item_art::VIEW_WIDTH;

fn ring(profile: Profile, width: BandWidth, pattern: Pattern) -> RingLook {
    RingLook {
        metal: Material::Gold,
        profile,
        width,
        pattern,
        settings: Vec::new(),
    }
}

const PROFILES: [Profile; 5] = [
    Profile::Domed,
    Profile::Flat,
    Profile::Knife,
    Profile::Round,
    Profile::Square,
];
const PATTERNS: [Pattern; 4] = [
    Pattern::Engraved,
    Pattern::Hammered,
    Pattern::Plain,
    Pattern::Twisted,
];

#[test]
fn every_band_stays_in_the_picture_and_has_faces() {
    for profile in PROFILES {
        for pattern in PATTERNS {
            let mesh = band_mesh(&ring(profile, BandWidth::Wide, pattern));
            let points = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
            let points = points.as_float3().unwrap();
            assert!(points.len() >= BAND_STEPS * 3);
            for &point in points {
                let reach = Vec3::from(point).length();
                assert!(reach < VIEW_WIDTH / 2.0, "{profile:?} {pattern:?} {reach}");
            }
        }
    }
}

#[test]
fn a_face_of_the_band_looks_the_way_its_normal_says() {
    let mesh = band_mesh(&ring(Profile::Square, BandWidth::Medium, Pattern::Plain));
    let points = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
    let points = points.as_float3().unwrap();
    let normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap();
    let normals = normals.as_float3().unwrap();
    for (corners, normals) in points.chunks(3).zip(normals.chunks(3)) {
        let (a, b, c) = (
            Vec3::from(corners[0]),
            Vec3::from(corners[1]),
            Vec3::from(corners[2]),
        );
        assert!((b - a).cross(c - a).dot(Vec3::from(normals[0])) > 0.0);
    }
}

#[test]
fn a_rod_goes_between_its_two_points() {
    let (from, to) = (Vec3::new(0.1, 0.0, 0.2), Vec3::new(0.4, 0.5, 0.0));
    let placed = rod(from, to, 0.02);
    let top = placed.transform_point(Vec3::Y * 0.5);
    let bottom = placed.transform_point(Vec3::NEG_Y * 0.5);
    assert!(top.distance(to) < 1e-4 && bottom.distance(from) < 1e-4);
}

#[test]
fn larger_sizes_draw_larger() {
    assert!(size_scale(1) > size_scale(0) && size_scale(0) > size_scale(-1));
}
