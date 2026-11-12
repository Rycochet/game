//! Standing a four-legged mesh on level ground. MakeHuman's animals stand
//! with their forefeet a little higher than their hind feet (about a
//! twentieth of their height), which would tilt every measurement.

use super::mesh::Mesh;

/// Slides every point up or down by how far along the body it is, so the
/// lowest point in front of `halfway` (a forefoot) comes level with the
/// lowest behind it (a hind foot). Over so slight a tilt this is as good as
/// turning the body, and keeps every cut across it upright.
pub fn stand_level(mesh: &mut Mesh, halfway: f64) {
    // The lowest of the skin's points on one side of halfway: (height, along).
    let lowest = |front: bool| {
        let points = mesh.skin.iter().flatten().map(|&at| mesh.points[at]);
        points
            .filter(|point| (point[2] > halfway) == front)
            .map(|point| (point[1], point[2]))
            .min_by(|a, b| a.0.total_cmp(&b.0))
    };
    let (Some(fore), Some(hind)) = (lowest(true), lowest(false)) else {
        return;
    };
    let slope = (fore.0 - hind.0) / (fore.1 - hind.1);
    for point in &mut mesh.points {
        point[1] -= slope * (point[2] - hind.1);
    }
}
