// The glow of an item, fading between two colours and going no further than its node:
// from the outline of its mask, either outside it or inside it.

import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct Aura {
    first: vec4<f32>,
    second: vec4<f32>,
    reach: vec2<f32>,
    inward: f32,
    time: f32,
}

@group(1) @binding(0) var<uniform> aura: Aura;
@group(1) @binding(1) var mask: texture_2d<f32>;
@group(1) @binding(2) var mask_sampler: sampler;

const RINGS: i32 = 8;
const DIRECTIONS: i32 = 16;
const TAU: f32 = 6.2831855;
// Radians a second: a slow pulse, a slower fade between colours, and a band of light
// drifting across.
const PULSE: f32 = 1.6;
const FADE: f32 = 0.8;
const DRIFT: f32 = 1.2;
const DIMMEST: f32 = 0.7;
// How much dimmer the glow is furthest from the band of light.
const BAND_DIMMEST: f32 = 0.35;

fn solid_at(uv: vec2<f32>) -> f32 {
    let inside = all(uv >= vec2(0.0)) && all(uv <= vec2(1.0));
    let solid = textureSampleLevel(mask, mask_sampler, clamp(uv, vec2(0.0), vec2(1.0)), 0.0).a;
    return select(0.0, solid, inside);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let inward = aura.inward > 0.5;
    var glow = 0.0;
    for (var ring = 1; ring <= RINGS; ring += 1) {
        let reach = f32(ring) / f32(RINGS);
        let strength = 1.0 - (reach - 1.0 / f32(RINGS));
        for (var direction = 0; direction < DIRECTIONS; direction += 1) {
            let angle = TAU * f32(direction) / f32(DIRECTIONS);
            let offset = vec2(cos(angle), sin(angle)) * aura.reach * reach;
            let solid = solid_at(uv + offset);
            // Looking for the other side of the outline: past the edge of the node
            // counts as empty, so plain cells glow in from their edges.
            let found = select(solid, 1.0 - solid, inward);
            glow = max(glow, found * strength);
        }
    }
    let pulse = mix(DIMMEST, 1.0, 0.5 + 0.5 * sin(aura.time * PULSE));
    let colour = mix(aura.first, aura.second, 0.5 + 0.5 * sin(aura.time * FADE));
    let centre = solid_at(uv);
    let here = select(1.0 - centre, centre, inward);
    let band = mix(BAND_DIMMEST, 1.0, 0.5 + 0.5 * sin((uv.x + uv.y) * TAU - aura.time * DRIFT));
    return vec4(colour.rgb, glow * pulse * band * here);
}
