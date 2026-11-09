// The world map's terrain in the prepass, which draws shadows: only what the
// window shows casts them, as terrain.wesl cuts the window out of the chunks.
// See docs/context/client/world_map.md.

import bevy_pbr::prepass::io::VertexOutput;

// As in terrain.wesl and `SkyParams`.
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
    light: vec4<f32>,
    shown: vec4<f32>,
    still: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> sky: Sky;

fn outside_window(place: vec2<f32>) -> bool {
    let from_centre = place - sky.centre;
    return abs(dot(from_centre, sky.right)) > sky.half
        || abs(dot(from_centre, sky.towards)) > sky.half;
}

// Shadows only need depth, so there is nothing to write unless Bevy emulates
// unclipped depth for the directional light, as its own prepass shader does.
// (Bevy's output struct would have no fields here, which WESL leaves out.)
@if(UNCLIPPED_DEPTH_ORTHO_EMULATION)
@fragment
fn fragment(in: VertexOutput) -> @builtin(frag_depth) f32 {
    if outside_window(in.world_position.xz) {
        discard;
    }
    return in.unclipped_depth;
}

@if(!UNCLIPPED_DEPTH_ORTHO_EMULATION)
@fragment
fn fragment(in: VertexOutput) {
    if outside_window(in.world_position.xz) {
        discard;
    }
}
