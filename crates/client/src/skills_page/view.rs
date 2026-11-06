//! `SkillsView`: whether the page is open, how it is moved and zoomed, and which
//! skill is chosen. Places are in interface pixels from the middle of the page.

use bevy::prelude::*;
use shared::skill::SkillId;

pub(super) const MIN_ZOOM: f32 = 0.3;
pub(super) const MAX_ZOOM: f32 = 3.0;
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
        let zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let on_tree = (about - self.pan) / self.zoom;
        self.zoom = zoom;
        self.pan = about - on_tree * zoom;
    }

    /// The character in the middle, zoomed so everything as far as `reach` from
    /// it fits a page `size` across, but never past zoom 1.
    pub fn fit(&mut self, size: Vec2, reach: f32) {
        self.pan = Vec2::ZERO;
        let room = size.min_element() / 2.0 * FIT_MARGIN;
        self.zoom = if reach <= 0.0 {
            1.0
        } else {
            (room / reach).clamp(MIN_ZOOM, 1.0)
        };
    }
}

#[cfg(test)]
mod tests;
