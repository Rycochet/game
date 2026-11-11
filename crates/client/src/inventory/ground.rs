//! What is on the ground under the player: a section of the Character panel, to the
//! right of the inventory, only while there is something there. The Drop icon is in
//! `panels`.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::id::Id;
use shared::protocol::ServerMessage;

use super::Carried;
use super::filter::spawn_filter;
use crate::common::{GROUND, GameView, GaveItem, HeldOverGifts, HeldStock, MapPosition, TookItem};
use crate::connection::FromServer;
use crate::dom::{GridShown, Template, document_in};
use crate::html;
use crate::player::Player;
use crate::widgets::{StockGrid, StockGridNode, StockOf, UiId, show_stock_aim};

/// How many cells across it is: the longest thing there can be.
const COLUMNS: u8 = 8;

/// What is on the ground where the server last said.
#[derive(Resource)]
pub(super) struct Ground {
    at: Option<(i64, i64)>,
    pub(super) grid: StockGrid,
}

impl Default for Ground {
    fn default() -> Self {
        Self {
            at: None,
            grid: StockGrid::new(GROUND, COLUMNS, &[]),
        }
    }
}

/// The section, shown or not.
#[derive(Component)]
pub(super) struct GroundSection;

/// What its grid goes in.
#[derive(Component)]
pub(super) struct GroundHolder;

/// The section's contents: its title and filter, and its grid, which scrolls
/// past half the window's height.
static SECTION: Template = html!(
    "inventory.ground",
    r#"
<form id="ground">
  <container id="frame" title="On the ground">
    <header id="controls"/>
    <scroll id="scroll" max-height="50%">
      <object-grid id="grid" type="item" places="packed" grows/>
    </scroll>
  </container>
</form>
"#
);

/// The section, for the Character panel, hidden until there is something to show.
pub(super) fn spawn_section(parent: &mut ChildSpawnerCommands) {
    let hidden = Node {
        display: Display::None,
        ..default()
    };
    let section = parent.spawn((GroundSection, hidden)).id();
    parent.commands().spawn(document_in(&SECTION, section));
}

/// Fills the section's places once it is built: its filter, and the grid the
/// ground's things are shown in.
pub(super) fn bind_section(new: Query<(Entity, &UiId), Added<UiId>>, mut commands: Commands) {
    for (entity, id) in &new {
        match id.parts() {
            ("ground", "controls") => {
                let mut controls = commands.entity(entity);
                controls.with_children(|row| spawn_filter(row, None));
            }
            ("ground", "grid") => {
                commands.entity(entity).insert(GroundHolder);
            }
            _ => {}
        }
    }
}

/// What the server says is there, and what has been given to it or taken from it
/// here. A new map forgets it: the server says again what is there.
pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut gave: MessageReader<GaveItem>,
    mut took: MessageReader<TookItem>,
    grids: Query<(&StockGridNode, &ComputedNode, &UiGlobalTransform)>,
    mut ground: ResMut<Ground>,
) {
    for GaveItem(gift) in gave.read().filter(|gave| gave.0.to == GROUND) {
        let grid = grids.iter().find(|(grid, ..)| grid.0 == GROUND);
        if let Some((_, node, transform)) = grid {
            ground.grid.gave(gift, node, transform);
        }
    }
    for took in took.read().filter(|took| took.with == GROUND) {
        ground.grid.took(took.item);
    }
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::GroundItems { x, y, items } => {
                if ground.at == Some((*x, *y)) {
                    ground.grid.heard(items);
                } else {
                    // Places are only for the one pile, while standing on it.
                    ground.at = Some((*x, *y));
                    ground.grid = StockGrid::new(GROUND, COLUMNS, items);
                }
            }
            ServerMessage::Welcome { .. }
            | ServerMessage::EnterLevel(_)
            | ServerMessage::EnterHall { .. }
            | ServerMessage::OnWorldMap { .. }
            | ServerMessage::Teleported { .. } => *ground = Ground::default(),
            _ => {}
        }
    }
}

/// Shows the section only over what the player is really standing on, and draws its
/// grid again when what is in it changes, or when something is picked up from it or
/// put back. What is held isn't drawn where it was.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_section(
    view: Res<State<GameView>>,
    player: Single<&MapPosition, With<Player>>,
    held: Res<HeldStock>,
    mut ground: ResMut<Ground>,
    mut sections: Query<&mut Node, With<GroundSection>>,
    holders: Query<Entity, With<GroundHolder>>,
    new_holders: Query<(), Added<GroundHolder>>,
    mut commands: Commands,
    mut drawn_without: Local<Option<Id>>,
) {
    let here = Some((player.x, player.y));
    let shown = *view.get() == GameView::Dungeon && ground.at == here && !ground.grid.is_empty();
    let display = if shown { Display::Flex } else { Display::None };
    for mut node in &mut sections {
        if node.display != display {
            node.display = display;
        }
    }
    let picked_up = *drawn_without != held.0;
    *drawn_without = held.0;
    if ground.grid.changed || picked_up || !new_holders.is_empty() {
        ground.grid.changed = false;
        for holder in &holders {
            ground.grid.show(&mut commands, holder, held.0);
        }
    }
}

/// Green where something held would land.
pub(super) fn show_aim(
    held: Res<HeldOverGifts>,
    carried: Res<Carried>,
    ground: Res<Ground>,
    grids: Query<(&StockGridNode, &ComputedNode, &UiGlobalTransform)>,
    mut shown: Query<(&StockOf, &mut GridShown)>,
) {
    let aim = held.0.filter(|gift| gift.to == GROUND).and_then(|gift| {
        let (_, node, transform) = grids.iter().find(|(grid, ..)| grid.0 == GROUND)?;
        ground.grid.aim(&gift, &carried.0, node, transform)
    });
    let ours = shown.iter_mut().filter(|(of, _)| of.0 == GROUND);
    show_stock_aim(ours, aim);
}

#[cfg(test)]
mod tests;
