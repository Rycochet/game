//! The shapes of a bag on the ground and of its sparkle, without Bevy so they can be
//! tested alone: a canvas sack turned about its middle, tied near the top, as full as
//! its `Load` looks. Everything stands on `y = 0`, in 3D units.

use std::f32::consts::TAU;

use shared::ground::Load;

/// How many ways round the bag goes, and up it.
const SEGMENTS: usize = 24;
const RINGS: usize = 22;
/// How many ways round the rope's own thickness goes, and along it.
const ROPE_SIDES: usize = 6;
const ROPE_SEGMENTS: usize = 16;
/// How far the outline stands out from a bag, always and while the pointer is on it.
const OUTLINE_WIDTH: f32 = 0.4;
const HOVER_OUTLINE_WIDTH: f32 = 0.9;
/// How many times the weave's picture goes round and up a bag.
const WEAVE_ROUND: f32 = 4.0;
const WEAVE_UP: f32 = 3.0;
const CANVAS: [f32; 3] = [0.74, 0.63, 0.45];
const ROPE: [f32; 3] = [0.3, 0.2, 0.11];
/// How dark the folds go, and memory's shade.
const FOLD_SHADE: f32 = 0.72;
const MEMORY_SHADE: f32 = 0.22;

/// How many different bags of each look there are.
pub(super) const VARIETIES: u8 = 3;

/// What a shape is made of, ready for a Bevy mesh.
#[derive(Default)]
pub(super) struct Model {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colours: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

/// The size of a bag of a look: how wide and tall, how far its top slumps over, and how
/// big it is drawn, as part of the biggest size.
#[derive(Clone, Copy)]
pub(super) struct Size {
    pub radius: f32,
    pub height: f32,
    lean: f32,
    pub scale: f32,
}

pub(super) fn size_of(load: Load) -> Size {
    let (radius, height, lean, scale) = match load {
        Load::Flat => (8.5, 3.6, 4.0, 0.5),
        Load::Slack => (7.5, 6.5, 2.0, 0.625),
        Load::Half => (7.0, 10.5, 1.0, 0.75),
        Load::Full => (8.0, 13.5, 0.6, 0.875),
        Load::Bulging => (9.5, 16.0, 0.3, 1.0),
    };
    Size {
        radius: radius * scale,
        height: height * scale,
        lean: lean * scale,
        scale,
    }
}

/// A bag's outline from the bottom's middle to the middle of its opening, as part of
/// its radius and height.
fn profile(load: Load) -> [(f32, f32); 10] {
    // Where the belly is: the less in it, the lower it all sags.
    let (belly, shoulder) = match load {
        Load::Flat | Load::Slack => (0.2, 0.42),
        Load::Half => (0.26, 0.5),
        Load::Full => (0.33, 0.56),
        Load::Bulging => (0.38, 0.6),
    };
    [
        (0.0, 0.0),
        (0.72, 0.0),
        (1.0, belly),
        (0.88, shoulder),
        (0.34, TIED_AT - 0.08),
        (0.2, TIED_AT),
        (0.36, 0.91),
        (0.5, 1.0),
        (0.28, 0.93),
        (0.0, 0.88),
    ]
}

/// How far up its height the rope is.
const TIED_AT: f32 = 0.78;

/// Along a line of points: a smooth curve through them all, `along` from 0 to 1.
fn on_curve(points: &[(f32, f32)], along: f32) -> (f32, f32) {
    let spans = points.len() - 1;
    let at = along.clamp(0.0, 1.0) * spans as f32;
    let span = (at as usize).min(spans - 1);
    let t = at - span as f32;
    let point = |index: isize| points[index.clamp(0, spans as isize) as usize];
    let span = span as isize;
    let (a, b, c, d) = (
        point(span - 1),
        point(span),
        point(span + 1),
        point(span + 2),
    );
    let blend = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * (2.0 * b
            + (c - a) * t
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * t * t
            + (3.0 * b - a - 3.0 * c + d) * t * t * t)
    };
    (
        blend(a.0, b.0, c.0, d.0),
        blend(a.1, b.1, c.1, d.1).max(0.0),
    )
}

impl Model {
    fn add_triangle(&mut self, a: usize, b: usize, c: usize) {
        self.indices.extend([a as u32, b as u32, c as u32]);
    }

    /// Normals from the triangles touching each point, added to what any have.
    fn smooth_normals(&mut self, first_point: usize, first_index: usize) {
        for point in first_point..self.positions.len() {
            self.normals[point] = [0.0; 3];
        }
        let triangles: Vec<[usize; 3]> = self.indices[first_index..]
            .chunks(3)
            .map(|triangle| {
                [
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                ]
            })
            .collect();
        for [a, b, c] in triangles {
            let [pa, pb, pc] = [a, b, c].map(|i| self.positions[i]);
            let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let cross = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            for i in [a, b, c] {
                let normal = &mut self.normals[i];
                *normal = [
                    normal[0] + cross[0],
                    normal[1] + cross[1],
                    normal[2] + cross[2],
                ];
            }
        }
        for normal in &mut self.normals[first_point..] {
            let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
                .sqrt()
                .max(1e-6);
            *normal = normal.map(|axis| axis / length);
        }
    }

    /// The same shape, grown along its normals: seen from inside, what stands out past
    /// the bag is its outline.
    pub fn grown(&self, by: f32) -> Self {
        let mut grown = Self {
            positions: Vec::with_capacity(self.positions.len()),
            normals: self.normals.clone(),
            uvs: self.uvs.clone(),
            colours: self.colours.clone(),
            indices: self.indices.clone(),
        };
        for (position, normal) in self.positions.iter().zip(&self.normals) {
            grown.positions.push([
                position[0] + normal[0] * by,
                position[1] + normal[1] * by,
                position[2] + normal[2] * by,
            ]);
        }
        grown
    }

    /// The same shape, grey, for memory.
    pub fn greyed(&self) -> Self {
        let colours = self.colours.iter().map(|colour| {
            let grey = (0.3 * colour[0] + 0.59 * colour[1] + 0.11 * colour[2]) * MEMORY_SHADE;
            [grey, grey, grey, 1.0]
        });
        Self {
            positions: self.positions.clone(),
            normals: self.normals.clone(),
            uvs: self.uvs.clone(),
            colours: colours.collect(),
            indices: self.indices.clone(),
        }
    }
}

/// A bag of this look. Each variety has its own lumps and slumps its own way.
pub(super) fn bag(load: Load, variety: u8) -> Model {
    let size = size_of(load);
    let points = profile(load);
    let phase = variety as f32 * 2.1 + 0.4;
    let lean_way = (phase.cos(), phase.sin());
    // Where a height is pushed to by the slump: more towards the top.
    let leaning = |up: f32| {
        let by = size.lean * up * up;
        (lean_way.0 * by, lean_way.1 * by)
    };
    let mut model = Model::default();
    for ring in 0..=RINGS {
        let along = ring as f32 / RINGS as f32;
        let (across, up) = on_curve(&points, along);
        let (shift_x, shift_z) = leaning(up);
        // Lumps only on the belly, and pleats where it is gathered.
        let belly = (up / TIED_AT).min(1.0);
        let belly = (belly * std::f32::consts::PI).sin().max(0.0);
        let frill = ((up - TIED_AT) / (1.0 - TIED_AT)).clamp(0.0, 1.0);
        for segment in 0..=SEGMENTS {
            // The last point round is the first again, with its own picture place.
            let round = (segment % SEGMENTS) as f32 / SEGMENTS as f32 * TAU;
            let lumps = 1.0
                + belly
                    * (0.1 * (2.0 * round + phase).sin()
                        + 0.07 * (3.0 * round + phase * 2.0).sin()
                        + 0.03 * (7.0 * round + phase * 3.0).sin());
            let pleats = 1.0 + frill * 0.22 * (9.0 * round + phase).sin();
            let radius = size.radius * across * lumps * pleats;
            let fold = 0.5 + 0.5 * (5.0 * round + phase).sin() * (1.0 - belly);
            let shade = 1.0 - (1.0 - FOLD_SHADE) * fold.clamp(0.0, 1.0);
            model.positions.push([
                radius * round.cos() + shift_x,
                size.height * up,
                radius * round.sin() + shift_z,
            ]);
            model.normals.push([0.0; 3]);
            model.uvs.push([
                segment as f32 / SEGMENTS as f32 * WEAVE_ROUND,
                along * WEAVE_UP,
            ]);
            model
                .colours
                .push([CANVAS[0] * shade, CANVAS[1] * shade, CANVAS[2] * shade, 1.0]);
        }
    }
    let row = SEGMENTS + 1;
    for ring in 0..RINGS {
        for segment in 0..SEGMENTS {
            let low = ring * row + segment;
            let high = low + row;
            model.add_triangle(low, high, high + 1);
            model.add_triangle(low, high + 1, low + 1);
        }
    }
    model.smooth_normals(0, 0);
    // Both ends of the seam share a normal, so it doesn't show.
    for ring in 0..=RINGS {
        let (first, last) = (ring * row, ring * row + SEGMENTS);
        let joined = model.normals[first];
        model.normals[last] = joined;
    }
    add_rope(&mut model, load, leaning);
    model
}

/// The rope round the neck.
fn add_rope(model: &mut Model, load: Load, leaning: impl Fn(f32) -> (f32, f32)) {
    let size = size_of(load);
    let (shift_x, shift_z) = leaning(TIED_AT);
    let thickness = (size.height * 0.06).clamp(0.15, 0.7);
    let round = size.radius * 0.2 + thickness * 0.6;
    let middle_up = size.height * TIED_AT;
    let first_point = model.positions.len();
    let first_index = model.indices.len();
    for along in 0..ROPE_SEGMENTS {
        let along = along as f32 / ROPE_SEGMENTS as f32 * TAU;
        let (out_x, out_z) = (along.cos(), along.sin());
        for side in 0..ROPE_SIDES {
            let side = side as f32 / ROPE_SIDES as f32 * TAU;
            let radius = round + thickness * side.cos();
            model.positions.push([
                radius * out_x + shift_x,
                middle_up + thickness * side.sin(),
                radius * out_z + shift_z,
            ]);
            model.normals.push([0.0; 3]);
            model.uvs.push([0.0, 0.0]);
            model.colours.push([ROPE[0], ROPE[1], ROPE[2], 1.0]);
        }
    }
    let point = |along: usize, side: usize| {
        first_point + (along % ROPE_SEGMENTS) * ROPE_SIDES + side % ROPE_SIDES
    };
    for along in 0..ROPE_SEGMENTS {
        for side in 0..ROPE_SIDES {
            let (a, b) = (point(along, side), point(along + 1, side));
            let (c, d) = (point(along + 1, side + 1), point(along, side + 1));
            model.add_triangle(a, c, b);
            model.add_triangle(a, d, c);
        }
    }
    model.smooth_normals(first_point, first_index);
}

/// A bag's outline shape.
pub(super) fn outline(model: &Model) -> Model {
    model.grown(OUTLINE_WIDTH)
}

/// The wider outline of a bag the pointer is on.
pub(super) fn hover_outline(model: &Model) -> Model {
    model.grown(HOVER_OUTLINE_WIDTH)
}

/// A flat star with four long points, facing +z, about its middle: what a sparkle is
/// made of. `grow` makes it larger all round, for its outline.
pub(super) fn star(long: f32, short: f32, grow: f32) -> Model {
    let mut model = Model {
        positions: vec![[0.0; 3]],
        normals: vec![[0.0, 0.0, 1.0]],
        uvs: vec![[0.0, 0.0]],
        colours: vec![[1.0; 4]],
        indices: Vec::new(),
    };
    const POINTS: usize = 8;
    for point in 0..POINTS {
        let angle = point as f32 / POINTS as f32 * TAU;
        let radius = if point % 2 == 0 { long } else { short } + grow;
        model
            .positions
            .push([radius * angle.cos(), radius * angle.sin(), 0.0]);
        model.normals.push([0.0, 0.0, 1.0]);
        model.uvs.push([0.0, 0.0]);
        model.colours.push([1.0; 4]);
    }
    for point in 0..POINTS {
        model.add_triangle(0, point + 1, (point + 1) % POINTS + 1);
    }
    model
}

#[cfg(test)]
mod tests;
