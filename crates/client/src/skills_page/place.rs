//! Moving the drawn tree as the page is moved and zoomed: the nodes spread apart
//! as it zooms in rather than growing, so writing stays sharp; deeper names hide
//! when zoomed far out; the chosen skill's edge is picked out.

use bevy::prelude::*;

use super::build::{CHOSEN_EDGE, NodeLabel, SkillLine, SkillNode};
use super::view::SkillsView;
use crate::common::SkillsPageOpen;
use crate::widgets::INK;

#[allow(clippy::type_complexity)]
pub(super) fn place_tree(
    open: Res<SkillsPageOpen>,
    view: Res<SkillsView>,
    mut nodes: Query<(&SkillNode, &mut Node, &mut BorderColor), Without<SkillLine>>,
    mut lines: Query<(&SkillLine, &mut Node, &mut UiTransform), Without<SkillNode>>,
    mut labels: Query<(&NodeLabel, &mut Visibility)>,
) {
    if !open.0 {
        return;
    }
    for (skill, mut node, mut edge) in &mut nodes {
        let at = view.on_page(skill.at) - Vec2::splat(skill.size / 2.0);
        node.left = px(at.x);
        node.top = px(at.y);
        let chosen = skill.id.is_some() && skill.id == view.chosen;
        *edge = BorderColor::all(if chosen { CHOSEN_EDGE } else { INK });
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
    for (label, mut visibility) in &mut labels {
        *visibility = if view.zoom >= label.hidden_below {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
