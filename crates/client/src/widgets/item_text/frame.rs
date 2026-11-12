//! What shows an item in writing: a frame in its quality's colour, as round
//! items in grids, with the item inside. A text can't hold nodes (one with a
//! node child stops showing its writing), so the frame is a node beside the text
//! (a child of what holds it), placed over the room its item has in the writing.

use bevy::prelude::*;
use bevy::text::TextLayoutInfo;
use bevy::ui::UiGlobalTransform;

use super::inline::{GAP, InlineItem, inline_item};
use super::look::Shows;
use crate::common::{KnownItem, KnownItems};
use crate::widgets::{item_edge, item_edge_width};

/// How far a frame is moved down from its room, as a share of its height:
/// about the font's descender, as the room is a line tall (1.2 of the font's
/// size) standing on the baseline.
const DROP: f32 = 0.2 / 1.2;
/// And a little further, which looks better (owner).
const DROP_PIXELS: f32 = 2.0;

/// The frame of the item `item`.
#[derive(Component, Clone, Copy, Debug)]
pub(in crate::widgets) struct InlineFrame {
    item: Entity,
}

/// On an item that has its frame.
#[derive(Component)]
pub(in crate::widgets) struct Framed;

/// Spawns an item `side` across among a text's spans, with `extra` on it; its
/// frame follows.
pub(in crate::widgets) fn spawn_inline_item(
    spans: &mut ChildSpawnerCommands,
    id: shared::id::Id,
    side: f32,
    extra: impl Bundle,
) {
    spans.spawn((inline_item(id, side), extra));
}

/// Gives new items their frames, beside their text, and lets go of frames whose
/// items have gone.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn add_frames(
    mut commands: Commands,
    new: Query<(Entity, &ChildOf), (With<InlineItem>, Without<Framed>)>,
    parents: Query<&ChildOf, With<Node>>,
    frames: Query<(Entity, &InlineFrame)>,
    items: Query<(), With<InlineItem>>,
) {
    for (item, text) in &new {
        let Ok(holder) = parents.get(text.parent()) else {
            continue;
        };
        commands.entity(item).insert(Framed);
        commands
            .spawn((
                InlineFrame { item },
                Shows::Loading,
                ChildOf(holder.parent()),
                // Not what the pointer is on: the writing is, for naming it.
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BorderColor::all(Color::NONE),
                Visibility::Hidden,
            ))
            .with_children(|frame| Shows::Loading.spawn(frame));
    }
    for (frame, of) in &frames {
        if !items.contains(of.item) {
            commands.entity(frame).despawn();
        }
    }
}

/// The top left of what a node's absolute children are placed from, in physical
/// pixels: inside its border.
fn inside_corner(node: &ComputedNode, transform: &UiGlobalTransform) -> Vec2 {
    transform.translation - node.size() / 2.0 + node.border.min_inset
}

type Frames<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static InlineFrame,
        &'static ChildOf,
        &'static mut Node,
        &'static mut BorderColor,
        &'static mut Visibility,
        &'static mut Shows,
    ),
>;

/// Puts each frame over its item's room, as last laid out, and fills it with
/// what is known of the item.
pub(in crate::widgets) fn place_frames(
    mut commands: Commands,
    mut frames: Frames,
    texts: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform)>,
    holders: Query<(&ComputedNode, &UiGlobalTransform)>,
    items: Query<(&InlineItem, &ChildOf)>,
    known: Res<KnownItems>,
) {
    for (entity, frame, holder, mut node, mut colour, mut visibility, mut shows) in &mut frames {
        let placed = (|| {
            let (item, text) = items.get(frame.item).ok()?;
            let (layout, text, text_at) = texts.get(text.parent()).ok()?;
            let (_, _, rect) = layout
                .inline_boxes
                .iter()
                .find(|(e, ..)| *e == frame.item)?;
            let (holder, holder_at) = holders.get(holder.parent()).ok()?;
            let corner = text_at.translation - text.size() / 2.0;
            let offset = corner - inside_corner(holder, holder_at);
            let scale = text.inverse_scale_factor;
            Some(((offset + rect.min) * scale, rect.size() * scale, item.id))
        })();
        let Some((at, size, id)) = placed else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let known = known.get(id);
        let now = Shows::of(known);
        if *shows != now {
            commands.entity(entity).despawn_related::<Children>();
            commands
                .entity(entity)
                .with_children(|inside| now.spawn(inside));
            *shows = now;
        }
        let quality = match known {
            Some(KnownItem::Found(item, _)) => Some(item.quality),
            _ => None,
        };
        let edge = quality.map_or(0.0, item_edge_width);
        let wanted = Node {
            position_type: PositionType::Absolute,
            left: px(at.x + GAP),
            // Its room stands on the writing's baseline, as letters do: down a
            // descender's depth, it sits in the middle of the line.
            top: px(at.y + size.y * DROP + DROP_PIXELS),
            width: px(size.x - 2.0 * GAP),
            height: px(size.y),
            border: UiRect::all(px(edge)),
            ..default()
        };
        if *node != wanted {
            *node = wanted;
        }
        let edge_colour = quality.map_or(Color::NONE, item_edge);
        colour.set_if_neq(BorderColor::all(edge_colour));
        visibility.set_if_neq(Visibility::Inherited);
    }
}
