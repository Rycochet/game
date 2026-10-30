// The world map's terrain: the standard lit material, darkened under clouds,
// plus moving water and vegetation. See docs/context/client/world_map.md.

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

// How far apart tree crowns are, in grid units.
const CROWN: f32 = 2.6;
// How strongly crowns and grass tilt the light.
const CROWN_BUMP: f32 = 1.1;
const GRASS_BUMP: f32 = 0.3;
// Grass tufts per grid unit, a little longer one way than the other.
const GRASS_SCALE: vec2<f32> = vec2<f32>(0.9, 0.6);

struct Canopy {
    // 0 in the gaps between crowns, 1 at the top of one.
    height: f32,
    // Which way, and how fast, it rises.
    slope: vec2<f32>,
    // A random number for the crown on top here.
    tint: f32,
}

// The tops of the trees: rounded crowns on a jittered grid, conical where
// `narrow`. Thin woods lose crowns one at a time, so their edges are ragged.
fn canopy_at(place: vec2<f32>, trees: f32, narrow: f32) -> Canopy {
    let at = place / CROWN;
    let cell = floor(at);
    var best = Canopy(0.0, vec2<f32>(0.0), 0.5);
    for (var dz = -1.0; dz <= 1.0; dz += 1.0) {
        for (var dx = -1.0; dx <= 1.0; dx += 1.0) {
            let other = cell + vec2<f32>(dx, dz);
            let chance = hash(other + vec2<f32>(7.0, 3.0));
            let present = smoothstep(chance * 0.85, chance * 0.85 + 0.12, trees);
            if present <= 0.0 {
                continue;
            }
            let centre = other + 0.15 + 0.7 * vec2<f32>(hash(other), hash(other + vec2<f32>(19.0, 41.0)));
            let radius = mix(0.75, 1.05, hash(other + vec2<f32>(53.0, 11.0))) * mix(1.0, 0.75, narrow);
            let offset = (at - centre) / radius;
            let distance = length(offset);
            if distance >= 1.0 {
                continue;
            }
            let height = present * mix(1.0 - distance * distance, 1.0 - distance, narrow);
            if height > best.height {
                // Per grid unit rather than per crown.
                let rise = mix(2.0 * offset, offset / max(distance, 0.05), narrow) * present / (radius * CROWN);
                best = Canopy(height, -rise, chance);
            }
        }
    }
    return best;
}

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
    // How many grid units one pixel covers (taken here, as `fwidth` can't be
    // used inside a branch that differs from pixel to pixel).
    let pixel = length(fwidth(place.xz));

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

    // Vegetation: x trees, y low growth, then how narrow and how lush the trees.
    var cover = vec4<f32>(0.0);
    @if(VERTEX_UVS_A && VERTEX_UVS_B) {
        cover = vec4<f32>(in.uv, in.uv_b);
    }
    if water <= 0.001 && cover.x + cover.y > 0.001 {
        let wrapped = place.xz % 4096.0;
        var bump = vec2<f32>(0.0);

        // Fine detail fades out before it gets near pixel size, or it shimmers.
        let grass_detail = 1.0 - smoothstep(0.5, 1.5, pixel);
        let grass = smooth_noise(TURN * wrapped * GRASS_SCALE);
        let tufts = smooth_noise(TURN * TURN * wrapped / 5.0);
        colour *= 1.0 + cover.y * ((grass - 0.5) * 0.3 * grass_detail + (tufts - 0.5) * 0.3);
        if grass_detail > 0.0 {
            let east = smooth_noise(TURN * (wrapped + vec2<f32>(0.5, 0.0)) * GRASS_SCALE);
            let south = smooth_noise(TURN * (wrapped + vec2<f32>(0.0, 0.5)) * GRASS_SCALE);
            bump += vec2<f32>(east - grass, south - grass) * 2.0 * GRASS_BUMP * cover.y * grass_detail;
        }

        if cover.x > 0.001 {
            let crown_detail = 1.0 - smoothstep(0.5, 1.2, pixel / CROWN);
            let canopy = canopy_at(wrapped, cover.x, cover.z);
            // Rainforest canopy rises and falls in big uneven lumps.
            let lumps = smooth_noise(TURN * wrapped / 11.0);
            // Each crown a slightly different shade, a little redder or greener.
            let tint = vec3<f32>(1.0 + (canopy.tint - 0.5) * 0.3, 1.0 + (canopy.tint - 0.5) * 0.12, 1.0);
            // Where no crown stands the ground shows through, in shade.
            let lit = colour * mix(0.62, 1.05, canopy.height) * tint;
            let lumpy = lit * mix(1.0, 0.7 + 0.6 * lumps, cover.w);
            // From far away, only the average: darker than open ground.
            let wooded = mix(colour * 0.8, lumpy, crown_detail);
            let amount = smoothstep(0.0, 0.25, cover.x);
            colour = mix(colour, wooded, amount);
            bump = mix(bump, canopy.slope * CROWN_BUMP * crown_detail, amount);
        }

        pbr_input.N = normalize(pbr_input.N - vec3<f32>(bump.x, 0.0, bump.y));
    }

    colour *= 1.0 - SHADOW * above.r;
    pbr_input.material.base_color = vec4<f32>(colour, pbr_input.material.base_color.a);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
