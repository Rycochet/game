// The edge of a set item: its quality's colour, as metal catching a light in
// diagonal bands. The bands go by where on the screen a pixel is, so every set item
// shows the same ones, and `shift` slides them all as the view changes.

import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct Edge {
    colour: vec4<f32>,
    shift: f32,
}

@group(1) @binding(0) var<uniform> edge: Edge;

const BAND_PIXELS: f32 = 58.0;
const DARKEST: f32 = 0.45;
const BRIGHTEST: f32 = 1.2;
const GLINT: f32 = 0.55;
const TAU: f32 = 6.2831855;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Only its edge: the node's border, in pixels: left, top, right, bottom.
    let widths = in.border_widths;
    let at = in.uv * in.size;
    if at.x > widths.x && at.y > widths.y && at.x < in.size.x - widths.z && at.y < in.size.y - widths.w {
        discard;
    }
    let along = (in.position.x + in.position.y + edge.shift) / BAND_PIXELS;
    let light = 0.5 + 0.5 * sin(along * TAU);
    // A broad shine, with a narrow white glint at its height, for colours already
    // at their brightest.
    let shine = mix(DARKEST, BRIGHTEST, light);
    let glint = GLINT * pow(light, 12.0);
    return vec4(edge.colour.rgb * shine + vec3(glint), 1.0);
}
