// A dungeon surface: the standard material, shown only on the side of the sight
// mask it belongs to, so the edge of sight falls part way across a cell. See
// docs/context/client/dungeon_map.md.

import bevy_pbr::render::{
    forward_io::{FragmentOutput, VertexOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
};

struct Mask {
    origin: vec2<f32>,
    per_unit: vec2<f32>,
    // 1 only in sight, -1 only out of sight, 0 everywhere.
    side: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> mask: Mask;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var mask_picture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var mask_sampler: sampler;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    // Sampled before anything can return, as derivatives need every fragment.
    let place = (in.world_position.xz - mask.origin) * mask.per_unit;
    let in_sight = textureSample(mask_picture, mask_sampler, place).r > 0.5;
    if (mask.side > 0.5 && !in_sight) || (mask.side < -0.5 && in_sight) {
        discard;
    }

    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color,
    );
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
