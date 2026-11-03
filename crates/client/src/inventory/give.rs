//! The Nearby panel: who is near enough to give something to. An item held and
//! dropped on someone's row is given to them.

use bevy::prelude::*;
use shared::id::Id;
use shared::mob::GIVE_REACH;
use shared::sheet::Feeling;

use super::Holding;
use super::hold::Target;
use crate::common::{MapPosition, Other};
use crate::player::Player;
use crate::widgets::{ALLOWED, GRID_COLOUR, INK, RaisePanel, TextSize, spawn_moveable_panel};

const AT: Vec2 = Vec2::new(900.0, 300.0);
const FONT: TextSize = TextSize::Medium;
const ROW_WIDTH: f32 = 200.0;
const ROW_HEIGHT: f32 = 28.0;
const ROW_GAP: f32 = 4.0;

/// The panel, if open.
#[derive(Resource, Default)]
pub(super) struct NearbyPanel(Option<Entity>);

/// What holds the rows.
#[derive(Component)]
pub(super) struct NearbyRows;

/// A row for someone: an item dropped here goes to them. Also on its words.
#[derive(Component, Clone, Copy)]
pub(super) struct GiveRow(pub Id);

impl NearbyPanel {
    pub(super) fn open(
        &mut self,
        commands: &mut Commands,
        raise: &mut MessageWriter<RaisePanel>,
        nodes: &Query<(), With<Node>>,
    ) {
        if let Some(panel) = self.0.filter(|&panel| nodes.contains(panel)) {
            raise.write(RaisePanel(panel));
            return;
        }
        let (panel, content) = spawn_moveable_panel(commands, "Nearby", AT);
        commands.entity(content).with_child((
            NearbyRows,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(ROW_GAP),
                ..default()
            },
        ));
        self.0 = Some(panel);
    }

    pub(super) fn close(&mut self, commands: &mut Commands, nodes: &Query<(), With<Node>>) {
        if let Some(panel) = self.0.take().filter(|&panel| nodes.contains(panel)) {
            commands.entity(panel).despawn();
        }
    }
}

/// Forgets the panel once its close button has closed it.
pub(super) fn forget_closed(mut panel: ResMut<NearbyPanel>, nodes: Query<(), With<Node>>) {
    if panel.0.is_some_and(|open| !nodes.contains(open)) {
        panel.0 = None;
    }
}

/// How it feels, in words.
fn feeling_word(feeling: Feeling) -> &'static str {
    match feeling {
        Feeling::None => "",
        Feeling::Friendly => " (friendly)",
        Feeling::Neutral => " (neutral)",
        Feeling::Unfriendly => " (unfriendly)",
    }
}

/// Whether something is near enough to be given something from here. The server
/// checks this, and that it is in sight.
fn in_reach(from: MapPosition, to: MapPosition) -> bool {
    (from.x - to.x).abs().max((from.y - to.y).abs()) <= GIVE_REACH
}

/// Lists who is near enough, when that changes.
pub(super) fn show_nearby(
    player: Single<&MapPosition, With<Player>>,
    others: Query<(&Other, &MapPosition)>,
    rows: Query<Entity, With<NearbyRows>>,
    new_rows: Query<(), Added<NearbyRows>>,
    mut commands: Commands,
    mut drawn: Local<Vec<(Id, Feeling)>>,
) {
    let mut near: Vec<(Id, Feeling, &'static str)> = others
        .iter()
        .filter(|(_, at)| in_reach(**player, **at))
        .map(|(other, _)| (other.id, other.feeling, other.kind.def().name))
        .collect();
    near.sort_by_key(|&(id, ..)| id);
    let now: Vec<_> = near.iter().map(|&(id, feeling, _)| (id, feeling)).collect();
    if *drawn == now && new_rows.is_empty() {
        return;
    }
    *drawn = now;
    for entity in &rows {
        commands.entity(entity).despawn_related::<Children>();
        commands.entity(entity).with_children(|rows| {
            if near.is_empty() {
                rows.spawn((Text::new("Nobody near enough"), FONT.font(), TextColor(INK)));
            }
            for &(id, feeling, name) in &near {
                rows.spawn((
                    GiveRow(id),
                    Node {
                        width: px(ROW_WIDTH),
                        height: px(ROW_HEIGHT),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(GRID_COLOUR),
                ))
                .with_child((
                    GiveRow(id),
                    Text::new(format!("{name}{}", feeling_word(feeling))),
                    FONT.font(),
                    TextColor(INK),
                ));
            }
        });
    }
}

/// Green behind the row a drop would give to.
pub(super) fn show_give_aim(
    holding: Res<Holding>,
    mut rows: Query<(&GiveRow, &mut BackgroundColor)>,
) {
    for (row, mut background) in &mut rows {
        let aimed = holding.aim == Some(Target::Someone(row.0));
        background.set_if_neq(BackgroundColor(if aimed { ALLOWED } else { GRID_COLOUR }));
    }
}

#[cfg(test)]
mod tests;
