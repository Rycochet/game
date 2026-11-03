//! The real edge of what is in sight, for drawing only: a polygon round each
//! viewer, cut by the same thin walls as `shared::dungeon`'s sight, drawn into
//! a mask the floors' shader reads, so the edge falls part way across a cell.
//! The game's own sight stays on the cell grid (`Sight`).
//!
//! The polygon is found by casting rays at the corners of walls, and it uses
//! floats, which is fine as nothing but the picture depends on it.

use std::f64::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType, TextureFormat};
use bevy::shader::ShaderRef;
use shared::dungeon::{CELL_UNITS, Level, Sight};

use super::CELL_SIZE;
use crate::common::{CurrentDungeon, SharesSight, SmoothPosition};
use crate::player::Player;
use crate::view3d::ViewCamera;

/// The mask's pixels each way, however much of the level it covers.
const MASK_PIXELS: u32 = 2048;

/// The layer only the mask's camera draws.
const MASK_LAYER: usize = 1;

/// A ray is cast this far (radians) to each side of a wall's corner, so one
/// passes it and the other stops on it.
const BESIDE_CORNER: f64 = 1.0e-5;

/// Rays also go out this many ways, for open space with no corners in it.
const EVEN_RAYS: usize = 72;

/// The mask covers this much more than the view each way, so it needn't move
/// whenever the camera does.
const MASK_SPARE: f32 = 0.25;

/// A new mask area is worked out when the view has shrunk to this share of it.
const MASK_TOO_BIG: f32 = 0.4;

/// A surface's material: the standard one, shown only inside or only outside
/// what the mask says is in sight.
pub type SurfaceMaterial = ExtendedMaterial<StandardMaterial, SightMask>;

/// Which part of a surface to show. Values match `sight_mask.wesl`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaskSide {
    InSight,
    OutOfSight,
}

impl MaskSide {
    fn value(self) -> f32 {
        match self {
            Self::InSight => 1.0,
            Self::OutOfSight => -1.0,
        }
    }
}

/// Order matches `struct Mask` in the shader.
#[derive(ShaderType, Reflect, Clone, Copy, Debug, Default)]
pub struct MaskParams {
    /// The corner of the mask with the smallest world `x` and `z`.
    origin: Vec2,
    /// One over the world size it covers.
    per_unit: Vec2,
    side: f32,
}

/// Adds the mask to a surface's standard material.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct SightMask {
    #[uniform(100)]
    params: MaskParams,
    #[texture(101)]
    #[sampler(102)]
    mask: Handle<Image>,
}

impl MaterialExtension for SightMask {
    fn fragment_shader() -> ShaderRef {
        "shaders/sight_mask.wesl".into()
    }
}

impl SightMask {
    pub fn new(mask: Handle<Image>, side: MaskSide) -> Self {
        Self {
            params: MaskParams {
                per_unit: Vec2::ONE,
                side: side.value(),
                ..default()
            },
            mask,
        }
    }
}

/// A rectangle on the floor, in world units.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Area {
    min: Vec2,
    max: Vec2,
}

impl Area {
    fn size(self) -> Vec2 {
        self.max - self.min
    }

    fn contains(self, other: Area) -> bool {
        self.min.cmple(other.min).all() && self.max.cmpge(other.max).all()
    }
}

/// What the outlines are worked out from, with what was last worked out.
#[derive(Resource)]
pub struct SightLines {
    walls: WallMap,
    /// Where each viewer was when last worked out.
    from: Vec<Vec2d>,
    /// Open cells any outline reaches into, and the walls touching them.
    pub touched: Option<Sight>,
    /// The same for each viewer: the player first, then those sharing sight.
    pub touched_by: Vec<Sight>,
    /// Cells an outline's edge crosses, or close to it: only part of each is
    /// in sight.
    pub crossed: Option<Sight>,
    /// Counts the times `touched` changed.
    pub changes: u64,
    /// How long the outlines last took, for `ViewStats`.
    pub cost_ms: f32,
    outline_mesh: Handle<Mesh>,
    area: Option<Area>,
}

/// Marks the mask's camera.
#[derive(Component)]
pub struct MaskCamera;

/// Makes the mask with nothing in sight yet, and what draws to it.
pub fn spawn_mask(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<Image> {
    let mask = images.add(Image::new_target_texture(
        MASK_PIXELS,
        MASK_PIXELS,
        TextureFormat::Rgba8Unorm,
        None,
    ));
    let outline_mesh = meshes.add(Mesh::from(Cuboid::from_length(0.0)));
    commands.spawn((
        MaskCamera,
        Camera3d::default(),
        Camera {
            // Before the view's camera, which shows it.
            order: -1,
            clear_color: Color::BLACK.into(),
            is_active: false,
            ..default()
        },
        RenderTarget::Image(mask.clone().into()),
        Projection::Orthographic(OrthographicProjection {
            near: 0.0,
            far: 2.0,
            scaling_mode: ScalingMode::Fixed {
                width: 1.0,
                height: 1.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Tonemapping::None,
        Msaa::Off,
        Transform::from_xyz(0.0, 1.0, 0.0).looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
        RenderLayers::layer(MASK_LAYER),
    ));
    commands.spawn((
        Mesh3d(outline_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
        NoFrustumCulling,
        OutlineMesh,
        RenderLayers::layer(MASK_LAYER),
    ));
    commands.insert_resource(SightLines {
        walls: WallMap::default(),
        from: Vec::new(),
        touched: None,
        touched_by: Vec::new(),
        crossed: None,
        changes: 0,
        cost_ms: 0.0,
        outline_mesh,
        area: None,
    });
    mask
}

/// Marks the mesh of the outlines.
#[derive(Component)]
pub struct OutlineMesh;

/// When a viewer moves or the level changes: the outlines again, what they
/// touch, and the mesh the mask's camera draws.
#[allow(clippy::type_complexity)]
pub fn update_outlines(
    dungeon: Res<CurrentDungeon>,
    player: Single<&SmoothPosition, With<Player>>,
    sharers: Query<&SmoothPosition, (With<SharesSight>, Without<Player>)>,
    mut lines: ResMut<SightLines>,
    mut meshes: ResMut<Assets<Mesh>>,
    outline: Single<&mut Visibility, With<OutlineMesh>>,
) {
    let from: Vec<Vec2d> = std::iter::once(*player)
        .chain(sharers.iter())
        .map(|position| Vec2d {
            x: position.x,
            y: position.y,
        })
        .collect();
    if !dungeon.is_changed() && lines.from == from {
        return;
    }
    let start = Instant::now();
    let level = &dungeon.level;
    if dungeon.is_changed() {
        lines.walls = WallMap::of(level);
    }
    let mut touched_by = Vec::new();
    let mut crossed = Sight::new(level);
    let mut triangles = Vec::new();
    for &viewer in &from {
        let outline = lines.walls.outline(viewer);
        let mut touched = Sight::new(level);
        touch_cells(level, viewer, &outline, &mut touched);
        touched_by.push(touched);
        cross_cells(&outline, &mut crossed);
        add_fan(&mut triangles, viewer, &outline);
    }
    let mut touched = Sight::new(level);
    for one in &touched_by {
        add_cells(level, one, &mut touched);
    }
    let mut visibility = outline.into_inner();
    visibility.set_if_neq(if triangles.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    if !triangles.is_empty()
        && let Some(mut stored) = meshes.get_mut(&lines.outline_mesh)
    {
        *stored = fan_mesh(triangles);
    }
    lines.from = from;
    lines.crossed = Some(crossed);
    lines.cost_ms = start.elapsed().as_secs_f32() * 1000.0;
    if lines.touched_by != touched_by {
        lines.touched = Some(touched);
        lines.touched_by = touched_by;
        lines.changes += 1;
    }
}

/// Moves the mask to cover what the camera shows, keeping it where it is while
/// it still does, so the materials aren't all changed every frame.
#[allow(clippy::type_complexity)]
pub fn place_mask(
    dungeon: Res<CurrentDungeon>,
    view: Single<(&Camera, &Transform), (With<ViewCamera>, Without<MaskCamera>)>,
    mask_camera: Single<(&mut Transform, &mut Projection), With<MaskCamera>>,
    mut lines: ResMut<SightLines>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
) {
    let (camera, transform) = *view;
    let Some(wanted) = view_area(camera, transform, dungeon.level.bounds()) else {
        return;
    };
    if lines.area.is_some_and(|area| {
        let share = wanted.size().element_product() / area.size().element_product();
        area.contains(wanted) && share > MASK_TOO_BIG
    }) {
        return;
    }
    let spare = wanted.size() * MASK_SPARE;
    let area = Area {
        min: wanted.min - spare,
        max: wanted.max + spare,
    };
    lines.area = Some(area);
    let (mut mask_transform, mut projection) = mask_camera.into_inner();
    let centre = (area.min + area.max) / 2.0;
    mask_transform.translation = Vec3::new(centre.x, 1.0, centre.y);
    if let Projection::Orthographic(orthographic) = &mut *projection {
        orthographic.scaling_mode = ScalingMode::Fixed {
            width: area.size().x,
            height: area.size().y,
        };
    }
    let ids: Vec<_> = materials.ids().collect();
    for id in ids {
        if let Some(mut material) = materials.get_mut(id) {
            material.extension.params.origin = area.min;
            material.extension.params.per_unit = area.size().recip();
        }
    }
}

/// The mask's camera only works in this view.
pub fn set_mask_active<const ACTIVE: bool>(mut camera: Single<&mut Camera, With<MaskCamera>>) {
    camera.is_active = ACTIVE;
}

/// The floor the camera shows, as far as the level goes.
fn view_area(camera: &Camera, transform: &Transform, bounds: (i64, i64, i64, i64)) -> Option<Area> {
    let size = camera.logical_viewport_size()?;
    let global = GlobalTransform::from(*transform);
    let far = transform.translation.y.abs() * 8.0 + CELL_SIZE * 8.0;
    let mut low = Vec2::splat(f32::MAX);
    let mut high = Vec2::splat(f32::MIN);
    for corner in [
        Vec2::ZERO,
        Vec2::new(size.x, 0.0),
        Vec2::new(0.0, size.y),
        size,
    ] {
        let ray = camera.viewport_to_world(&global, corner).ok()?;
        // A corner above the horizon has no floor: as far as anything shows.
        let along = if ray.direction.y < -1.0e-3 {
            (-ray.origin.y / ray.direction.y).min(far)
        } else {
            far
        };
        let point = ray.origin + *ray.direction * along;
        let point = Vec2::new(point.x, point.z);
        low = low.min(point);
        high = high.max(point);
    }
    let (min_x, min_y, width, height) = bounds;
    let level_low = Vec2::new(min_x as f32 - 1.0, min_y as f32 - 1.0) * CELL_SIZE;
    let level_high = Vec2::new((min_x + width) as f32, (min_y + height) as f32) * CELL_SIZE;
    let min = low.clamp(level_low, level_high);
    let max = high.clamp(level_low, level_high).max(min + CELL_SIZE);
    Some(Area { min, max })
}

/// A point in cells, whose centres are at whole numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}

/// A bar of wall in cells: its smallest and largest corners.
type Bar = (Vec2d, Vec2d);

/// The walls of a level, ready for casting rays, with the corners worth casting
/// them at.
#[derive(Default)]
pub struct WallMap {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    bars: Vec<Option<[Bar; 2]>>,
    corners: Vec<Vec2d>,
}

impl WallMap {
    pub fn of(level: &Level) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        let to_cells = |units: i64| units as f64 / CELL_UNITS as f64;
        let mut bars = Vec::with_capacity((width * height) as usize);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                bars.push(level.wall_bars(x, y).map(|found| {
                    found.map(|(low, high)| {
                        (
                            Vec2d {
                                x: to_cells(low.0),
                                y: to_cells(low.1),
                            },
                            Vec2d {
                                x: to_cells(high.0),
                                y: to_cells(high.1),
                            },
                        )
                    })
                }));
            }
        }
        let mut map = Self {
            min_x,
            min_y,
            width,
            height,
            bars,
            corners: Vec::new(),
        };
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                let Some(found) = map.bars_at(x, y).copied() else {
                    continue;
                };
                for (low, high) in found {
                    for corner in [
                        low,
                        high,
                        Vec2d {
                            x: low.x,
                            y: high.y,
                        },
                        Vec2d {
                            x: high.x,
                            y: low.y,
                        },
                    ] {
                        if !map.corner_is_hidden(corner, (x, y)) {
                            map.corners.push(corner);
                        }
                    }
                }
            }
        }
        map
    }

    fn bars_at(&self, x: i64, y: i64) -> Option<&[Bar; 2]> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        if !(0..self.width).contains(&col) || !(0..self.height).contains(&row) {
            return None;
        }
        self.bars[(row * self.width + col) as usize].as_ref()
    }

    /// Whether a bar's corner is no corner of the wall as a whole: inside its
    /// own cell's other bar, or where the wall carries on into the next cell.
    fn corner_is_hidden(&self, corner: Vec2d, cell: (i64, i64)) -> bool {
        for y in cell.1 - 1..=cell.1 + 1 {
            for x in cell.0 - 1..=cell.0 + 1 {
                let Some(found) = self.bars_at(x, y) else {
                    continue;
                };
                let own = (x, y) == cell;
                for (low, high) in found {
                    let inside = if own {
                        low.x < corner.x
                            && corner.x < high.x
                            && low.y < corner.y
                            && corner.y < high.y
                    } else {
                        low.x <= corner.x
                            && corner.x <= high.x
                            && low.y <= corner.y
                            && corner.y <= high.y
                    };
                    if inside {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// What can be seen from a point: the ends of its rays, going round.
    pub fn outline(&self, from: Vec2d) -> Vec<Vec2d> {
        let mut angles = Vec::with_capacity(self.corners.len() * 2 + EVEN_RAYS);
        for i in 0..EVEN_RAYS {
            angles.push(i as f64 / EVEN_RAYS as f64 * TAU - TAU / 2.0);
        }
        for corner in &self.corners {
            let angle = (corner.y - from.y).atan2(corner.x - from.x);
            angles.push(angle - BESIDE_CORNER);
            angles.push(angle + BESIDE_CORNER);
        }
        angles.sort_by(f64::total_cmp);
        angles
            .into_iter()
            .map(|angle| self.ray_end(from, angle))
            .collect()
    }

    /// Where a ray stops: the first wall, or just past the level's edge. Goes
    /// through the cells it crosses in turn.
    fn ray_end(&self, from: Vec2d, angle: f64) -> Vec2d {
        let (dx, dy) = (angle.cos(), angle.sin());
        let at = |along: f64| Vec2d {
            x: from.x + dx * along,
            y: from.y + dy * along,
        };
        let (mut x, mut y) = ((from.x + 0.5).floor() as i64, (from.y + 0.5).floor() as i64);
        let (step_x, step_y) = (if dx > 0.0 { 1 } else { -1 }, if dy > 0.0 { 1 } else { -1 });
        // How far along the ray the next cell edge is, each way.
        let edge = |start: f64, cell: i64, step: i64, by: f64| {
            if by == 0.0 {
                f64::INFINITY
            } else {
                (cell as f64 + step as f64 * 0.5 - start) / by
            }
        };
        let mut next_x = edge(from.x, x, step_x, dx);
        let mut next_y = edge(from.y, y, step_y, dy);
        let (cross_x, cross_y) = (1.0 / dx.abs(), 1.0 / dy.abs());
        let (right, bottom) = (self.min_x + self.width, self.min_y + self.height);
        let mut along = 0.0;
        loop {
            if x < self.min_x || y < self.min_y || x >= right || y >= bottom {
                return at(along);
            }
            if let Some(found) = self.bars_at(x, y) {
                let hit = found
                    .iter()
                    .filter_map(|bar| ray_hits_bar(from, (dx, dy), bar))
                    .min_by(f64::total_cmp);
                if let Some(hit) = hit {
                    return at(hit);
                }
            }
            if next_x < next_y {
                along = next_x;
                next_x += cross_x;
                x += step_x;
            } else {
                along = next_y;
                next_y += cross_y;
                y += step_y;
            }
        }
    }
}

/// How far along a ray it first meets a bar, if it does. A ray starting inside
/// a bar passes through it: a viewer can't be blinded by the wall it is cutting
/// the corner of.
fn ray_hits_bar(from: Vec2d, (dx, dy): (f64, f64), (low, high): &Bar) -> Option<f64> {
    let slab = |start: f64, by: f64, low: f64, high: f64| {
        if by == 0.0 {
            (low..=high)
                .contains(&start)
                .then_some((f64::NEG_INFINITY, f64::INFINITY))
        } else {
            let (a, b) = ((low - start) / by, (high - start) / by);
            Some((a.min(b), a.max(b)))
        }
    };
    let (x_in, x_out) = slab(from.x, dx, low.x, high.x)?;
    let (y_in, y_out) = slab(from.y, dy, low.y, high.y)?;
    let (enter, leave) = (x_in.max(y_in), x_out.min(y_out));
    (enter <= leave && enter > 0.0).then_some(enter)
}

/// Counts every open cell an outline reaches into as seen, row by row: the part
/// of each triangle of its fan in a row is a span of cells.
fn touch_cells(level: &Level, from: Vec2d, outline: &[Vec2d], touched: &mut Sight) {
    for i in 0..outline.len() {
        let triangle = [from, outline[i], outline[(i + 1) % outline.len()]];
        let lowest = triangle.iter().map(|p| p.y).fold(f64::MAX, f64::min);
        let highest = triangle.iter().map(|p| p.y).fold(f64::MIN, f64::max);
        for row in (lowest + 0.5).floor() as i64..=(highest + 0.5).floor() as i64 {
            let Some((left, right)) = span_in_row(&triangle, row) else {
                continue;
            };
            for x in (left + 0.5).floor() as i64..=(right + 0.5).floor() as i64 {
                if level.is_open(x, row) && !touched.is_seen(x, row) {
                    touched.reveal(level, x, row);
                }
            }
        }
    }
}

/// Counts the cells in `from` as seen in `into` too.
pub fn add_cells(level: &Level, from: &Sight, into: &mut Sight) {
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if from.is_seen(x, y) {
                into.insert(x, y);
            }
        }
    }
}

/// Counts the cells an outline's edge goes through, and those beside them.
fn cross_cells(outline: &[Vec2d], crossed: &mut Sight) {
    for i in 0..outline.len() {
        let (a, b) = (outline[i], outline[(i + 1) % outline.len()]);
        let edge = [a, b, b];
        for row in (a.y.min(b.y) + 0.5).floor() as i64..=(a.y.max(b.y) + 0.5).floor() as i64 {
            let Some((left, right)) = span_in_row(&edge, row) else {
                continue;
            };
            for x in (left + 0.5).floor() as i64 - 1..=(right + 0.5).floor() as i64 + 1 {
                for y in row - 1..=row + 1 {
                    crossed.insert(x, y);
                }
            }
        }
    }
}

/// The smallest and largest `x` of a triangle within a row of cells.
fn span_in_row(triangle: &[Vec2d; 3], row: i64) -> Option<(f64, f64)> {
    let (top, bottom) = (row as f64 - 0.5, row as f64 + 0.5);
    let mut span: Option<(f64, f64)> = None;
    let mut add = |x: f64| {
        span = Some(span.map_or((x, x), |(a, b)| (a.min(x), b.max(x))));
    };
    for i in 0..3 {
        let (a, b) = (triangle[i], triangle[(i + 1) % 3]);
        if (top..=bottom).contains(&a.y) {
            add(a.x);
        }
        for y in [top, bottom] {
            if (a.y < y) != (b.y < y) {
                add(a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y));
            }
        }
    }
    span
}

/// Adds the triangles from a viewer to its outline, on the floor.
fn add_fan(triangles: &mut Vec<[f32; 3]>, from: Vec2d, outline: &[Vec2d]) {
    let on_floor = |p: Vec2d| [p.x as f32 * CELL_SIZE, 0.0, p.y as f32 * CELL_SIZE];
    for i in 0..outline.len() {
        triangles.push(on_floor(from));
        triangles.push(on_floor(outline[i]));
        triangles.push(on_floor(outline[(i + 1) % outline.len()]));
    }
}

fn fan_mesh(positions: Vec<[f32; 3]>) -> Mesh {
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32((0..count as u32).collect()))
}

#[cfg(test)]
mod tests;
