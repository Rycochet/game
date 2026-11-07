//! Moving and sizing the drawn tree as the page is moved and zoomed: the circles
//! grow with the zoom as much as the room between them does, so they never touch;
//! the skill hovered over, or else the chosen one, gets an outline.

use bevy::prelude::*;

use super::build::{SkillLine, SkillNode};
use super::view::SkillsView;

/// The outline round the skill hovered over or chosen.
pub(super) const PICKED_OUTLINE: Color = Color::srgb(0.85, 0.55, 0.1);

#[allow(clippy::type_complexity)]
pub(super) fn place_tree(
    view: Res<SkillsView>,
    mut nodes: Query<(&SkillNode, &mut Node, &mut Outline), Without<SkillLine>>,
    mut lines: Query<(&SkillLine, &mut Node, &mut UiTransform), Without<SkillNode>>,
) {
    if !view.shown {
        return;
    }
    let picked = view.hovered.or(view.chosen);
    for (skill, mut node, mut outline) in &mut nodes {
        let size = skill.size * view.zoom;
        let at = view.on_page(skill.at) - Vec2::splat(size / 2.0);
        node.left = px(at.x);
        node.top = px(at.y);
        node.width = px(size);
        node.height = px(size);
        let shown = skill.id.is_some() && skill.id == picked;
        outline.color = if shown { PICKED_OUTLINE } else { Color::NONE };
    }
    for (line, mut node, mut transform) in &mut lines {
        let (from, to) = (view.on_page(line.from), view.on_page(line.to));
        let length = from.distance(to);
        let middle = (from + to) / 2.0;
        node.width = px(length);
        node.left = px(middle.x - length / 2.0);
        node.top = px(middle.y - 1.0);
        transform.rotation = Rot2::radians((to - from).to_angle());
    }
}
