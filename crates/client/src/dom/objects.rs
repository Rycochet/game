//! `object`: one thing shown, named by its `for`. `type="item"`: the item with
//! that id, drawn as in a grid (its picture or cells, its quality's edge,
//! glow, set edge and count) and named when the pointer is on it. What the
//! client doesn't know yet is asked of the server, and shows as an empty cell
//! until it answers. `type="ability"`: the ability of that name
//! (`KnownAbilities`), as on the Abilities page.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::Item;

use super::object_ability::{AbilityCell, AbilityThing, KnownAbilities};
use super::parts::Element;
use super::tag::Tag;
use crate::common::{KnownItem, KnownItems};
use crate::widgets::{
    CELL, GRID_COLOUR, GRID_LINE, ItemLook, ItemName, spawn_ability, spawn_item_cells,
};

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
    Ability(Box<AbilityThing>),
}

/// What to show for an `object` with these attributes, asking after its item
/// if it names one.
fn drawn(element: &Element, known: &mut KnownItems, abilities: &KnownAbilities, now: f64) -> Drawn {
    let named = element.attrs.text("for");
    match element.attrs.text("type") {
        Some("item") => {}
        Some("ability") => {
            let ability = named.and_then(|name| abilities.get(name));
            return ability.map_or(Drawn::Nothing, |thing| {
                Drawn::Ability(Box::new(thing.clone()))
            });
        }
        _ => return Drawn::Nothing,
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
    abilities: Res<KnownAbilities>,
    time: Res<Time<Real>>,
    mut objects: Query<(Entity, &Element, &mut ObjectShown)>,
) {
    let now = time.elapsed_secs_f64();
    for (entity, element, mut shown) in &mut objects {
        if element.tag != Tag::Object {
            continue;
        }
        // (Without marking the items as changed when nothing is asked.)
        let now_drawn = drawn(element, known.bypass_change_detection(), &abilities, now);
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
            Drawn::Ability(thing) => {
                let cell = AbilityCell {
                    of: entity,
                    ability: thing.ability,
                };
                let look = thing.look.clone();
                object.with_children(|object| {
                    let frame = spawn_ability(object, look);
                    object.commands().entity(frame).insert(cell);
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
