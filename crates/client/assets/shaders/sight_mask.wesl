// A dungeon surface: the standard material, shown only on the side of the sight
// mask it belongs to, so the edge of sight falls part way across a cell. See
// docs/context/client/dungeon_map.md.

import bevy_pbr::render::{
    forward_io::{FragmentOutput, VertexOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, main_pass_post_lighting_processing},
};

struct Mask {
    origin: vec2<f32>,
    per_unit: vec2<f32>,
    // 1 only in sight, -1 only out of sight.
    side: f32,
    // Pads the uniform to 32 bytes: WebGL 2 needs a multiple of 16.
    _pad_a: f32,
    _pad_b: f32,
    _pad_c: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> mask: Mask;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var mask_picture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var mask_sampler: sampler;

// How far in front of an upright surface the mask is read, in world units (a
// tenth of a cell): a wall's side stands right on the edge of what is in sight.
const IN_FRONT: f32 = 2.4;

// Tops, facing up over the floor: sight stops at the wall below them, so they go
// by the cell, as they always did. Match `RAISED` in mesh.rs.
const RAISED: f32 = 1.0;
const RAISED_NORMAL: f32 = 0.5;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    // Sampled before anything can return, as derivatives need every fragment.
    let normal = normalize(in.world_normal);
    let at = in.world_position.xz + normal.xz * IN_FRONT;
    let place = (at - mask.origin) * mask.per_unit;
    let in_sight = textureSample(mask_picture, mask_sampler, place).r > 0.5;
    let raised = in.world_position.y > RAISED && normal.y > RAISED_NORMAL;
    if !raised && ((mask.side > 0.0 && !in_sight) || (mask.side < 0.0 && in_sight)) {
        discard;
    }

    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color,
    );
    var out: FragmentOutput;
    // Unlit: the light is baked into the vertex colours, so real lights would
    // only add a shine.
    out.color = pbr_input.material.base_color;
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
