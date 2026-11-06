//! Sine and cosine by whole degrees, in ten-thousandths, from a table: floats' `sin`
//! may differ between machines, and both sides must agree on where a tilted body's
//! points are.

/// Sine of 0 to 90 degrees, in ten-thousandths.
const SINE: [i32; 91] = [
    0, 175, 349, 523, 698, 872, 1045, 1219, 1392, 1564, 1736, 1908, 2079, 2250, 2419, 2588, 2756,
    2924, 3090, 3256, 3420, 3584, 3746, 3907, 4067, 4226, 4384, 4540, 4695, 4848, 5000, 5150, 5299,
    5446, 5592, 5736, 5878, 6018, 6157, 6293, 6428, 6561, 6691, 6820, 6947, 7071, 7193, 7314, 7431,
    7547, 7660, 7771, 7880, 7986, 8090, 8192, 8290, 8387, 8480, 8572, 8660, 8746, 8829, 8910, 8988,
    9063, 9135, 9205, 9272, 9336, 9397, 9455, 9511, 9563, 9613, 9659, 9703, 9744, 9781, 9816, 9848,
    9877, 9903, 9925, 9945, 9962, 9976, 9986, 9994, 9998, 10000,
];

/// What sines and cosines are out of: 10,000 is 1.
pub const ONE: i32 = 10_000;

/// The sine of an angle in whole degrees, any size, in ten-thousandths.
pub fn sin(degrees: i32) -> i32 {
    let turn = degrees.rem_euclid(360);
    let index = |d: i32| SINE[d as usize];
    match turn {
        0..=90 => index(turn),
        91..=180 => index(180 - turn),
        181..=270 => -index(turn - 180),
        _ => -index(360 - turn),
    }
}

/// The cosine of an angle in whole degrees, in ten-thousandths.
pub fn cos(degrees: i32) -> i32 {
    sin(degrees + 90)
}
