//! `SkillsView`: how the page is moved and zoomed, and which skill is chosen or
//! hovered over. Places are in interface pixels from the middle of the page.

use bevy::prelude::*;
use shared::skill::SkillId;

use super::layout::SIZES;

pub(super) const MIN_ZOOM: f32 = 0.1;
/// How far in it may zoom: until the smallest level's circle is this many times
/// the size the middle was when the tree was fitted (owner, 2026-11-07).
const ZOOM_IN_TO: f32 = 2.0;
/// What each zoom button, wheel notch or key press multiplies the zoom by.
pub(super) const ZOOM_STEP: f32 = 1.2;
/// Room left round the tree when it is fitted to the page, as a share.
const FIT_MARGIN: f32 = 0.85;

#[derive(Resource, Debug)]
pub(super) struct SkillsView {
    /// Where the character is, from the middle of the page.
    pub pan: Vec2,
    pub zoom: f32,
    pub chosen: Option<SkillId>,
    /// The skill under the pointer, if any: it shows over the chosen one.
    pub hovered: Option<SkillId>,
    /// How far in it may zoom (`ZOOM_IN_TO`), set on fitting.
    pub max_zoom: f32,
    /// Where the mouse was pressed on the page, and where it last was, while held.
    pub pressed_at: Option<Vec2>,
    pub drag_from: Option<Vec2>,
    /// Fit the tree to the page once it has been laid out (just opened).
    pub fit_wanted: bool,
}

impl Default for SkillsView {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            chosen: None,
            hovered: None,
            max_zoom: ZOOM_IN_TO * SIZES[0] / SIZES[SIZES.len() - 1],
            pressed_at: None,
            drag_from: None,
            fit_wanted: true,
        }
    }
}

impl SkillsView {
    /// Where a place on the tree (at zoom 1, from the character) is on the page.
    pub fn on_page(&self, at: Vec2) -> Vec2 {
        self.pan + at * self.zoom
    }

    /// Zooms by `factor` keeping what is at `about` (from the middle of the page)
    /// where it is.
    pub fn zoom_about(&mut self, factor: f32, about: Vec2) {
        let zoom = (self.zoom * factor).clamp(MIN_ZOOM, self.max_zoom);
        let on_tree = (about - self.pan) / self.zoom;
        self.zoom = zoom;
        self.pan = about - on_tree * zoom;
    }

    /// The tree's extent (`low` to `high` at zoom 1, circles included) in the middle
    /// of a page `size` across, as big as fits but never past zoom 1; and zooming in
    /// allowed until the smallest circles are twice the middle's size now.
    pub fn fit(&mut self, size: Vec2, (low, high): (Vec2, Vec2)) {
        let extent = (high - low).max(Vec2::ONE);
        let zoom = (size * FIT_MARGIN / extent).min_element();
        self.zoom = zoom.clamp(MIN_ZOOM, 1.0);
        self.pan = -(low + high) / 2.0 * self.zoom;
        self.max_zoom = ZOOM_IN_TO * SIZES[0] * self.zoom / SIZES[SIZES.len() - 1];
    }
}

#[cfg(test)]
mod tests;
