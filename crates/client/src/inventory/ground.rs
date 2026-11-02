//! What is on the ground under the player: a section of the Character panel, to the
//! right of the pockets, only while there is something there. The Drop icon is in
//! `panels`.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::id::Id;
use shared::protocol::ServerMessage;

use super::Carried;
use crate::common::{GROUND, GameView, GaveItem, HeldOverGifts, HeldStock, MapPosition, TookItem};
use crate::connection::FromServer;
use crate::player::Player;
use crate::widgets::{
    INK, StockAim, StockAimAt, StockGrid, StockGridNode, TextSize, draw_stock_aim,
    spawn_scroll_area,
};

/// How many cells across it is: the longest thing there can be.
const COLUMNS: u8 = 8;
/// The most of the window's height it takes, before it scrolls.
const MOST_HEIGHT: Val = Val::Vh(50.0);
const FONT: TextSize = TextSize::Medium;

/// What is on the ground where the server last said.
#[derive(Resource)]
pub(super) struct Ground {
    at: Option<(i64, i64)>,
    grid: StockGrid,
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

/// The section, for the Character panel, hidden until there is something to show.
pub(super) fn spawn_section(parent: &mut ChildSpawnerCommands) -> Entity {
    let mut holder = Entity::PLACEHOLDER;
    parent
        .spawn((
            GroundSection,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|section| {
            section.spawn((Text::new("On the ground"), FONT.font(), TextColor(INK)));
            holder = spawn_scroll_area(section, MOST_HEIGHT);
        });
    holder
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
            ServerMessage::EnterLevel(_)
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
            commands.entity(holder).despawn_related::<Children>();
            ground.grid.draw(&mut commands, holder, held.0);
        }
    }
}

/// Green where something held would land.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_aim(
    held: Res<HeldOverGifts>,
    carried: Res<Carried>,
    ground: Res<Ground>,
    grids: Query<(&StockGridNode, &ComputedNode, &UiGlobalTransform)>,
    aims: Query<(Entity, &StockAim)>,
    new_aims: Query<&StockAim, Added<StockAim>>,
    mut commands: Commands,
    mut drawn: Local<Option<StockAimAt>>,
) {
    let aim = held.0.filter(|gift| gift.to == GROUND).and_then(|gift| {
        let (_, node, transform) = grids.iter().find(|(grid, ..)| grid.0 == GROUND)?;
        ground.grid.aim(&gift, &carried.0, node, transform)
    });
    let new = new_aims.iter().any(|aim| aim.0 == GROUND);
    if *drawn == aim && !new {
        return;
    }
    *drawn = aim;
    let ours = aims.iter().filter(|(_, aim)| aim.0 == GROUND);
    draw_stock_aim(&mut commands, ours, aim);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bevy only finds two parameters clashing when a system starts.
    #[test]
    fn no_system_asks_for_what_another_of_its_parameters_has() {
        fn starts<M>(system: impl IntoSystem<(), (), M>) {
            IntoSystem::into_system(system).initialize(&mut World::new());
        }
        starts(read_server);
        starts(show_section);
        starts(show_aim);
        starts(super::super::hold_items);
    }
}
