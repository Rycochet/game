//! `object`: one thing shown, named by its `for`. So far `type="item"`: the
//! item with that id, drawn as in a grid (its picture or cells, its quality's
//! edge, glow, set edge and count) and named when the pointer is on it. What
//! the client doesn't know yet is asked of the server, and shows as an empty
//! cell until it answers.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::Item;

use super::parts::Element;
use super::tag::Tag;
use crate::common::{KnownItem, KnownItems};
use crate::widgets::{CELL, GRID_COLOUR, GRID_LINE, ItemLook, ItemName, spawn_item_cells};

/// On an `object`'s element: what it last drew.
#[derive(Component, Default)]
pub(super) struct ObjectShown(Option<Drawn>);

/// What an `object` shows of its item.
#[derive(Clone, Debug, PartialEq)]
enum Drawn {
    /// Nothing named, or not an item: no room taken.
    Nothing,
    /// Asked after, with no answer yet.
    Loading,
    /// The server knows nothing of it.
    Unknown,
    Item(Box<Item>),
}

/// What to show for an `object` with these attributes, asking after its item
/// if it names one.
fn drawn(element: &Element, known: &mut KnownItems, now: f64) -> Drawn {
    if element.attrs.text("type") != Some("item") {
        return Drawn::Nothing;
    }
    let Some(id) = element.attrs.text("for").and_then(Id::parse) else {
        return Drawn::Nothing;
    };
    known.want(id, now);
    match known.get(id) {
        Some(KnownItem::Found(item, _)) => Drawn::Item(item.clone()),
        Some(KnownItem::Unknown(_)) => Drawn::Unknown,
        Some(KnownItem::Asked(_)) | None => Drawn::Loading,
    }
}

/// Draws each `object` again when what it shows changes.
pub(super) fn show_objects(
    mut commands: Commands,
    mut known: ResMut<KnownItems>,
    time: Res<Time<Real>>,
    mut objects: Query<(Entity, &Element, &mut ObjectShown)>,
) {
    let now = time.elapsed_secs_f64();
    for (entity, element, mut shown) in &mut objects {
        if element.tag != Tag::Object {
            continue;
        }
        // (Without marking the items as changed when nothing is asked.)
        let now_drawn = drawn(element, known.bypass_change_detection(), now);
        if shown.0.as_ref() == Some(&now_drawn) {
            continue;
        }
        let mut object = commands.entity(entity);
        object.despawn_related::<Children>();
        match &now_drawn {
            Drawn::Nothing => {}
            Drawn::Loading | Drawn::Unknown => {
                let words = match now_drawn {
                    Drawn::Unknown => "?",
                    _ => "",
                };
                object.with_children(|object| {
                    object
                        .spawn((placeholder(), BackgroundColor(GRID_COLOUR)))
                        .with_child(Text::new(words));
                });
            }
            Drawn::Item(item) => {
                let shape = item.shape();
                let room = Node {
                    width: px(shape.width as f32 * CELL),
                    height: px(shape.height as f32 * CELL),
                    flex_shrink: 0.0,
                    ..default()
                };
                let name = ItemName {
                    words: item.name(),
                    quality: item.quality,
                    set: item.set,
                };
                let look = ItemLook::of(item);
                object.with_children(|object| {
                    object.spawn(room).with_children(|room| {
                        spawn_item_cells(room, look, shape, (0.0, 0.0), false, |_, _| name.clone());
                    });
                });
            }
        }
        shown.0 = Some(now_drawn);
    }
}

/// An empty cell, for an item not known yet.
fn placeholder() -> impl Bundle {
    (
        Node {
            width: px(CELL),
            height: px(CELL),
            flex_shrink: 0.0,
            border: UiRect::all(px(1)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::from(GRID_LINE),
    )
}

#[cfg(test)]
mod tests;
