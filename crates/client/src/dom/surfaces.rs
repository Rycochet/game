//! Keeping panels and dialogs made from markup in step with their elements
//! and their documents: shown or hidden, and gone when either goes.

use std::collections::HashMap;

use bevy::prelude::*;

use super::attrs::Length;
use super::build::place_of;
use super::controls::{UiEvent, UiValue};
use super::parts::{Control, Document, Element, SurfaceOf};
use super::tag::Tag;
use crate::widgets::{MoveablePanel, PanelTitleOf, UiId};

/// Shows or hides the widget round a panel or dialog as its element's
/// `hidden` says. A hidden panel keeps its place. A panel that moves also
/// takes its `title`, and until it has first been placed, where it starts.
pub(super) fn follow_surfaces(
    changed: Query<(&Element, &Control), Changed<Element>>,
    mut layers: Query<&mut Visibility>,
    mut roots: Query<&mut Node, Without<Element>>,
    mut panels: Query<&mut MoveablePanel>,
    mut titles: Query<(&PanelTitleOf, &mut Text)>,
) {
    for (element, Control(widget)) in &changed {
        let hidden = element.attrs.hidden;
        match element.tag {
            Tag::Dialog => {
                let wanted = match hidden {
                    true => Visibility::Hidden,
                    false => Visibility::Inherited,
                };
                if let Ok(mut shown) = layers.get_mut(*widget)
                    && *shown != wanted
                {
                    *shown = wanted;
                }
            }
            Tag::Panel => {
                let wanted = match hidden {
                    true => Display::None,
                    false => Display::Flex,
                };
                if let Ok(mut node) = roots.get_mut(*widget)
                    && node.display != wanted
                {
                    node.display = wanted;
                }
                if let Ok(mut panel) = panels.get_mut(*widget) {
                    panel.start_at(place_of(&element.attrs));
                }
                let title = element.attrs.title.as_deref().unwrap_or_default();
                for (PanelTitleOf(of), mut words) in &mut titles {
                    if of == widget && words.0 != title {
                        words.0 = title.to_owned();
                    }
                }
            }
            // Where its `left` and `top` say, in the interface's pixels.
            Tag::Popup => {
                let Ok(mut node) = roots.get_mut(*widget) else {
                    continue;
                };
                let place = |side: Option<Length>| match side {
                    Some(Length::Pixels(pixels)) => px(pixels),
                    Some(Length::Percent(share)) => percent(share),
                    None => Val::Auto,
                };
                let (left, top) = (place(element.attrs.left), place(element.attrs.top));
                let display = match hidden {
                    true => Display::None,
                    false => Display::Flex,
                };
                if (node.left, node.top, node.display) != (left, top, display) {
                    (node.left, node.top, node.display) = (left, top, display);
                }
            }
            _ => {}
        }
    }
}

/// Panels and dialogs whose documents went this frame go too, before any
/// new documents are built: one built in a closed one's place has the same
/// ids, and changes asked for by them must find the new one.
pub(super) fn clear_surfaces(
    mut commands: Commands,
    surfaces: Query<(Entity, &SurfaceOf)>,
    documents: Query<(), With<Document>>,
) {
    for (surface, SurfaceOf(document)) in &surfaces {
        if !documents.contains(*document) {
            commands.entity(surface).try_despawn();
        }
    }
}

/// A panel or dialog goes when its document does. One that goes while its
/// document is still there was closed by the player (a panel's close
/// button), which is told by its id: its owner then despawns the document.
pub(super) fn tidy_surfaces(
    mut commands: Commands,
    surfaces: Query<(Entity, &SurfaceOf, Option<&UiId>)>,
    documents: Query<(), With<Document>>,
    mut gone: RemovedComponents<SurfaceOf>,
    mut known: Local<HashMap<Entity, (UiId, Entity)>>,
    mut events: MessageWriter<UiEvent>,
) {
    for (surface, SurfaceOf(document), id) in &surfaces {
        if !documents.contains(*document) {
            known.remove(&surface);
            commands.entity(surface).try_despawn();
        } else if let Some(id) = id {
            known.insert(surface, (*id, *document));
        }
    }
    for surface in gone.read() {
        // (Not one that went with its document: nobody closed that.)
        if let Some((id, document)) = known.remove(&surface)
            && documents.contains(document)
        {
            let value = UiValue::Closed;
            events.write(UiEvent { id, value });
        }
    }
}

#[cfg(test)]
mod tests;
