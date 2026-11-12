// A model's hull (see docs/context/client/view3d.md, "Models"), grown along its
// normals by `grow` world units and moved `push` units away from the camera (towards
// it when negative). As the black outline it is grown so many screen pixels (kept so
// as the camera zooms), pushed back and seen from inside (front faces culled), so it
// only shows round the model's edge and where one part stands well clear of another,
// not round every rivet. Through walls (the x-ray's pipeline, view3d/token/hull.rs)
// it is drawn before its own model, and pulled a little towards the camera, clear of
// the base it stands on: a faint fill, then an outline grown round it.

import bevy_pbr::render::mesh_functions;
import bevy_pbr::render::view_transformations::position_world_to_clip;
import bevy_pbr::render::mesh_view_bindings::view;

// 32 bytes, a multiple of 16 as WebGL 2 needs.
struct Hull {
    colour: vec4<f32>,
    grow: f32,
    push: f32,
    unused_a: f32,
    unused_b: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> hull: Hull;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vertex(in: Vertex) -> @builtin(position) vec4<f32> {
    let world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    let local = vec4<f32>(in.position, 1.0);
    var world = mesh_functions::mesh_position_local_to_world(world_from_local, local).xyz;
    let normal = mesh_functions::mesh_normal_local_to_world(in.normal, in.instance_index);
    let away = normalize(world - view.world_position);
    world += normalize(normal) * hull.grow + away * hull.push;
    return position_world_to_clip(world);
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return hull.colour;
}
