//! Giving items and changing the character's creature.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params};
use shared::creature::CreatureKind;
use shared::item::{Favour, ItemKind, Quality};
use shared::protocol::ClientMessage;

use super::done;
use crate::connection::ToServer;

#[derive(Deserialize)]
pub(super) struct GiveItemParams {
    /// By name: `"Ring"`, `"Backpack"`, ...
    kind: ItemKind,
    #[serde(default)]
    quality: Quality,
    #[serde(default)]
    magical: bool,
    #[serde(default)]
    set: bool,
    /// `"Blessed"` or `"Cursed"`.
    favour: Option<Favour>,
}

/// `client.give_item { "kind": "Ring", "quality": "Rare", "magical": true }`:
/// into the pockets, as Give Item does. Saved.
pub(super) fn give_item(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let GiveItemParams {
        kind,
        quality,
        magical,
        set,
        favour,
    } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevGiveItem {
        kind,
        size: None,
        magical,
        quality,
        set,
        favour,
    }));
    done()
}

#[derive(Deserialize)]
pub(super) struct SetCreatureParams {
    /// By name: `"Human"`, ...
    kind: CreatureKind,
}

/// `client.set_creature { "kind": "Human" }`: as Change Creature
/// (`HOW_DEBUG_CREATURE`); what no longer fits is lost.
pub(super) fn set_creature(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let SetCreatureParams { kind } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevSetCreature(kind)));
    done()
}
