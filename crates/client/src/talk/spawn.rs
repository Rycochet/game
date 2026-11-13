//! Opening a talk panel for a door, once the server has answered with
//! something to show, and closing them all on a new level.

use bevy::prelude::*;

use super::markup::TALK;
use super::{AT, NEXT_AT, OpenTalk, OpenTalks, Tab, key_of};
use crate::common::CurrentDungeon;
use crate::dom::{DocumentOf, Dom, document, top_strip};
use crate::widgets::UiId;

/// The panel for `building`'s door, opened at `tab` if it isn't open yet.
/// `None` if the building isn't on this level.
pub(super) fn open_panel<'a>(
    open: &'a mut OpenTalks,
    building: u16,
    tab: Tab,
    dungeon: &CurrentDungeon,
    dom: &mut Dom,
    commands: &mut Commands,
) -> Option<&'a mut OpenTalk> {
    if !open.is_open(building) {
        let look = dungeon.buildings.iter().find(|look| look.id == building)?;
        let at = AT + NEXT_AT * open.0.len() as f32;
        let key = key_of(building);
        // One of several documents alike, by building: `talk#<key>/stock`.
        let panel = (commands.spawn((document(&TALK), DocumentOf(key))))
            .insert(top_strip())
            .id();
        let id = UiId::of("talk", key, "panel").to_string();
        dom.set(&id, "title", look.kind.name());
        dom.start_at(&id, at);
        open.0.push(OpenTalk::new(building, panel, look.door, tab));
    }
    open.of(building)
}

/// Closes every panel, as on a new level (or another server).
pub(super) fn close_all(
    open: &mut OpenTalks,
    nodes: &Query<(), With<Node>>,
    commands: &mut Commands,
) {
    for talk in open.0.drain(..) {
        if nodes.contains(talk.panel) {
            commands.entity(talk.panel).despawn();
        }
    }
}
