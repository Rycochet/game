//! Small sums on 3D vectors.

pub(super) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(super) fn unit(v: [f64; 3]) -> [f64; 3] {
    scaled(v, 1.0 / dot(v, v).sqrt())
}

pub(super) fn scaled(v: [f64; 3], by: f64) -> [f64; 3] {
    v.map(|channel| channel * by)
}
