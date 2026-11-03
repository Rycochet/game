// The black edge round item pictures (see docs/context/client/item_art.md): over a
// page of models, every pixel within `radius` of one is black, with the model on top.
// The page and what this draws to are the same size, so a pixel here is the same
// pixel there.

import bevy_pbr::render::forward_io::VertexOutput;

struct Edge {
    radius: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> edge: Edge;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var models: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var models_sampler: sampler;

// How much of a pixel a model must cover to count as there.
const COVERED: f32 = 0.5;

fn model_at(at: vec2<f32>, size: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(models, models_sampler, (at + 0.5) / size, 0.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(models));
    let at = floor(in.position.xy);
    let own = model_at(at, size);
    // Models were drawn over nothing, so their colour is already as over black.
    let over_black = vec4(own.rgb, 1.0);
    if own.a >= COVERED {
        return over_black;
    }
    // Round: every pixel whose middle is within the radius.
    let reach = i32(ceil(edge.radius));
    let most = edge.radius * edge.radius + 0.25;
    for (var dy = -reach; dy <= reach; dy += 1) {
        for (var dx = -reach; dx <= reach; dx += 1) {
            if f32(dx * dx + dy * dy) <= most && model_at(at + vec2(f32(dx), f32(dy)), size).a >= COVERED {
                return over_black;
            }
        }
    }
    return vec4(0.0);
}
