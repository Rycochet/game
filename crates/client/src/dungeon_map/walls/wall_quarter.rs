//! Building one quarter of a wall, its foot, and wet feet.

use bevy::prelude::*;

use super::super::liquid::{self, Outline, outline_at, surfaces_near};
use super::super::mesh::Surface;
use super::{Layer, Layers, Patch, TOP_SHADE, Tile, flat_uv};

impl<'a> Tile<'a> {
    /// One quarter of a wall: floor, up to two faces, and its top. `reach` is
    /// how far the wall goes at its narrowest, as `[north, east, south, west]`;
    /// `turned_away` is from `Self::turned_away`.
    pub(super) fn build_wall_quarter(
        &self,
        layers: &mut Layers,
        quarter: (f32, f32),
        reach: [f32; 4],
        turned_away: [bool; 2],
    ) {
        let scene = self.scene;
        let in_sight = self.quarter_shows(scene.sight, quarter);
        // The wall is remembered, so a side not yet seen is drawn as memory
        // too: otherwise it is a hole whenever the camera is behind the side
        // that was seen. Quarters with nothing open round them are solid rock.
        if !in_sight && !self.quarter_open(quarter) {
            return;
        }
        let (east_west, north_south) = Self::quarter_sides(quarter);
        let see_through = |facing: usize| turned_away[self.stretch(facing)];
        let faces = [
            // Facing north or south, along the part of the wall running
            // east-west, then the other way.
            self.wall_face(
                layers,
                quarter,
                in_sight,
                north_south,
                east_west,
                reach,
                see_through(north_south),
            ),
            self.wall_face(
                layers,
                quarter,
                in_sight,
                east_west,
                north_south,
                reach,
                see_through(east_west),
            ),
        ];
        // The mask decides what shows of a floor in sight: lit inside its edge
        // and grey outside (`copy_as_memory`). Memory alone would leave a hole
        // inside the edge. A top isn't cut, so one whose only faces are
        // remembered is remembered too, to match them.
        let live = faces.contains(&Some(Layer::Solid)) || faces.contains(&Some(Layer::SeeThrough));
        let remembered = !in_sight || (!live && faces.contains(&Some(Layer::Memory)));
        let foot = self.wall_foot((east_west, north_south), reach);
        let floor_layer = if in_sight {
            Layer::Solid
        } else {
            Layer::Memory
        };
        match outline_at(scene.level, (self.x, self.y), quarter) {
            Some(outline) => self.wet_quarter(layers, floor_layer, quarter, outline, &foot),
            None => self.floor_quarter(layers, floor_layer, quarter, &foot, &foot),
        }
        let top_layer = if remembered {
            Layer::Memory
        } else if !faces.contains(&Some(Layer::Solid))
            && (see_through(north_south) || see_through(east_west))
        {
            Layer::SeeThrough
        } else {
            Layer::Solid
        };
        self.wall_top(layers, top_layer, quarter, (east_west, north_south), reach);
    }

    /// The ground under a wall's foot in a quarter, shaded like the wall's top.
    pub(super) fn fill_foot(&self, layers: &mut Layers, quarter: (f32, f32), reach: [f32; 4]) {
        let (sx, sz) = quarter;
        for patch in self.wall_foot(Self::quarter_sides(quarter), reach) {
            let point = |east: f32, south: f32| {
                let (e, s) = (sx * east, sz * south);
                (self.centre + Vec3::new(e, 0.0, s), flat_uv(e, s), TOP_SHADE)
            };
            let corners = [
                point(patch.east.0, patch.south.0),
                point(patch.east.1, patch.south.0),
                point(patch.east.1, patch.south.1),
                point(patch.east.0, patch.south.1),
            ];
            self.add(layers, Layer::Solid, Surface::Wall, Vec3::Y, corners);
        }
    }

    /// A quarter of a wall's cell that liquid reaches: the liquid up to the
    /// wall, with the floor beside it stopping at the wall's foot and fading
    /// from it as usual. (The liquid runs on under the wall, out of sight.)
    fn wet_quarter(
        &self,
        layers: &mut Layers,
        layer: Layer,
        quarter: (f32, f32),
        outline: Outline,
        foot: &[Patch],
    ) {
        let (liquid, floor) = surfaces_near(self.scene.level, (self.x, self.y));
        let straight = outline.straight_shore();
        if let Some((east, south)) = straight {
            let mut dry = foot.to_vec();
            dry.push(Patch {
                east: (0.0, east),
                south: (0.0, south),
            });
            self.floor_quarter(layers, layer, quarter, &dry, foot);
        }
        let drawn = liquid::Quarter {
            layer,
            centre: self.centre,
            quarter,
            outline,
            liquid,
            floor,
            with_floor: straight.is_none(),
            colour: &|point| self.colour(layer, point, 1.0),
        };
        drawn.build_in_wall(layers);
        drawn.close_behind_wall(layers, self.scene.level, (self.x, self.y));
    }
}
