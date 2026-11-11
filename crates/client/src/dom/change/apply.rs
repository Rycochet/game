//! Making the changes asked for through `Dom`, one at a time and in order.
//! It has the whole world to itself (an exclusive system: nothing else runs
//! meanwhile), so each change sees what the one before it made.

use std::borrow::Cow;

use bevy::prelude::*;

use super::{Change, DomChange};
use crate::dom::build::{Builder, Inside};
use crate::dom::parts::{Bar, Element, TemplateBody, TextStyle, Words};
use crate::dom::theme::Theme;
use crate::widgets::UiId;

pub(in crate::dom) fn apply_changes(world: &mut World) {
    let asked: Vec<DomChange> = world
        .resource_mut::<Messages<DomChange>>()
        .drain()
        .collect();
    for DomChange { id, change } in asked {
        let Some(entity) = find(world, &id) else {
            warn!("no element is `{id}`");
            continue;
        };
        let done = match change {
            Change::Text(words) => set_text(world, entity, words),
            Change::Set(name, value) => edit(world, entity, |element| {
                let value = value.map(Cow::Owned);
                element.attrs.set(element.tag, &name, value)
            }),
            Change::Clear(name) => edit(world, entity, |element| {
                element.attrs.clear(element.tag, &name)
            }),
            Change::Copy { template, index } => copy(world, entity, &template, index),
            Change::Empty => {
                world.entity_mut(entity).despawn_related::<Children>();
                Ok(())
            }
        };
        if let Err(problem) = done {
            warn!("`{id}`: {problem}");
        }
        world.flush();
    }
}

/// The element with this id, as written.
fn find(world: &mut World, id: &str) -> Option<Entity> {
    let mut ids = world.query_filtered::<(Entity, &UiId), With<Element>>();
    let found = ids.iter(world).find(|(_, known)| known.to_string() == id);
    found.map(|(entity, _)| entity)
}

/// Changes an element's attributes, if the change is allowed.
fn edit(
    world: &mut World,
    entity: Entity,
    change: impl FnOnce(&mut Element) -> Result<(), String>,
) -> Result<(), String> {
    let element = world.get::<Element>(entity).ok_or("it has gone")?;
    // On a copy, so a refused change leaves the element as it was.
    let mut changed = element.clone();
    change(&mut changed)?;
    if let Some(mut element) = world.get_mut::<Element>(entity) {
        *element = changed;
    }
    Ok(())
}

/// The words straight inside an element, or on its bar.
fn words_of(world: &World, entity: Entity) -> Vec<Entity> {
    let holder = world.get::<Bar>(entity).map_or(entity, |bar| bar.0);
    let children = world.get::<Children>(holder);
    let children = children
        .map(|children| children.to_vec())
        .unwrap_or_default();
    let is_words = |child: &Entity| world.get::<Words>(*child).is_some();
    children.into_iter().filter(is_words).collect()
}

fn set_text(world: &mut World, entity: Entity, words: String) -> Result<(), String> {
    let mut old = words_of(world, entity).into_iter();
    match old.next() {
        Some(first) => {
            if let Some(mut text) = world.get_mut::<Text>(first) {
                text.0 = words;
            }
            old.for_each(|extra| world.entity_mut(extra).despawn());
        }
        None => {
            let style = *world.get::<TextStyle>(entity).ok_or("it has no writing")?;
            build(world, |builder| {
                builder.words(entity, &words, style);
            });
        }
    }
    Ok(())
}

fn copy(world: &mut World, into: Entity, template: &str, index: usize) -> Result<(), String> {
    let template = find(world, template).ok_or_else(|| format!("no template is `{template}`"))?;
    let body = world.get::<TemplateBody>(template);
    let nodes = body.ok_or("that is not a template")?.0.clone();
    let style = *world.get::<TextStyle>(into).ok_or("it has no writing")?;
    let inside = Inside {
        style,
        index: Some(index),
    };
    build(world, |builder| builder.nodes(into, &nodes, inside));
    Ok(())
}

/// Runs `with` on a builder, with the theme as it is now.
fn build(world: &mut World, with: impl FnOnce(&mut Builder)) {
    world.resource_scope(|world, theme: Mut<Theme>| {
        let mut commands = world.commands();
        with(&mut Builder {
            commands: &mut commands,
            theme: &theme,
        });
    });
}
