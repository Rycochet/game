//! Placeholder 3D models of world map pois, built from simple shapes to match
//! what the poi leads to (`PoiLook`): a building's outline, size and floors, a
//! town's number of buildings. The same triangles make the `Mesh` on the world
//! map and, projected isometrically, its picture on the maps (`picture`).
//!
//! Units are world grid points: `x` east, `y` up, `z` south, with the ground at
//! `y` 0 and the poi at the origin. Every model stays within `REACH` of its
//! centre, the size of the base it replaced.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use shared::poi::PoiKind;
use shared::site::{CaveEntrance, Outline, PoiLook};

/// How far a model reaches from its centre.
const REACH: f32 = 8.0;

/// How wide each piece of a ruined building's wall is.
const WALL_STONE: f32 = 1.2;

/// How far walls carry on below the ground, so a model on a slope never floats.
const FOUNDATION: f32 = 8.0;

/// One floor's height.
const FLOOR: f32 = 2.2;

/// The longest side of a building that still grows its model: anything longer
/// is drawn at `REACH`.
const CELLS_AT_REACH: f32 = 50.0;

/// A picture shows this many grid points across, or the whole model if bigger,
/// so a hut looks smaller than a castle.
const PICTURE_SPAN: f32 = 18.0;

/// Sunlight on a picture's faces, and the light they always get.
const PICTURE_LIGHT: [f32; 3] = [-0.45, 0.8, 0.4];
const PICTURE_AMBIENT: f32 = 0.45;

type Rgb = [u8; 3];

const EARTH: Rgb = [112, 92, 66];
const WOOD: Rgb = [122, 86, 52];
const THATCH: Rgb = [186, 158, 88];
const STONE: Rgb = [146, 144, 138];
const DARK_STONE: Rgb = [98, 98, 100];
const ADOBE: Rgb = [204, 164, 112];
const PLASTER: Rgb = [216, 204, 178];
const RED_ROOF: Rgb = [168, 62, 46];
const GREY_ROOF: Rgb = [92, 96, 106];
const SNOW: Rgb = [232, 238, 244];
const ICE: Rgb = [176, 214, 232];
const MOSS: Rgb = [94, 124, 70];
const BLOOD: Rgb = [120, 30, 30];
const SAND: Rgb = [214, 186, 122];
const GRASS: Rgb = [108, 138, 72];
const ROCK: Rgb = [118, 112, 104];
const DARK: Rgb = [20, 18, 18];

/// Triangles with one colour and normal each.
#[derive(Default)]
pub struct PoiModel {
    /// Three corners per triangle, in the order they face outwards.
    corners: Vec<[Vec3; 3]>,
    colours: Vec<Rgb>,
}

impl PoiModel {
    /// A triangle facing away from `inside`.
    fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3, inside: Vec3, colour: Rgb) {
        let normal = (b - a).cross(c - a);
        let corners = if normal.dot(a - inside) < 0.0 {
            [a, c, b]
        } else {
            [a, b, c]
        };
        self.corners.push(corners);
        self.colours.push(colour);
    }

    /// Upright walls round `ring` (points on the ground plane, relative to
    /// `at`) from `from` to `to`, with a flat lid.
    fn prism(&mut self, at: Vec2, ring: &[Vec2], (from, to): (f32, f32), colour: Rgb) {
        let inside = Vec3::new(at.x, (from + to) / 2.0, at.y);
        let point = |p: Vec2, y: f32| Vec3::new(at.x + p.x, y, at.y + p.y);
        for (i, &p) in ring.iter().enumerate() {
            let q = ring[(i + 1) % ring.len()];
            let (a, b, c, d) = (point(p, from), point(q, from), point(q, to), point(p, to));
            self.triangle(a, b, c, inside, colour);
            self.triangle(a, c, d, inside, colour);
        }
        let top = Vec3::new(at.x, to, at.y);
        for (i, &p) in ring.iter().enumerate() {
            let q = ring[(i + 1) % ring.len()];
            self.triangle(point(p, to), point(q, to), top, inside, colour);
        }
    }

    /// Slopes from `ring` at height `from` up to a point at `from + rise`.
    fn cone(&mut self, at: Vec2, ring: &[Vec2], from: f32, rise: f32, colour: Rgb) {
        let inside = Vec3::new(at.x, from, at.y);
        let tip = Vec3::new(at.x, from + rise, at.y);
        let point = |p: Vec2| Vec3::new(at.x + p.x, from, at.y + p.y);
        for (i, &p) in ring.iter().enumerate() {
            let q = ring[(i + 1) % ring.len()];
            self.triangle(point(p), point(q), tip, inside, colour);
        }
    }

    /// A box `half` wide each way, from below the ground to `height`.
    fn block(&mut self, at: Vec2, half: Vec2, height: f32, colour: Rgb) {
        self.prism(at, &rectangle(half), (-FOUNDATION, height), colour);
    }

    /// A dark opening on the south face, at `z` in front of `x`.
    fn doorway(&mut self, x: f32, z: f32, half_width: f32, height: f32) {
        let inside = Vec3::new(x, height / 2.0, z - 1.0);
        let corner = |dx: f32, y: f32| Vec3::new(x + dx, y, z + 0.05);
        let (a, b) = (corner(-half_width, 0.0), corner(half_width, 0.0));
        let (c, d) = (corner(half_width, height), corner(-half_width, height));
        self.triangle(a, b, c, inside, DARK);
        self.triangle(a, c, d, inside, DARK);
    }

    /// A house with a pointed roof.
    fn house(&mut self, at: Vec2, half: Vec2, height: f32, walls: Rgb, roof: Rgb) {
        self.block(at, half, height, walls);
        let eaves = half + Vec2::splat(0.3);
        self.cone(
            at,
            &rectangle(eaves),
            height,
            half.min_element() * 1.2,
            roof,
        );
        self.doorway(at.x, at.y + half.y, 0.3, (height * 0.6).min(1.0));
    }

    /// Battlements round the top of `ring`.
    fn battlements(&mut self, at: Vec2, ring: &[Vec2], height: f32, colour: Rgb) {
        for &p in ring {
            let top = (height, height + 0.8);
            self.prism(at + p, &rectangle(Vec2::splat(0.4)), top, colour);
        }
    }

    /// Makes the `Mesh`.
    pub fn mesh(&self) -> Mesh {
        let mut positions = Vec::with_capacity(self.corners.len() * 3);
        let mut normals = Vec::with_capacity(self.corners.len() * 3);
        let mut colours = Vec::with_capacity(self.corners.len() * 3);
        for (corners, colour) in self.corners.iter().zip(&self.colours) {
            let [a, b, c] = *corners;
            let normal = (b - a).cross(c - a).normalize_or_zero();
            let [red, green, blue] = *colour;
            let linear = Color::srgb_u8(red, green, blue).to_linear();
            for corner in corners {
                positions.push(corner.to_array());
                normals.push(normal.to_array());
                colours.push(linear.to_f32_array());
            }
        }
        let indices = (0..positions.len() as u32).collect();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
    }

    /// The model seen from the south-east from above, `size` pixels square, as
    /// RGBA with a see-through background: an isometric picture for the maps.
    pub fn picture(&self, size: u32) -> Vec<u8> {
        // Screen axes and the direction towards the viewer.
        let right = Vec3::new(1.0, 0.0, -1.0).normalize();
        let down = Vec3::new(1.0, -1.2, 1.0).normalize();
        let towards = right.cross(down).normalize();
        let towards = if towards.y < 0.0 { -towards } else { towards };
        let above_ground = |corner: &Vec3| corner.y >= -0.01;
        let shown = self.corners.iter().flatten().filter(|c| above_ground(c));
        let (mut min, mut max) = (Vec2::MAX, Vec2::MIN);
        for corner in shown {
            let at = Vec2::new(corner.dot(right), corner.dot(down));
            (min, max) = (min.min(at), max.max(at));
        }
        let span = (max - min).max_element().max(PICTURE_SPAN);
        let scale = size as f32 / span;
        // Centred, with the feet near the bottom.
        let offset = Vec2::new(
            (size as f32 - (max.x - min.x) * scale) / 2.0 - min.x * scale,
            size as f32 - max.y * scale,
        );
        let light = Vec3::from(PICTURE_LIGHT).normalize();

        let side = size as usize;
        let mut pixels = vec![0; side * side * 4];
        let mut nearest = vec![f32::MIN; side * side];
        for (corners, colour) in self.corners.iter().zip(&self.colours) {
            let [a, b, c] = *corners;
            let normal = (b - a).cross(c - a).normalize_or_zero();
            if normal.dot(towards) <= 0.0 {
                continue;
            }
            let shade = (PICTURE_AMBIENT + normal.dot(light).max(0.0)).min(1.2);
            let on_screen = |p: Vec3| Vec2::new(p.dot(right), p.dot(down)) * scale + offset;
            let (pa, pb, pc) = (on_screen(a), on_screen(b), on_screen(c));
            let area = edge(pa, pb, pc);
            if area == 0.0 {
                continue;
            }
            let low = pa.min(pb).min(pc).floor().max(Vec2::ZERO);
            let high = pa
                .max(pb)
                .max(pc)
                .ceil()
                .min(Vec2::splat(size as f32 - 1.0));
            for y in low.y as usize..=high.y as usize {
                for x in low.x as usize..=high.x as usize {
                    let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    let (wa, wb, wc) = (
                        edge(pb, pc, p) / area,
                        edge(pc, pa, p) / area,
                        edge(pa, pb, p) / area,
                    );
                    if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                        continue;
                    }
                    let point = a * wa + b * wb + c * wc;
                    // What is underground never shows.
                    let depth = point.dot(towards);
                    if point.y < 0.0 || depth <= nearest[y * side + x] {
                        continue;
                    }
                    nearest[y * side + x] = depth;
                    let at = (y * side + x) * 4;
                    for channel in 0..3 {
                        let lit = f32::from(colour[channel]) * shade;
                        pixels[at + channel] = lit.clamp(0.0, 255.0) as u8;
                    }
                    pixels[at + 3] = 255;
                }
            }
        }
        pixels
    }
}

/// Twice the signed area of a triangle.
fn edge(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b - a).perp_dot(c - a)
}

fn rectangle(half: Vec2) -> Vec<Vec2> {
    vec![
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        half,
        Vec2::new(-half.x, half.y),
    ]
}

/// A ring of points round an ellipse.
fn circle(half: Vec2, points: usize) -> Vec<Vec2> {
    (0..points)
        .map(|i| {
            let angle = i as f32 / points as f32 * TAU;
            Vec2::new(angle.cos() * half.x, angle.sin() * half.y)
        })
        .collect()
}

/// The ring of a building's outline.
fn ring(outline: Outline, half: Vec2) -> Vec<Vec2> {
    let cut = half.min_element() * 0.35;
    match outline {
        Outline::Rectangle => rectangle(half),
        Outline::CutCorners => vec![
            Vec2::new(-half.x + cut, -half.y),
            Vec2::new(half.x - cut, -half.y),
            Vec2::new(half.x, -half.y + cut),
            Vec2::new(half.x, half.y - cut),
            Vec2::new(half.x - cut, half.y),
            Vec2::new(-half.x + cut, half.y),
            Vec2::new(-half.x, half.y - cut),
            Vec2::new(-half.x, -half.y + cut),
        ],
        Outline::RoundCorners | Outline::Circle => circle(half, 12),
    }
}

/// Half a building's width on the model, from its width in cells.
fn half_size(cells: u16) -> f32 {
    let share = (f32::from(cells) / CELLS_AT_REACH).min(1.0);
    2.5 + share * 3.7
}

/// The model of a poi. `variety` makes towns of the same size differ.
pub fn build(kind: PoiKind, look: PoiLook, variety: u64) -> PoiModel {
    let mut model = PoiModel::default();
    match look {
        PoiLook::Town { buildings } => town(&mut model, buildings, variety),
        PoiLook::Building {
            outline,
            width,
            height,
            floors,
            ruin,
        } => {
            let half = Vec2::new(half_size(width), half_size(height));
            if ruin > 0 {
                ruined(&mut model, kind, outline, half, floors, ruin, variety);
            } else {
                building(&mut model, kind, outline, half, floors);
            }
        }
        PoiLook::Cave { entrance } => underground(&mut model, kind, entrance),
    }
    model
}

/// More houses the bigger it is, packed in a disc; the biggest get walls and a
/// keep. Spread by the golden angle, which fills a disc evenly.
fn town(model: &mut PoiModel, buildings: u16, variety: u64) {
    const GOLDEN_ANGLE: f32 = 2.399_963;
    let count = 3 + (f32::from(buildings).sqrt() * 1.6) as usize;
    let big = buildings >= 26;
    let radius = if big {
        REACH * 0.8
    } else {
        3.2 + count as f32 * 0.3
    };
    let turn = (variety % 8) as f32 * 0.7;
    for i in 0..count {
        let share = (i as f32 + 0.5) / count as f32;
        let angle = i as f32 * GOLDEN_ANGLE + turn;
        let at = Vec2::from_angle(angle) * share.sqrt() * radius;
        let pick = (i as u64 + variety).wrapping_mul(2_654_435_761) >> 7;
        let walls = [PLASTER, ADOBE, STONE, WOOD][(pick % 4) as usize];
        let roof = [RED_ROOF, GREY_ROOF, THATCH, RED_ROOF][((pick >> 3) % 4) as usize];
        let size = 0.6 + ((pick >> 5) % 4) as f32 * 0.1;
        model.house(at, Vec2::new(size, size * 0.8), 1.0 + size, walls, roof);
    }
    if big {
        let keep = rectangle(Vec2::splat(1.3));
        model.prism(Vec2::ZERO, &keep, (-FOUNDATION, 4.5), DARK_STONE);
        model.battlements(Vec2::ZERO, &keep, 4.5, DARK_STONE);
    }
    if buildings >= 61 {
        for corner in circle(Vec2::splat(REACH * 0.92), 8) {
            model.block(corner, Vec2::splat(0.6), 2.6, STONE);
        }
    }
}

fn building(model: &mut PoiModel, kind: PoiKind, outline: Outline, half: Vec2, floors: u8) {
    use PoiKind::*;
    let at = Vec2::ZERO;
    let height = FLOOR * f32::from(floors.max(1));
    let shape = ring(outline, half);
    let walls_and_roof = |model: &mut PoiModel, walls, roof, rise| {
        model.prism(at, &shape, (-FOUNDATION, height), walls);
        model.cone(at, &ring(outline, half * 1.08), height, rise, roof);
        model.doorway(at.x, half.y, 0.5, 1.4);
    };
    let flat = |model: &mut PoiModel, walls| {
        model.prism(at, &shape, (-FOUNDATION, height), walls);
        model.doorway(at.x, half.y, 0.5, 1.4);
    };
    let rise = half.min_element() * 0.8;
    match kind {
        ThatchedHut => walls_and_roof(model, WOOD, THATCH, rise),
        StiltHut | GreenStiltHut => {
            let roof = if kind == StiltHut { THATCH } else { MOSS };
            for post in circle(half * 0.8, 4) {
                model.block(post, Vec2::splat(0.2), 1.5, WOOD);
            }
            model.prism(at, &shape, (1.5, 1.5 + height), WOOD);
            model.cone(at, &ring(outline, half * 1.08), 1.5 + height, rise, roof);
        }
        RedTent | BlueTent | GreenTent => {
            let cloth = match kind {
                RedTent => RED_ROOF,
                BlueTent => [58, 84, 164],
                _ => MOSS,
            };
            model.prism(at, &shape, (-FOUNDATION, 0.05), EARTH);
            model.cone(at, &shape, 0.05, half.min_element() * 1.3, cloth);
        }
        IceShelter => {
            model.prism(at, &shape, (-FOUNDATION, 0.05), SNOW);
            let lower = ring(outline, half * 0.75);
            model.prism(at, &shape, (0.05, 0.9), ICE);
            model.prism(at, &lower, (0.9, 1.5), SNOW);
            model.cone(at, &lower, 1.5, half.min_element() * 0.5, ICE);
        }
        AdobeHouse | AdobeStore => flat(model, ADOBE),
        RedRoofCottage => walls_and_roof(model, PLASTER, RED_ROOF, rise),
        GreyRoofCottage => walls_and_roof(model, PLASTER, GREY_ROOF, rise),
        StoneCottage => walls_and_roof(model, STONE, GREY_ROOF, rise),
        SnowyCottage => walls_and_roof(model, PLASTER, SNOW, rise),
        Warehouse => walls_and_roof(model, WOOD, GREY_ROOF, rise * 0.6),
        ArcheryRange => {
            model.prism(at, &shape, (-FOUNDATION, 0.1), GRASS);
            model.block(
                Vec2::new(0.0, -half.y * 0.7),
                Vec2::new(half.x * 0.8, 0.4),
                1.2,
                WOOD,
            );
            for x in [-half.x * 0.5, half.x * 0.5] {
                model.block(Vec2::new(x, half.y * 0.5), Vec2::splat(0.5), 1.6, RED_ROOF);
            }
        }
        StoneTower => {
            model.prism(at, &shape, (-FOUNDATION, height), STONE);
            model.battlements(at, &shape, height, STONE);
            model.doorway(at.x, half.y, 0.4, 1.3);
        }
        WoodenTower => walls_and_roof(model, WOOD, THATCH, rise),
        Castle | Fort => {
            let (colour, wall_height) = if kind == Castle {
                (STONE, 2.6)
            } else {
                (WOOD, 1.8)
            };
            model.prism(at, &shape, (-FOUNDATION, wall_height), colour);
            model.battlements(at, &shape, wall_height, colour);
            model.doorway(at.x, half.y, 0.7, 1.6);
            let keep = ring(outline, half * 0.5);
            model.prism(at, &keep, (wall_height, wall_height + height), DARK_STONE);
            model.battlements(at, &keep, wall_height + height, DARK_STONE);
        }
        StonePyramid | SandPyramid => {
            let colour = if kind == StonePyramid { STONE } else { SAND };
            let steps = floors.max(3);
            let step = half.min_element() / f32::from(steps);
            for i in 0..steps {
                let side = half.min_element() - step * f32::from(i);
                let from = if i == 0 {
                    -FOUNDATION
                } else {
                    f32::from(i) * 1.1
                };
                let to = f32::from(i + 1) * 1.1;
                model.prism(at, &rectangle(Vec2::splat(side)), (from, to), colour);
            }
            model.doorway(at.x, half.min_element(), 0.5, 1.0);
        }
        // Not buildings (`Generator::for_poi`), but still something to see.
        _ => flat(model, STONE),
    }
}

/// What is left of a building: no roof, and broken walls round a bare floor,
/// with more of them gone and the rest lower the more ruined it is.
fn ruined(
    model: &mut PoiModel,
    kind: PoiKind,
    outline: Outline,
    half: Vec2,
    floors: u8,
    ruin: u8,
    variety: u64,
) {
    use PoiKind::*;
    let walls = match kind {
        ThatchedHut | StiltHut | GreenStiltHut | Warehouse | WoodenTower | Fort | ArcheryRange => {
            WOOD
        }
        AdobeHouse | AdobeStore => ADOBE,
        SandPyramid => SAND,
        _ => STONE,
    };
    let shape = ring(outline, half);
    model.prism(Vec2::ZERO, &shape, (-FOUNDATION, 0.1), EARTH);
    let full = FLOOR * f32::from(floors.max(1));
    let harm = f32::from(ruin);
    let mut number = variety;
    for (i, &from) in shape.iter().enumerate() {
        let to = shape[(i + 1) % shape.len()];
        let stones = (from.distance(to) / WALL_STONE).ceil().max(1.0) as usize;
        for stone in 0..stones {
            let (gone, low) = (hashed(&mut number), hashed(&mut number));
            if gone < harm * 0.14 {
                continue;
            }
            let at = from.lerp(to, (stone as f32 + 0.5) / stones as f32);
            let height = (full * (1.0 - low * harm * 0.17)).max(0.4);
            model.block(at, Vec2::splat(WALL_STONE / 2.0), height, walls);
        }
    }
}

/// The next of a run of numbers from `0..1`. Only for how a model looks.
fn hashed(number: &mut u64) -> f32 {
    *number = number
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (*number >> 40) as f32 / (1u64 << 24) as f32
}

/// What stands over a cave or dungeon: its mouth, or the way down.
fn underground(model: &mut PoiModel, kind: PoiKind, entrance: CaveEntrance) {
    use PoiKind::*;
    let at = Vec2::ZERO;
    let tomb = |model: &mut PoiModel, stone: Rgb, roof: Rgb| {
        model.block(at, Vec2::new(3.0, 2.4), 2.4, stone);
        model.cone(at, &rectangle(Vec2::new(3.3, 2.7)), 2.4, 1.6, roof);
        model.doorway(at.x, 2.4, 0.7, 1.6);
    };
    match kind {
        Crypt => tomb(model, DARK_STONE, DARK_STONE),
        MossyCrypt => tomb(model, DARK_STONE, MOSS),
        BloodyCrypt => tomb(model, DARK_STONE, BLOOD),
        Mausoleum => {
            model.block(at, Vec2::new(3.8, 3.0), 2.8, STONE);
            for corner in rectangle(Vec2::new(4.2, 3.4)) {
                model.block(corner, Vec2::splat(0.35), 3.2, PLASTER);
            }
            model.cone(at, &rectangle(Vec2::new(4.8, 4.0)), 3.2, 2.2, DARK_STONE);
            model.doorway(at.x, 3.0, 0.8, 1.8);
        }
        Grave => {
            model.block(at + Vec2::new(0.0, 0.6), Vec2::new(1.2, 2.0), 0.4, EARTH);
            model.block(
                at + Vec2::new(0.0, -1.8),
                Vec2::new(1.0, 0.25),
                2.2,
                DARK_STONE,
            );
        }
        Barrel | Barrels => {
            let spots: &[Vec2] = if kind == Barrel {
                &[Vec2::ZERO]
            } else {
                &[
                    Vec2::new(-1.3, 0.6),
                    Vec2::new(1.3, 0.8),
                    Vec2::new(0.0, -1.2),
                ]
            };
            for &spot in spots {
                let round = circle(Vec2::splat(1.1), 8);
                model.prism(spot, &round, (-FOUNDATION, 2.6), WOOD);
            }
        }
        Boulders => {
            for (spot, size, height) in [
                (Vec2::new(-2.0, 1.0), 2.2, 2.0),
                (Vec2::new(2.2, 0.4), 1.8, 1.4),
                (Vec2::new(0.2, -2.2), 2.6, 2.6),
            ] {
                let round = circle(Vec2::splat(size), 6);
                model.prism(spot, &round, (-FOUNDATION, height * 0.5), ROCK);
                model.cone(spot, &round, height * 0.5, height * 0.8, ROCK);
            }
        }
        Ruins => {
            for (spot, half, height) in [
                (Vec2::new(-3.0, 0.0), Vec2::new(0.5, 3.0), 2.6),
                (Vec2::new(0.5, -3.0), Vec2::new(3.0, 0.5), 1.5),
                (Vec2::new(3.0, 1.5), Vec2::new(0.5, 1.2), 0.9),
            ] {
                model.block(spot, half, height, STONE);
            }
        }
        _ => {
            // A natural cave: a mound with its mouth to the south.
            let colour = match kind {
                GrassyCave => GRASS,
                EarthCave => EARTH,
                IceCave => ICE,
                _ => ROCK,
            };
            let base = circle(Vec2::new(5.5, 4.5), 9);
            model.prism(at, &base, (-FOUNDATION, 1.4), colour);
            model.cone(at, &base, 1.4, 3.4, colour);
            if entrance == CaveEntrance::Opening {
                model.doorway(at.x, 4.5, 1.3, 1.3);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use shared::rng::Seed;

    use super::*;

    fn looks() -> impl Iterator<Item = (PoiKind, PoiLook)> {
        PoiKind::ALL
            .into_iter()
            .flat_map(|kind| (0..8).map(move |seed| (kind, PoiLook::of(kind, Seed::new(seed)))))
    }

    #[test]
    fn every_poi_has_a_model_that_fits_where_its_base_was() {
        for (kind, look) in looks() {
            let model = build(kind, look, 3);
            assert!(!model.corners.is_empty(), "{kind:?}");
            for corner in model.corners.iter().flatten() {
                let reach = corner.xz().length();
                assert!(reach <= REACH * 1.3, "{kind:?} {look:?}: {reach}");
            }
        }
    }

    #[test]
    fn pictures_show_the_model_and_leave_the_rest_clear() {
        for (kind, look) in looks() {
            let pixels = build(kind, look, 0).picture(48);
            let solid = pixels.chunks(4).filter(|pixel| pixel[3] == 255).count();
            assert!(solid > 20, "{kind:?} {look:?}: {solid}");
            assert!(solid < 48 * 48, "{kind:?} {look:?}");
        }
    }

    #[test]
    fn more_floors_stand_taller() {
        let building = |floors| {
            let look = PoiLook::Building {
                outline: Outline::Circle,
                width: 15,
                height: 15,
                floors,
                ruin: 0,
            };
            let model = build(PoiKind::StoneTower, look, 0);
            let top = model.corners.iter().flatten().map(|c| c.y);
            top.fold(f32::MIN, f32::max)
        };
        assert!(building(5) > building(2));
    }

    #[test]
    fn bigger_towns_have_more_to_them() {
        let triangles = |buildings| {
            build(PoiKind::Town, PoiLook::Town { buildings }, 0)
                .corners
                .len()
        };
        assert!(triangles(5) < triangles(20));
        assert!(triangles(20) < triangles(40));
        assert!(triangles(40) < triangles(100));
    }

    #[test]
    fn the_more_ruined_the_less_is_left() {
        let triangles = |ruin| {
            let look = PoiLook::Building {
                outline: Outline::Rectangle,
                width: 30,
                height: 30,
                floors: 2,
                ruin,
            };
            build(PoiKind::Castle, look, 1).corners.len()
        };
        assert!(triangles(5) < triangles(1));
    }
}
