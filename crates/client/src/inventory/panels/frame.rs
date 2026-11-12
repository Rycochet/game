//! A grid in its frame, as markup: sort and filter above it, and under it the
//! words on its weight and, for the inventory (whose frame is in the Character
//! panel's markup, `pages`), the Drop and bin slots. A bag's is in a panel of
//! its own. The places in it are filled here once it is built (`bind_frames`).

use bevy::prelude::*;
use shared::id::Id;
use shared::item::Holder;

use super::super::filter::{GridFilter, filter_options};
use super::super::sort::SortButton;
use super::{BinNode, BinSlot, DropNode, FONT, GridOwner, WeightText};
use crate::dom::Dom;
use crate::dom::{DocumentOf, GridShown, SlotShown, Template, document, top_strip};
use crate::html;
use crate::widgets::{ButtonTooltip, INK, UiId};

/// A bag, in a panel that moves: one of several documents alike, by the bag
/// (`bag#<id>/...`). Whoever opens it gives the panel its title and place.
static BAG: Template = html!(
    "inventory.bag",
    r#"
<form id="bag">
  <panel id="panel" moveable>
    <container id="frame">
      <header>
        <button id="sort" class="small"><img src="picture:sort"/></button>
        <select id="filter" class="small"><option>All</option></select>
      </header>
      <object-grid id="grid" type="item" places="kept"/>
      <footer><canvas id="weight" fills/></footer>
    </container>
  </panel>
</form>
"#
);

/// A bag whose grid grows with what is in it: it scrolls past a height.
static GROWING_BAG: Template = html!(
    "inventory.growing_bag",
    r#"
<form id="bag">
  <panel id="panel" moveable>
    <container id="frame">
      <header>
        <button id="sort" class="small"><img src="picture:sort"/></button>
        <select id="filter" class="small"><option>All</option></select>
      </header>
      <scroll id="scroll" max-height="60%">
        <object-grid id="grid" type="item" places="kept" grows/>
      </scroll>
      <footer><canvas id="weight" fills/></footer>
    </container>
  </panel>
</form>
"#
);

/// A bag's panel, with its grid in its frame, built by the end of the frame:
/// its document. One that `grows` scrolls. (The inventory's own grid is in
/// the Character panel's markup.)
pub(super) fn spawn_bag(commands: &mut Commands, bag: Id, grows: bool) -> Entity {
    let template = if grows { &GROWING_BAG } else { &BAG };
    let mut document = commands.spawn((document(template), DocumentOf(bag)));
    // (On a strip: its own node takes no room over the view.)
    document.insert(top_strip()).id()
}

/// Fills the places of each frame once it is built: its sort and filter, its grid
/// (which draws itself: `show_grids` says what is in it), its weight and its
/// slots.
pub(in crate::inventory) fn bind_frames(
    new: Query<(Entity, &UiId), Added<UiId>>,
    mut commands: Commands,
    mut dom: Dom,
) {
    for (entity, id) in &new {
        let (form, name) = id.parts();
        let holder = match (form, id.which()) {
            ("inventory", _) => Holder::Inventory,
            ("bag", Some(bag)) => Holder::Inside(bag),
            _ => continue,
        };
        let mut place = commands.entity(entity);
        match name {
            "sort" => {
                place.insert((SortButton(holder), ButtonTooltip("Sort".into())));
            }
            // Its options are every category (`keep_filters` gives it their
            // pictures).
            "filter" => {
                place.insert((GridFilter(Some(holder)), ButtonTooltip("Filter".into())));
                dom.set_options(&id.to_string(), filter_options());
            }
            "grid" => {
                place.insert((GridOwner { holder }, GridShown::default()));
            }
            "weight" => {
                let words = (Text::default(), FONT.font(), TextColor(INK));
                place.with_child((WeightText { holder }, words));
            }
            // Slots drawing themselves: `show_drop` and `show_bins` say what
            // each shows.
            "drop" => {
                place.with_child((DropNode, SlotShown::default()));
            }
            "bin" => {
                place.with_child((BinNode, BinSlot, SlotShown::default()));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
