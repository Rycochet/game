//! The box naming the item under the pointer.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::set_edge::{SetEdge, set_edge};
use super::super::spawn_tooltip;
use super::{
    ItemName, ItemTip, ItemTooltip, NAME_EDGE, NameBox, NameEdge, NameText, TOOLTIP_OFFSET,
};
use crate::common::{NO_QUALITY_NAME, interface_area, quality_colour};
use crate::dom::Hit;

/// The widest a tooltip grows before its words wrap, in the interface's pixels.
const TIP_WIDTH: f32 = 420.0;

pub(in crate::widgets) fn spawn_name_box(mut commands: Commands) {
    commands
        .spawn((
            NameBox,
            Node {
                position_type: PositionType::Absolute,
                // Long words (a setting's) wrap; names fit.
                max_width: px(TIP_WIDTH),
                ..default()
            },
            // In front of everything, full-screen layers included (Settings).
            GlobalZIndex(i32::MAX),
            Visibility::Hidden,
        ))
        .with_children(|holder| {
            spawn_tooltip(holder, NameText, "");
            holder.spawn((
                NameEdge,
                set_edge(None, UiRect::all(px(NAME_EDGE)), 0.0),
                Visibility::Hidden,
            ));
        });
}

/// Asks for the name of the `ItemName` in front at the pointer (`dom::Hit`:
/// the thing in front, or the nearest thing it is in), unless a name has been
/// asked for already.
pub(in crate::widgets) fn name_item_at_pointer(
    hit: Res<Hit>,
    names: Query<&ItemName>,
    mut tooltip: ResMut<ItemTooltip>,
) {
    let Some(at) = hit.at.filter(|_| tooltip.0.is_none()) else {
        return;
    };
    if let Some(name) = hit.first(|entity| names.get(entity).ok()) {
        tooltip.0 = Some(ItemTip {
            words: name.words.clone(),
            quality: name.quality,
            set: name.set,
            at: at + TOOLTIP_OFFSET,
        });
    }
}

#[allow(clippy::type_complexity)]
/// Shows the name asked for this frame, on a dark box so every colour can be read.
pub(in crate::widgets) fn show_item_tooltip(
    mut tooltip: ResMut<ItemTooltip>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    name_box: Single<(&mut Node, &mut Visibility, &ComputedNode), With<NameBox>>,
    text: Single<(&mut Text, &mut TextColor), With<NameText>>,
    edge: Single<(&mut SetEdge, &mut Visibility), (With<NameEdge>, Without<NameBox>)>,
) {
    let (mut node, mut visibility, computed) = name_box.into_inner();
    let Some(tip) = tooltip.0.take() else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let (mut edge, mut edge_visible) = edge.into_inner();
    edge.set_if_neq(SetEdge(tip.quality));
    edge_visible.set_if_neq(if tip.set {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    let (mut text, mut colour) = text.into_inner();
    if text.0 != tip.words {
        text.0 = tip.words;
    }
    colour.set_if_neq(TextColor(
        tip.quality.map_or(NO_QUALITY_NAME, quality_colour),
    ));
    // Kept on the screen, by the size it last had.
    let size = computed.size() * computed.inverse_scale_factor;
    let at = tip
        .at
        .min(interface_area(&window, &ui_scale) - size)
        .max(Vec2::ZERO);
    node.left = px(at.x);
    node.top = px(at.y);
}
