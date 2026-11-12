//! The Nearby panel: who is near enough to give something to. An item held and
//! dropped on someone's row is given to them.

use bevy::prelude::*;
use shared::id::Id;
use shared::mob::GIVE_REACH;
use shared::sheet::Feeling;

use super::Holding;
use super::hold::Target;
use crate::common::{MapPosition, Other};
use crate::dom::{Dom, Template, UiEvent, UiValue, document, top_strip};
use crate::html;
use crate::player::Player;
use crate::widgets::{ALLOWED, GRID_COLOUR, UiId};

/// The panel, which moves: a row for each of those near enough, made from
/// the template, or the words saying there is nobody.
pub(super) static NEARBY: Template = html!(
    "nearby",
    r#"
<form id="nearby">
  <panel id="panel" title="Nearby" moveable left="900" top="300">
    <p id="nobody" class="medium">Nobody near enough</p>
    <div id="rows" class="tight"/>
  </panel>
  <template id="row">
    <span id="who" class="place" width="200" height="28" valign="middle">
      <p id="name" class="medium"/>
    </span>
  </template>
</form>
"#
);

/// The panel's document, if open, and who its rows are for, in order.
#[derive(Resource, Default)]
pub(super) struct NearbyPanel {
    panel: Option<Entity>,
    near: Vec<(Id, Feeling)>,
}

/// A row for someone: an item dropped here, or on anything in it, goes to them.
#[derive(Component, Clone, Copy)]
pub(super) struct GiveRow(pub Id);

impl NearbyPanel {
    pub(super) fn open(
        &mut self,
        commands: &mut Commands,
        dom: &mut Dom,
        nodes: &Query<(), With<Node>>,
    ) {
        if self.panel.is_some_and(|panel| nodes.contains(panel)) {
            dom.raise("nearby/panel");
            return;
        }
        // (On a strip: its own node takes no room over the view.)
        let mut nearby = commands.spawn(document(&NEARBY));
        self.panel = Some(nearby.insert(top_strip()).id());
    }

    pub(super) fn close(&mut self, commands: &mut Commands, nodes: &Query<(), With<Node>>) {
        if let Some(panel) = self.panel.take().filter(|&panel| nodes.contains(panel)) {
            commands.entity(panel).despawn();
        }
    }
}

/// Forgets the panel once its close button has closed it, which it says by
/// its id: its document goes then.
pub(super) fn forget_closed(
    mut events: MessageReader<UiEvent>,
    mut panel: ResMut<NearbyPanel>,
    nodes: Query<(), With<Node>>,
    mut commands: Commands,
) {
    let closed = |event: &UiEvent| {
        event.id == UiId::new("nearby", "panel") && event.value == UiValue::Closed
    };
    if events.read().filter(|event| closed(event)).count() > 0 {
        panel.close(&mut commands, &nodes);
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

/// Lists who is near enough, when that changes or the panel opens.
pub(super) fn show_nearby(
    player: Single<&MapPosition, With<Player>>,
    others: Query<(&Other, &MapPosition)>,
    mut panel: ResMut<NearbyPanel>,
    mut dom: Dom,
    mut shown_in: Local<Option<Entity>>,
) {
    if panel.panel.is_none() {
        *shown_in = None;
        return;
    }
    let mut near: Vec<(Id, Feeling, &'static str)> = others
        .iter()
        .filter(|(_, at)| in_reach(**player, **at))
        .map(|(other, _)| (other.id, other.feeling, other.kind.def().name))
        .collect();
    near.sort_by_key(|&(id, ..)| id);
    let now: Vec<_> = near.iter().map(|&(id, feeling, _)| (id, feeling)).collect();
    if panel.near == now && *shown_in == panel.panel {
        return;
    }
    *shown_in = panel.panel;
    panel.near = now;
    dom.set_flag("nearby/nobody", "hidden", !near.is_empty());
    dom.empty("nearby/rows");
    for (at, &(_, feeling, name)) in near.iter().enumerate() {
        dom.copy_template("nearby/row", "nearby/rows", at);
        let words = format!("{name}{}", feeling_word(feeling));
        dom.set_text(&format!("nearby/name.{at}"), words);
    }
}

/// Says who each new row is for.
pub(super) fn bind_rows(
    new: Query<(Entity, &UiId), Added<UiId>>,
    panel: Res<NearbyPanel>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        let (("nearby", "who"), Some(at)) = (id.parts(), id.index()) else {
            continue;
        };
        if let Some(&(who, _)) = panel.near.get(at) {
            commands.entity(entity).insert(GiveRow(who));
        }
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
