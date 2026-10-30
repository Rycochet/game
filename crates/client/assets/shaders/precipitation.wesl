// Falling rain, sleet, snow and hail. The mesh is thousands of small quads whose
// positions hold random numbers; this places each one, using the sky texture
// to decide if it falls at all and as what. Nothing here affects the game.

import bevy_pbr::render::mesh_view_bindings::view;

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
        textureSampleLevel(sky_a, sky_sampler, (place - sky.origin_a) / sky.extent, 0.0),
        textureSampleLevel(sky_b, sky_sampler, (place - sky.origin_b) / sky.extent, 0.0),
        sky.blend,
    );
}

struct Vertex {
    @location(0) seed: vec3<f32>,
    @location(1) corner: vec2<f32>,
}

struct Output {
    @builtin(position) clip: vec4<f32>,
    @location(0) colour: vec4<f32>,
    @location(1) corner: vec2<f32>,
    @location(2) is_round: f32,
}

// Temperatures (0..1) below which it falls as snow, then sleet.
const SNOW_BELOW: f32 = 0.2;
const SLEET_BELOW: f32 = 0.3;
// How hard it must be raining for hail.
const HAIL_ABOVE: f32 = 0.8;
// Grid units the drops' pattern repeats over: more than the window's diagonal.
const DROP_TILE: f32 = 600.0;
// The lowest anything falls: below every shore.
const LOWEST: f32 = 15.0;

@vertex
fn vertex(in: Vertex) -> Output {
    var out: Output;
    out.corner = in.corner;
    out.is_round = 0.0;
    // Not drawn, unless it turns out below that it falls.
    out.clip = vec4<f32>(0.0, 0.0, 2.0, 1.0);
    out.colour = vec4<f32>(0.0);

    // Each drop has a fixed place on the map, in a pattern repeating every
    // `DROP_TILE`, taken at the repeat nearest the player, so walking doesn't
    // move it. Only the ones over the window are drawn.
    let spot = in.seed.xz * DROP_TILE;
    let wrapped = fract((spot - sky.centre) / DROP_TILE + 0.5) - 0.5;
    let relative = wrapped * DROP_TILE;
    if abs(dot(relative, sky.right)) > sky.half || abs(dot(relative, sky.towards)) > sky.half {
        return out;
    }
    let ground = sky.centre + relative;
    let cell = sky_at(ground);
    let rain = cell.g;
    let temperature = cell.b;

    // Harder rain lights up more of the quads.
    if rain * 1.05 < fract(in.seed.y * 57.3) + 0.03 {
        return out;
    }

    var speed = 70.0;
    var size = vec2<f32>(0.3, 5.0);
    var colour = vec4<f32>(0.65, 0.75, 0.95, 0.55);
    if temperature < SNOW_BELOW {
        speed = 11.0;
        size = vec2<f32>(0.9, 0.9);
        colour = vec4<f32>(1.0, 1.0, 1.0, 0.9);
        out.is_round = 1.0;
    } else if temperature < SLEET_BELOW {
        speed = 38.0;
        size = vec2<f32>(0.5, 2.2);
        colour = vec4<f32>(0.85, 0.9, 1.0, 0.75);
    } else if rain > HAIL_ABOVE {
        speed = 90.0;
        size = vec2<f32>(0.8, 0.8);
        colour = vec4<f32>(0.95, 0.97, 1.0, 1.0);
        out.is_round = 1.0;
    }

    let fall = sky.cloud_height - LOWEST;
    let height = LOWEST + fall * fract(in.seed.y - sky.time * speed / fall);
    let view_right = view.world_from_view[0].xyz;
    let place = vec3<f32>(ground.x, height, ground.y)
        + view_right * in.corner.x * size.x
        + vec3<f32>(0.0, 1.0, 0.0) * in.corner.y * size.y;
    out.clip = view.clip_from_world * vec4<f32>(place, 1.0);
    out.colour = colour;
    return out;
}

@fragment
fn fragment(in: Output) -> @location(0) vec4<f32> {
    if in.is_round > 0.5 && dot(in.corner, in.corner) > 0.25 {
        discard;
    }
    return in.colour;
}
