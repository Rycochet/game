// Something waiting its turn: yellow and black hazard stripes in the node's
// border, chasing round it clockwise. Each pixel's place along the edge (from
// the top left corner, clockwise) plus its depth into the border gives the
// stripes their slant; `shift` moves them along.

import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct Stripe {
    // x: how far the stripes have moved, in pixels; the rest unused.
    params: vec4<f32>,
}

@group(1) @binding(0) var<uniform> stripe: Stripe;

const STRIPE_PIXELS: f32 = 12.0;
const YELLOW: vec3<f32> = vec3(1.0, 0.78, 0.0);
const BLACK: vec3<f32> = vec3(0.02, 0.02, 0.02);

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Only its edge: the node's border, in pixels: left, top, right, bottom.
    let widths = in.border_widths;
    let size = in.size;
    let at = in.uv * size;
    let depths = vec4(at.x, at.y, size.x - at.x, size.y - at.y);
    if depths.x > widths.x && depths.y > widths.y && depths.z > widths.z && depths.w > widths.w {
        discard;
    }
    // The nearest side, and how far along the edge, clockwise from the top left.
    let nearest = min(min(depths.x, depths.y), min(depths.z, depths.w));
    var along: f32;
    if nearest == depths.y {
        along = at.x;
    } else if nearest == depths.z {
        along = size.x + at.y;
    } else if nearest == depths.w {
        along = size.x + size.y + (size.x - at.x);
    } else {
        along = 2.0 * size.x + size.y + (size.y - at.y);
    }
    let place = (along + nearest - stripe.params.x) / STRIPE_PIXELS;
    let colour = select(BLACK, YELLOW, fract(place) < 0.5);
    return vec4(colour, 1.0);
}
