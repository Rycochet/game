//! What is in the bags on the ground in the player's reach: a section of the
//! Character panel, to the right of the inventory, only while there is something
//! there. The Drop icon is in `panels`.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::ground::in_reach;
use shared::id::Id;
use shared::protocol::ServerMessage;
use shared::spot::Spot;

use super::Carried;
use super::filter::{GridFilter, filter_options};
use crate::common::{
    GROUND, GameView, GaveItem, HeldOverGifts, HeldStock, PlayerSize, SpotPosition, TookItem,
};
use crate::connection::FromServer;
use crate::dom::{Dom, GridShown, Template, document_in};
use crate::html;
use crate::player::Player;
use crate::widgets::{ButtonTooltip, StockGrid, StockGridNode, StockOf, UiId, show_stock_aim};

/// How many cells across it is: the longest thing there can be.
const COLUMNS: u8 = 8;

/// What is in the bags in reach, as the server last said: where they are, and
/// all that is in them.
#[derive(Resource)]
pub(super) struct Ground {
    bags: Vec<Spot>,
    pub(super) grid: StockGrid,
}

impl Default for Ground {
    fn default() -> Self {
        Self {
            bags: Vec::new(),
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
    <header>
      <select id="filter" class="small"><option>All</option></select>
    </header>
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
pub(super) fn bind_section(
    new: Query<(Entity, &UiId), Added<UiId>>,
    mut commands: Commands,
    mut dom: Dom,
) {
    for (entity, id) in &new {
        match id.parts() {
            ("ground", "filter") => {
                let marks = (GridFilter(None), ButtonTooltip("Filter".into()));
                commands.entity(entity).insert(marks);
                dom.set_options("ground/filter", filter_options());
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
            ServerMessage::GroundItems { bags, items } => {
                if ground.bags == *bags {
                    ground.grid.heard(items);
                } else {
                    // Places are only for the same bags, while they are in reach.
                    ground.bags = bags.clone();
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

/// Shows the section only while a bag it is of is still in the player's reach where
/// they really are (the server answers moves late), and draws its
/// grid again when what is in it changes, or when something is picked up from it or
/// put back. What is held isn't drawn where it was.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_section(
    view: Res<State<GameView>>,
    player: Single<&SpotPosition, With<Player>>,
    size: Res<PlayerSize>,
    held: Res<HeldStock>,
    mut ground: ResMut<Ground>,
    mut sections: Query<&mut Node, With<GroundSection>>,
    holders: Query<Entity, With<GroundHolder>>,
    new_holders: Query<(), Added<GroundHolder>>,
    mut commands: Commands,
    mut drawn_without: Local<Option<Id>>,
) {
    let reached = (ground.bags.iter()).any(|&bag| in_reach(size.0, player.0, bag));
    let shown = *view.get() == GameView::Dungeon && reached && !ground.grid.is_empty();
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
