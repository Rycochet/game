// The world map's terrain: the standard lit material, darkened under clouds,
// plus moving water. See docs/context/client/world_map.md.

import bevy_pbr::render::{
    forward_io::{FragmentOutput, VertexOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
};

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

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> sky: Sky;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var sky_a: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var sky_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var sky_b: texture_2d<f32>;

// The sky here: two moments of it, faded between so it changes smoothly.
fn sky_at(place: vec2<f32>) -> vec4<f32> {
    return mix(
        textureSample(sky_a, sky_sampler, (place - sky.origin_a) / sky.extent),
        textureSample(sky_b, sky_sampler, (place - sky.origin_b) / sky.extent),
        sky.blend,
    );
}

// How much a full cloud darkens the ground straight below it.
const SHADOW: f32 = 0.2;

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
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color,
    );

    let place = in.world_position.xyz;
    let above = sky_at(place.xz);

    var colour = pbr_input.material.base_color.rgb;

    // Vertex alpha below 1 marks water.
    var water = 0.0;
    @if(VERTEX_COLORS) {
        water = 1.0 - in.color.a;
    }
    if water > 0.001 {
        // Two layers of slow ripples, rougher and greyer under a storm.
        let rough = above.g;
        let speed = 0.25 + 0.75 * above.r + rough;
        let wrapped = place.xz % 4096.0;
        let small = smooth_noise(TURN * wrapped / 6.0 + vec2<f32>(sky.time * 0.11, sky.time * 0.07) * speed);
        let large = smooth_noise(TURN * TURN * wrapped / 19.0 - vec2<f32>(sky.time * 0.05, sky.time * -0.03) * speed);
        let ripple = small * 0.55 + large * 0.45;
        let strength = 0.16 + 0.4 * rough;
        var moving = colour * (1.0 + (ripple - 0.5) * 2.0 * strength);
        moving += vec3<f32>(smoothstep(0.68, 0.8, ripple) * (0.04 + 0.1 * rough));
        let grey = vec3<f32>(dot(moving, vec3<f32>(0.3, 0.6, 0.1)));
        moving = mix(moving, grey, rough * 0.4);
        colour = mix(colour, moving, water);
    }

    colour *= 1.0 - SHADOW * above.r;
    pbr_input.material.base_color = vec4<f32>(colour, pbr_input.material.base_color.a);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
