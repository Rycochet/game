//! An uncut stone's rough mesh.

use bevy::prelude::*;
use shared::rng::Seed;

use super::Facets;

/// A sphere pushed in and out a different way for each seed. Its faces shade only a
/// little: the lumps do the work.
pub(super) fn uncut_mesh(shape_seed: u16) -> Mesh {
    let seed = Seed::new(shape_seed as u64).derive("uncut gem");
    let sphere = Sphere::new(1.0)
        .mesh()
        .ico(2)
        .expect("2 is few enough subdivisions");
    let points = sphere
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|positions| positions.as_float3())
        .expect("a sphere has positions");
    let number = |purpose: i64, from: f32, to: f32| {
        let bits = seed.at(&[purpose]) >> 40;
        from + (to - from) * bits as f32 / (1u64 << 24) as f32
    };
    // Squashed along each axis a different amount, then lumps at each point.
    let squash = Vec3::new(
        number(-1, 0.8, 1.0),
        number(-2, 0.65, 0.9),
        number(-3, 0.8, 1.0),
    );
    let points: Vec<Vec3> = points
        .iter()
        .enumerate()
        .map(|(i, &point)| Vec3::from(point) * number(i as i64, 0.8, 1.08) * squash)
        .collect();
    let mut facets = Facets::new(shape_seed as u64 + 1, 0.05);
    let corners: Vec<usize> = sphere
        .indices()
        .map_or(Vec::new(), |indices| indices.iter().collect());
    for &[a, b, c] in corners.as_chunks::<3>().0 {
        facets.triangle(points[a], points[b], points[c]);
    }
    facets.into_mesh()
}
