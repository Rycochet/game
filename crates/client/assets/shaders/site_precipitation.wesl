// Falling rain, sleet, snow and hail over a site's outside ground. The mesh's
// quads hold each drop's place on the ground and a random number; one weather
// for the whole site says whether it falls and as what. Kinds, sizes and speeds
// match `precipitation.wesl`. Nothing here affects the game.

import bevy_pbr::render::mesh_view_bindings::view;

struct Weather {
    rain: f32,
    temperature: f32,
    time: f32,
    fall_height: f32,
    light: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> weather: Weather;

struct Vertex {
    @location(0) drop: vec3<f32>,
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

@vertex
fn vertex(in: Vertex) -> Output {
    var out: Output;
    out.corner = in.corner;
    out.is_round = 0.0;
    // Not drawn, unless it turns out below that it falls.
    out.clip = vec4<f32>(0.0, 0.0, 2.0, 1.0);
    out.colour = vec4<f32>(0.0);

    // Harder rain lights up more of the quads.
    let random = in.drop.y;
    if weather.rain * 1.05 < fract(random * 57.3) + 0.03 {
        return out;
    }

    var speed = 70.0;
    var size = vec2<f32>(0.3, 5.0);
    var colour = vec4<f32>(0.65, 0.75, 0.95, 0.55);
    if weather.temperature < SNOW_BELOW {
        speed = 11.0;
        size = vec2<f32>(0.9, 0.9);
        colour = vec4<f32>(1.0, 1.0, 1.0, 0.9);
        out.is_round = 1.0;
    } else if weather.temperature < SLEET_BELOW {
        speed = 38.0;
        size = vec2<f32>(0.5, 2.2);
        colour = vec4<f32>(0.85, 0.9, 1.0, 0.75);
    } else if weather.rain > HAIL_ABOVE {
        speed = 90.0;
        size = vec2<f32>(0.8, 0.8);
        colour = vec4<f32>(0.95, 0.97, 1.0, 1.0);
        out.is_round = 1.0;
    }

    let height = weather.fall_height * fract(random - weather.time * speed / weather.fall_height);
    let view_right = view.world_from_view[0].xyz;
    let place = vec3<f32>(in.drop.x, height, in.drop.z)
        + view_right * in.corner.x * size.x
        + vec3<f32>(0.0, 1.0, 0.0) * in.corner.y * size.y;
    out.clip = view.clip_from_world * vec4<f32>(place, 1.0);
    out.colour = vec4<f32>(colour.rgb * weather.light.rgb, colour.a);
    return out;
}

@fragment
fn fragment(in: Output) -> @location(0) vec4<f32> {
    if in.is_round > 0.5 && dot(in.corner, in.corner) > 0.25 {
        discard;
    }
    return in.colour;
}
