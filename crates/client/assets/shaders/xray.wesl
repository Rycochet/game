// What is hidden behind something else, drawn through it in one colour (see
// docs/context/client/view3d.md, "X-ray"). The pipeline (view3d/token/xray.rs)
// only lets through what is further away than what is already drawn, so this
// shader just fills: the colour, cut out by a picture's alpha if it has one.

import bevy_pbr::render::forward_io::VertexOutput;

// One colour, 16 bytes as WebGL 2 needs.
struct Xray {
    colour: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> xray: Xray;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var picture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var picture_sampler: sampler;

// How much of a picture's pixel must be there to count, as the pictures' own
// alpha cut-out.
const CUT: f32 = 0.5;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Without a picture this samples Bevy's white stand-in, which is all there.
    var alpha = 1.0;
    @if(VERTEX_UVS_A) {
        alpha = textureSample(picture, picture_sampler, in.uv).a;
    }
    if alpha < CUT {
        discard;
    }
    return xray.colour;
}
