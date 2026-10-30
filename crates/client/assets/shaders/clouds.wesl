// The world map's cloud layer: a see-through sheet straight above the terrain
// window, ending in the same hard edge (the mesh is cut to it), its shape from the low-resolution sky texture with some fine detail added.

import bevy_pbr::render::forward_io::VertexOutput;

struct Sky {
    origin_a: vec2<f32>,
    origin_b: vec2<f32>,
    extent: f32,
    blend: f32,
    time: f32,
    cloud_height: f32,
    half: f32,
    right: vec2<f32>,
    towards: vec2<f32>,
    centre: vec2<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sky: Sky;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var sky_a: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var sky_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var sky_b: texture_2d<f32>;

// The sky here: two moments of it, faded between so it changes smoothly.
fn sky_at(place: vec2<f32>) -> vec4<f32> {
    return mix(
        textureSample(sky_a, sky_sampler, (place - sky.origin_a) / sky.extent),
        textureSample(sky_b, sky_sampler, (place - sky.origin_b) / sky.extent),
        sky.blend,
    );
}

// The thickest cloud still lets this much of the map show.
const MOST_OPAQUE: f32 = 0.3;

// Grid units across each block of cloud, for a chunky, low-resolution look:
// about 8 screen pixels in the starting view.
const BLOCK: f32 = 2.5;

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

// Smooth gradient noise, about 0..1: random slopes at whole coordinates, which
// shows the grid much less than random values there would.
fn slope_at(cell: vec2<f32>, offset: vec2<f32>) -> f32 {
    let angle = hash(cell) * 6.2831853;
    return dot(vec2<f32>(cos(angle), sin(angle)), offset);
}

fn smooth_noise(p: vec2<f32>) -> f32 {
    let cell = floor(p);
    let f = fract(p);
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let value = mix(
        mix(slope_at(cell, f), slope_at(cell + vec2<f32>(1.0, 0.0), f - vec2<f32>(1.0, 0.0)), u.x),
        mix(slope_at(cell + vec2<f32>(0.0, 1.0), f - vec2<f32>(0.0, 1.0)), slope_at(cell + vec2<f32>(1.0, 1.0), f - vec2<f32>(1.0, 1.0)), u.x),
        u.y,
    );
    return clamp(0.5 + value * 0.9, 0.0, 1.0);
}

// A turn of the coordinates between layers, so grid lines don't show.
const TURN: mat2x2<f32> = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // The whole block takes the look of its middle.
    let place = (floor(in.world_position.xz / BLOCK) + 0.5) * BLOCK;
    let cell = sky_at(place);

    let wrapped = place % 4096.0;
    let detail = smooth_noise(TURN * wrapped / 26.0) * 0.6 + smooth_noise(TURN * TURN * wrapped / 11.0) * 0.4;
    let thickness = smoothstep(0.2, 0.85, cell.r + (detail - 0.5) * 0.4);

    let colour = mix(vec3<f32>(0.9, 0.92, 0.95), vec3<f32>(0.3, 0.32, 0.36), cell.g);
    return vec4<f32>(colour, thickness * MOST_OPAQUE);
}
