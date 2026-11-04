// View Planet's clouds: a see-through shell just above the globe, for looks
// only. Soft blobs from noise on the sphere, which change over time two ways:
// the noise is slowly moved through with the hours, and bands of latitude
// drift east or west at their own speeds (trade winds, westerlies, polar
// easterlies), shearing the blobs into streaks.

import bevy_pbr::render::forward_io::VertexOutput;

struct Clouds {
    // xyz: where in the noise this world's clouds start; w: game hours.
    start_hours: vec4<f32>,
    // xyz: towards the main sun.
    sun: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> clouds: Clouds;

const TAU: f32 = 6.2831853;
const PI: f32 = 3.1415927;

// How fine the blobs are: a few dozen across the globe.
const SCALE: f32 = 3.5;
// How fast the noise is moved through, in noise units a game hour.
const CHANGE: f32 = 0.025;
// How fast the bands drift, in radians of longitude a game hour.
const DRIFT: f32 = 0.03;
// The thickest cloud, and the dull light on the night side.
const MOST_OPAQUE: f32 = 0.85;
const NIGHT: f32 = 0.06;

// A random value 0..1 for a whole-numbered corner. Integer maths, so a corner
// gets exactly the same value in every pixel: a `sin` hash isn't exact on
// graphics cards, and broke the clouds along the mesh's triangles.
fn hash(corner: vec3<i32>) -> f32 {
    var v = bitcast<vec3<u32>>(corner) * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v = v ^ (v >> vec3<u32>(16u));
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    return f32(v.x & 0xffffffu) / 16777216.0;
}

// Smooth value noise, 0..1.
fn noise(p: vec3<f32>) -> f32 {
    let cell = vec3<i32>(floor(p));
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let x00 = mix(hash(cell), hash(cell + vec3<i32>(1, 0, 0)), u.x);
    let x10 = mix(hash(cell + vec3<i32>(0, 1, 0)), hash(cell + vec3<i32>(1, 1, 0)), u.x);
    let x01 = mix(hash(cell + vec3<i32>(0, 0, 1)), hash(cell + vec3<i32>(1, 0, 1)), u.x);
    let x11 = mix(hash(cell + vec3<i32>(0, 1, 1)), hash(cell + vec3<i32>(1, 1, 1)), u.x);
    return mix(mix(x00, x10, u.y), mix(x01, x11, u.y), u.z);
}

// Four layers, each twice as fine and half as strong.
fn layered(p: vec3<f32>) -> f32 {
    var total = 0.0;
    var strength = 0.5;
    var at = p;
    for (var layer = 0; layer < 4; layer++) {
        total += noise(at) * strength;
        at = at * 2.03 + vec3<f32>(17.0, 31.0, 7.0);
        strength *= 0.5;
    }
    return total / 0.9375;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let hours = clouds.start_hours.w;
    let latitude = PI * (0.5 - in.uv.y);
    // West in the tropics and near the poles, east between: -cos(4 × latitude).
    let longitude = in.uv.x * TAU - cos(4.0 * latitude) * DRIFT * hours;
    let point = vec3<f32>(
        cos(latitude) * cos(longitude),
        sin(latitude),
        -cos(latitude) * sin(longitude),
    );
    let drifting = clouds.start_hours.xyz + vec3<f32>(0.3, 0.7, 1.0) * CHANGE * hours;
    let shape = layered(point * SCALE + drifting);

    // More cloud at the equator and the middle latitudes, less in the dry
    // belts about 30 degrees out and at the poles.
    let wet = cos(6.0 * latitude);
    let threshold = 0.55 - 0.08 * wet;
    let thickness = smoothstep(threshold, threshold + 0.15, shape);

    let lit = max(dot(normalize(in.world_normal), normalize(clouds.sun.xyz)), 0.0);
    let light = NIGHT + (1.0 - NIGHT) * lit;
    return vec4<f32>(vec3<f32>(light), thickness * MOST_OPAQUE * mix(0.4, 1.0, sqrt(lit)));
}
