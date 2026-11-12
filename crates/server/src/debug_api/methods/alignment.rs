//! `server.alignment`: a player's hidden alignment (`shared::alignment`), read
//! or set. Nothing else tells it, but the debug menu's Alignment panel.

use bevy::prelude::*;
use serde::Deserialize;
use shared::alignment::Alignment;

use super::super::{BrpResult, Value, json, params, refused};
use crate::Connections;

#[derive(Deserialize)]
pub(super) struct AlignmentParams {
    /// Which connected player, in `server.state`'s order.
    #[serde(default)]
    player: usize,
    /// What to set it to (kept within −1000 and 1000), if anything.
    set: Option<i32>,
}

/// `server.alignment { "player": 0, "set": -400 }`: the player's alignment and
/// its band, after setting it if `set` is given.
pub(super) fn alignment(
    In(request): In<Option<Value>>,
    mut connections: ResMut<Connections>,
) -> BrpResult {
    let AlignmentParams { player, set } = params(request)?;
    let client = connections
        .clients_mut()
        .get_mut(player)
        .ok_or_else(|| refused(format!("no player {player}")))?;
    if let Some(value) = set {
        client.sheet.alignment = Some(Alignment::new(value));
        client.unsaved = true;
    }
    let alignment = client.sheet.alignment;
    Ok(json!({
        "character": client.character_id,
        "alignment": alignment.map(Alignment::value),
        "band": alignment.map(|alignment| alignment.band().name()),
    }))
}
