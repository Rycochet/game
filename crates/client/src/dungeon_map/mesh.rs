//! Builds the dungeon's meshes from what is seen and lit: floors and walls as
//! textured quads whose vertex colours carry the tile colour and the light.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use shared::dungeon::{Level, Sight};

use super::{AMBIENT_LIGHT, CELL_SIZE, FLOOR_COLOUR, WALL_COLOUR, WALL_HEIGHT};

/// The four sides of a cell, as grid steps (`y` south).
const SIDES: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// How much light reaches each cell of a level, as linear RGB from 0 (only
/// the dull ambient light) to 1 (fully lit).
pub struct LightGrid {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    light: Vec<Vec3>,
}

impl LightGrid {
    pub fn new(level: &Level) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        Self {
            min_x,
            min_y,
            width,
            height,
            light: vec![Vec3::ZERO; (width * height) as usize],
        }
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        ((0..self.width).contains(&col) && (0..self.height).contains(&row))
            .then_some((row * self.width + col) as usize)
    }

    /// The light at a cell (none outside the level).
    pub fn at(&self, x: i64, y: i64) -> Vec3 {
        self.index(x, y).map_or(Vec3::ZERO, |i| self.light[i])
    }

    /// Adds a light at `(x, y)` reaching `radius` cells. Walls block it (the
    /// same rule as sight, from `shared`); it fades slowly at first, then
    /// faster, to nothing just beyond the radius.
    pub fn add_light(&mut self, level: &Level, (x, y): (i64, i64), radius: u32, colour: Vec3) {
        let mut reached = Sight::new(level);
        reached.add_view(level, x, y, Some(radius));
        let reach = radius as f32 + 1.0;
        let r = radius as i64;
        for cy in y - r..=y + r {
            for cx in x - r..=x + r {
                if !reached.is_seen(cx, cy) {
                    continue;
                }
                let distance = Vec2::new((cx - x) as f32, (cy - y) as f32).length();
                let strength = 1.0 - (distance / reach).powi(2);
                if let Some(i) = self.index(cx, cy) {
                    self.light[i] = (self.light[i] + colour * strength).min(Vec3::ONE);
                }
            }
        }
    }

    /// The light at a grid corner (between four cells; corner `(x, y)` is the
    /// north-west corner of cell `(x, y)`): the average over the open cells
    /// around it, so light fades smoothly across floors without leaking out
    /// of walls. `None` if no open cell touches it.
    fn corner(&self, level: &Level, x: i64, y: i64) -> Option<Vec3> {
        let mut total = Vec3::ZERO;
        let mut count = 0.0;
        for (cx, cy) in [(x - 1, y - 1), (x, y - 1), (x - 1, y), (x, y)] {
            if level.is_open(cx, cy) {
                total += self.at(cx, cy);
                count += 1.0;
            }
        }
        (count > 0.0).then(|| total / count)
    }
}

/// What a viewer (the player, or anything sharing its sight) sees from its
/// cell.
pub struct View {
    pub cell: (i64, i64),
    pub sight: Sight,
}

/// A wall side drawn solid, kept for click picking: its centre and the way
/// it faces.
#[derive(Clone, Copy, Debug)]
pub struct WallFace {
    pub centre: Vec3,
    pub normal: Vec3,
}

/// The two meshes the dungeon is drawn with.
pub struct DungeonMeshes {
    /// Floors and wall sides facing the camera.
    pub solid: Mesh,
    /// Wall sides facing away from the camera (the wall stands between the
    /// camera and what is seen), drawn half see-through.
    pub see_through: Mesh,
    /// Whether each mesh has anything in it.
    pub has_solid: bool,
    pub has_see_through: bool,
    /// The solid wall sides, which block clicks.
    pub solid_faces: Vec<WallFace>,
}

/// Builds the meshes for everything in `sight` (all the views combined), lit
/// by `light`. `camera` is the camera's position.
pub fn build(
    level: &Level,
    views: &[View],
    sight: &Sight,
    light: &LightGrid,
    camera: Vec3,
) -> DungeonMeshes {
    let mut solid = Builder::default();
    let mut see_through = Builder::default();
    let mut solid_faces = Vec::new();
    let (min_x, min_y, width, height) = level.bounds();

    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if !sight.is_seen(x, y) {
                continue;
            }
            let centre = Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE);
            if level.is_open(x, y) {
                solid.face(centre, Vec3::Y, CELL_SIZE / 2.0, FLOOR_COLOUR, |p| {
                    corner_light(level, light, p).unwrap_or(light.at(x, y))
                });
                continue;
            }

            // Only the sides someone sees; never the top.
            for (dx, dy) in visible_sides(level, views, (x, y)) {
                let normal = Vec3::new(dx as f32, 0.0, dy as f32);
                let middle = centre + normal * (CELL_SIZE / 2.0) + Vec3::Y * (WALL_HEIGHT / 2.0);
                let builder = if faces_camera(middle, normal, camera) {
                    solid_faces.push(WallFace {
                        centre: middle,
                        normal,
                    });
                    &mut solid
                } else {
                    &mut see_through
                };
                let (nx, ny) = (x + dx, y + dy);
                builder.face(middle, normal, CELL_SIZE / 2.0, WALL_COLOUR, |p| {
                    corner_light(level, light, p).unwrap_or(light.at(nx, ny))
                });
            }
        }
    }

    DungeonMeshes {
        has_solid: !solid.indices.is_empty(),
        has_see_through: !see_through.indices.is_empty(),
        solid: solid.into_mesh(),
        see_through: see_through.into_mesh(),
        solid_faces,
    }
}

/// The light at the grid corner nearest a vertex's (x, z) position.
fn corner_light(level: &Level, light: &LightGrid, position: Vec3) -> Option<Vec3> {
    let x = (position.x / CELL_SIZE + 0.5).round() as i64;
    let y = (position.z / CELL_SIZE + 0.5).round() as i64;
    light.corner(level, x, y)
}

/// The sides of the wall at `wall` that someone sees, as grid steps
/// outwards. A viewer sees a side when it sees the wall and the open cell in
/// front of that side, and stands on that side of the wall, so one viewer
/// sees at most two sides.
pub fn visible_sides(level: &Level, views: &[View], (x, y): (i64, i64)) -> Vec<(i64, i64)> {
    SIDES
        .into_iter()
        .filter(|&(dx, dy)| {
            let (nx, ny) = (x + dx, y + dy);
            level.is_open(nx, ny)
                && views.iter().any(|view| {
                    let (vx, vy) = view.cell;
                    view.sight.is_seen(x, y)
                        && view.sight.is_seen(nx, ny)
                        && dx * (vx - x) + dy * (vy - y) > 0
                })
        })
        .collect()
}

/// Whether a wall side (centred on `centre`, facing `normal`) faces the
/// camera at `camera`: the camera is in front of the side's plane. Measured
/// from the camera's real position (not just the way it looks), so a side
/// square-on to the view turns away only once the camera passes its plane.
pub fn faces_camera(centre: Vec3, normal: Vec3, camera: Vec3) -> bool {
    normal.dot(camera - centre) >= 0.0
}

/// Collects quads into one mesh.
#[derive(Default)]
struct Builder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colours: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl Builder {
    /// A square face centred on `centre`, facing `normal`, `half` units from
    /// its centre to each side, textured once. `light_at` gives each corner's
    /// light from its position; the vertex colour is `colour` lit by it on top
    /// of the ambient light.
    fn face(
        &mut self,
        centre: Vec3,
        normal: Vec3,
        half: f32,
        colour: Color,
        light_at: impl Fn(Vec3) -> Vec3,
    ) {
        // "Up" on the face: north on floors and tops, up on walls. "Right"
        // then makes the corners run anticlockwise seen from the front.
        let up = if normal == Vec3::Y {
            Vec3::NEG_Z
        } else {
            Vec3::Y
        };
        let right = up.cross(normal);
        let (right, up) = (right * half, up * half);
        let base = LinearRgba::from(colour);
        let base = Vec3::new(base.red, base.green, base.blue);
        let first = self.positions.len() as u32;
        let corners = [
            (centre - right - up, [0.0, 1.0]),
            (centre + right - up, [1.0, 1.0]),
            (centre + right + up, [1.0, 0.0]),
            (centre - right + up, [0.0, 0.0]),
        ];
        for (position, uv) in corners {
            let lit = AMBIENT_LIGHT + (Vec3::ONE - AMBIENT_LIGHT) * light_at(position);
            let shade = base * lit;
            self.positions.push(position.to_array());
            self.normals.push(normal.to_array());
            self.uvs.push(uv);
            self.colours.push([shade.x, shade.y, shade.z, 1.0]);
        }
        self.indices
            .extend([0, 1, 2, 0, 2, 3].map(|corner| first + corner));
    }

    fn into_mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colours)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What each viewer sees from its cell.
    fn views_from(level: &Level, cells: &[(i64, i64)]) -> Vec<View> {
        cells
            .iter()
            .map(|&(x, y)| {
                let mut sight = Sight::new(level);
                sight.add_view(level, x, y, None);
                View {
                    cell: (x, y),
                    sight,
                }
            })
            .collect()
    }

    #[test]
    fn only_sides_facing_a_viewer_are_seen() {
        let level = Level::test_room();
        let views = views_from(&level, &[(0, 0)]);
        // The north wall shows its south side; the short wall its east side.
        assert_eq!(visible_sides(&level, &views, (0, -4)), [(0, 1)]);
        assert_eq!(visible_sides(&level, &views, (-2, -1)), [(1, 0)]);
        // A corner shows nothing.
        assert!(visible_sides(&level, &views, (8, -4)).is_empty());
        // Diagonally off its end, two sides.
        let views = views_from(&level, &[(-1, -2)]);
        assert_eq!(visible_sides(&level, &views, (-2, -1)), [(0, -1), (1, 0)]);
        // A second viewer adds the sides it sees.
        let views = views_from(&level, &[(0, 0), (-4, 0)]);
        assert_eq!(visible_sides(&level, &views, (-2, 0)), [(1, 0), (-1, 0)]);
    }

    #[test]
    fn sides_face_the_camera_by_its_position() {
        // Camera south of the room, a little east of the middle.
        let camera = Vec3::new(20.0, 600.0, 900.0);
        let north_wall = Vec3::new(0.0, 12.0, -84.0);
        assert!(faces_camera(north_wall, Vec3::Z, camera));
        assert!(!faces_camera(-north_wall, Vec3::NEG_Z, camera));
        // A west-facing side east of the camera faces it, even though the
        // camera looks a little away from it...
        let east_wall = Vec3::new(180.0, 12.0, 0.0);
        assert!(faces_camera(east_wall, Vec3::NEG_X, camera));
        // ...until the camera passes its plane.
        assert!(!faces_camera(
            east_wall,
            Vec3::NEG_X,
            Vec3::new(200.0, 600.0, 900.0)
        ));
    }

    #[test]
    fn light_fades_and_stops_at_walls() {
        let level = Level::test_room();
        let mut light = LightGrid::new(&level);
        light.add_light(&level, (0, 0), 6, Vec3::ONE);
        assert_eq!(light.at(0, 0), Vec3::ONE);
        assert!(light.at(3, 0).x < light.at(1, 0).x);
        assert!(light.at(6, 0).x > 0.0);
        assert_eq!(light.at(7, 0), Vec3::ZERO);
        // Behind the short wall.
        assert_eq!(light.at(-3, 0), Vec3::ZERO);
        // Lights add up but never go past full.
        light.add_light(&level, (1, 0), 6, Vec3::ONE);
        assert_eq!(light.at(0, 0), Vec3::ONE);
    }

    #[test]
    fn only_what_is_seen_is_built() {
        let level = Level::test_room();
        let views = views_from(&level, &[(0, 0)]);
        let light = LightGrid::new(&level);
        let camera = Vec3::new(0.0, 600.0, 900.0);
        let meshes = build(&level, &views, &views[0].sight, &light, camera);
        assert!(meshes.has_solid);
        // The south wall's inner sides face away from the camera.
        assert!(meshes.has_see_through);
        assert!(!meshes.solid_faces.is_empty());

        let nothing = Sight::new(&level);
        let meshes = build(&level, &[], &nothing, &light, camera);
        assert!(!meshes.has_solid);
        assert!(!meshes.has_see_through);
    }
}
