// How good a skill is, as a ring round the inside of its circle's edge: filled
// clockwise from the top by `fill` in one colour (the quality of its level), with a
// dark track where it isn't yet. A specialised skill has a second, thinner ring
// inside it (`special` 0 or more; below 0 there is none), filled by `special`,
// drawn as the metal edge of a set item (`set_edge.wesl`) in `special_colour`, its
// bands sliding with `shift`.

import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct Ring {
    colour: vec4<f32>,
    special_colour: vec4<f32>,
    fill: f32,
    special: f32,
    shift: f32,
    // Where the ring starts, as a share of the circle's radius.
    inner: f32,
}

@group(1) @binding(0) var<uniform> ring: Ring;

const BAND_PIXELS: f32 = 58.0;
const DARKEST: f32 = 0.45;
const BRIGHTEST: f32 = 1.2;
const GLINT: f32 = 0.55;
const TAU: f32 = 6.2831855;
const TRACK: vec4<f32> = vec4(0.0, 0.0, 0.0, 0.35);
// The second ring, inside the first: the gap between them and its width, as shares
// of the circle's radius.
const SPECIAL_GAP: f32 = 0.03;
const SPECIAL_WIDTH: f32 = 0.12;

// How much of a pixel at `distance` lies within a band from `inner` to `outer`,
// soft by a pixel at both edges.
fn band(distance: f32, inner: f32, outer: f32) -> f32 {
    return clamp(outer - distance, 0.0, 1.0) * clamp(distance - inner, 0.0, 1.0);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let radius = min(in.size.x, in.size.y) / 2.0;
    let from_middle = in.uv * in.size - in.size / 2.0;
    let distance = length(from_middle);
    let main_cover = band(distance, radius * ring.inner, radius);
    var special_cover = 0.0;
    if ring.special >= 0.0 {
        let outer = radius * (ring.inner - SPECIAL_GAP);
        special_cover = band(distance, outer - radius * SPECIAL_WIDTH, outer);
    }
    if main_cover <= 0.0 && special_cover <= 0.0 {
        discard;
    }
    // Clockwise from the top, as a share of a turn (y is down).
    let turn = fract(atan2(from_middle.x, -from_middle.y) / TAU + 1.0);
    var colour = TRACK;
    var cover = main_cover;
    if special_cover > 0.0 {
        cover = special_cover;
        if turn < ring.special {
            let along = (in.position.x + in.position.y + ring.shift) / BAND_PIXELS;
            let light = 0.5 + 0.5 * sin(along * TAU);
            let shine = mix(DARKEST, BRIGHTEST, light);
            let glint = GLINT * pow(light, 12.0);
            colour = vec4(ring.special_colour.rgb * shine + vec3(glint), 1.0);
        }
    } else if turn < ring.fill {
        colour = vec4(ring.colour.rgb, 1.0);
    }
    return vec4(colour.rgb, colour.a * cover);
}
