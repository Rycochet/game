//! Showing the draft: the triangles' picture, their corners' names, and what the
//! attributes come to.

use bevy::prelude::*;
use shared::sheet::{POINTS_TOTAL, Row};

use super::draft::Draft;
use super::geometry::{centre, corners};
use super::picture::{Drawn, paint};
use super::screen::{CHOOSER_SIDE, Chooser, CornerLabel};
use crate::dom::Dom;

/// How far above the top corner, or below a lower one, a corner's name sits, in
/// the chooser's units. Straight up and down, as the triangles stay upright, so
/// the two lower corners meeting between triangles keep their names apart.
const LABEL_OUT: f32 = 0.028;
/// Half a short name's size, in interface pixels, to put its middle there.
const LABEL_HALF: Vec2 = Vec2::new(12.0, 8.0);

/// Paints the picture again when the draft changes.
pub(super) fn draw_triangles(
    draft: Res<Draft>,
    chooser: Single<&Chooser>,
    mut images: ResMut<Assets<Image>>,
) {
    if !draft.is_changed() {
        return;
    }
    let attributes = draft.attributes();
    let bonus = draft.kind.def().bonus();
    let drawn = Drawn {
        build: &draft.build,
        attributes: &attributes,
        bonus: &bonus,
        chosen: draft.chosen,
    };
    if let Some(mut image) = images.get_mut(&chooser.0) {
        image.data = Some(paint(&drawn));
    }
}

/// Puts each corner's name just past its corner, as the triangles turn.
pub(super) fn place_labels(draft: Res<Draft>, mut labels: Query<(&CornerLabel, &mut Node)>) {
    if !draft.is_changed() {
        return;
    }
    for (label, mut node) in &mut labels {
        let Some(row) = Row::ALL
            .into_iter()
            .find(|row| row.attributes().contains(&label.0))
        else {
            continue;
        };
        let column = row
            .attributes()
            .iter()
            .position(|&which| which == label.0)
            .unwrap_or(0);
        let middle = centre(row, draft.build.turn);
        let corner = corners(row, draft.build.turn)[column];
        let below = if corner.y > middle.y { 1.0 } else { -1.0 };
        let at = (corner + Vec2::new(0.0, LABEL_OUT * below)) * CHOOSER_SIDE - LABEL_HALF;
        node.left = px(at.x);
        node.top = px(at.y);
    }
}

/// The attributes as words: each row's, with its points.
pub(super) fn show_values(draft: Res<Draft>, mut dom: Dom) {
    if !draft.is_changed() {
        return;
    }
    let attributes = draft.attributes();
    let points = draft.build.points();
    let lines: Vec<String> = Row::ALL
        .into_iter()
        .map(|row| {
            let values: Vec<String> = row
                .attributes()
                .iter()
                .map(|&which| format!("{} {}", which.name(), attributes.get(which)))
                .collect();
            let share = points[row as usize] as f32 / (POINTS_TOTAL / 4) as f32;
            format!("{} ({share:.1}): {}", row.name(), values.join(", "))
        })
        .collect();
    dom.set_text("character_create/values", lines.join("\n"));
}
