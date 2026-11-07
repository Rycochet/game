//! `HOW_DEBUG_DEV_AREA=size,generator,mobs,count,seed`: start in a Dev Area
//! (as the debug menu's), and reading its choices by name for that and for the
//! debug API's `client.dev_area`.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::protocol::ClientMessage;
use shared::site::{DEV_AREA_SIZES, DevArea, DevLayout, DevMobs};

use super::Start;
use crate::common::GameView;
use crate::connection::ToServer;

const VARIABLE: &str = "HOW_DEBUG_DEV_AREA";

pub(super) fn add(app: &mut App) {
    let Ok(text) = std::env::var(VARIABLE) else {
        return;
    };
    let area = match parse_dev_area(&text) {
        Ok(area) => area,
        Err(why) => {
            warn!("{VARIABLE}: {why}");
            return;
        }
    };
    app.add_systems(
        OnExit(GameView::Connecting),
        (move |mut to_server: MessageWriter<ToServer>| {
            to_server.write(ToServer(ClientMessage::DevGenerateArea(area)));
        })
        .in_set(Start::Enter),
    );
}

/// `size,generator,mobs,count,seed`, each optional: `64,moat,wolf,3,test`.
pub(super) fn parse_dev_area(text: &str) -> Result<DevArea, String> {
    let mut parts = text.split(',').map(str::trim);
    let size = parts.next().filter(|part| !part.is_empty());
    let size = size
        .map(|size| size.parse().map_err(|_| format!("no size {size:?}")))
        .transpose()?;
    let layout = parts.next();
    let mobs = parts.next();
    let count = parts.next().filter(|part| !part.is_empty());
    let count = count
        .map(|count| count.parse().map_err(|_| format!("no count {count:?}")))
        .transpose()?;
    dev_area_of(size, layout, mobs, count, parts.next())
}

/// A Dev Area from its choices by name, missing ones as the panel starts:
/// size 32, Empty, Default, 3 of a kind, seed 1. A seed that isn't a number is
/// hashed, as the panel does.
pub(super) fn dev_area_of(
    size: Option<u16>,
    layout: Option<&str>,
    mobs: Option<&str>,
    count: Option<u8>,
    seed: Option<&str>,
) -> Result<DevArea, String> {
    let size = size.unwrap_or(DEV_AREA_SIZES[1]);
    let layout = match layout.filter(|name| !name.is_empty()) {
        None => DevLayout::Empty,
        Some(name) => DevLayout::named(name).ok_or_else(|| {
            let names: Vec<_> = DevLayout::all().into_iter().map(DevLayout::name).collect();
            format!("no generator {name:?}: one of {}", names.join(", "))
        })?,
    };
    let count = count.unwrap_or(3);
    let mobs = match mobs.filter(|name| !name.is_empty()) {
        None => DevMobs::Default,
        Some(name) if name.eq_ignore_ascii_case("default") => DevMobs::Default,
        Some(name) if name.eq_ignore_ascii_case("none") => DevMobs::None,
        Some(name) => DevMobs::Kind {
            kind: kind_named(name)?,
            count,
        },
    };
    let seed = seed.filter(|seed| !seed.is_empty()).unwrap_or("1");
    Ok(DevArea {
        size,
        layout,
        mobs,
        seed: DevArea::seed_from_text(seed),
    }
    .checked())
}

/// A kind of creature by its name, as shown or as saved, in any case.
fn kind_named(name: &str) -> Result<CreatureKind, String> {
    let squashed = |text: &str| text.replace([' ', '_', '-'], "").to_lowercase();
    let wanted = squashed(name);
    CreatureKind::ALL
        .into_iter()
        .find(|kind| {
            squashed(kind.def().name) == wanted || squashed(&format!("{kind:?}")) == wanted
        })
        .ok_or_else(|| {
            let names: Vec<_> = CreatureKind::ALL.iter().map(|k| k.def().name).collect();
            format!(
                "no mobs {name:?}: Default, None or one of {}",
                names.join(", ")
            )
        })
}

#[cfg(test)]
mod tests;
