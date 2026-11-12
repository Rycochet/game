//! The Character panel's tabs: Character (what they are like), Inventory (what they
//! wear and carry, and the ground), Abilities (filled by `abilities`) and Skills
//! (filled by `skills_page`). Only the chosen tab's page is laid out; the others
//! are kept, hidden. The panel is markup (`CHARACTER`), its places filled here
//! once it is built (`bind_pages`).

use bevy::prelude::*;

use super::super::body::spawn_rows;
use super::super::ground::spawn_section;
use super::super::wear_note::wear_note;
use crate::common::{AbilitiesPage, SkillsPage};
use crate::dom::{Dom, Template, UiEvent, UiValue};
use crate::html;
use crate::widgets::UiId;

/// The panel, which moves: the tabs, then a page for each. The Inventory page is
/// the clothes (with why something didn't go on under them), what is carried in
/// its frame, and the ground's section.
pub(super) static CHARACTER: Template = html!(
    "inventory.character",
    r#"
<form id="inventory">
  <panel id="panel" title="Character" moveable>
    <tab-bar id="tabs">
      <option>Character</option>
      <option>Inventory</option>
      <option>Abilities</option>
      <option>Skills</option>
    </tab-bar>
    <p id="sheet" class="medium"/>
    <span id="carried" class="labelled" valign="top">
      <div>
        <canvas id="clothes"/>
        <canvas id="wear_note"/>
      </div>
      <container id="frame">
        <header>
        <button id="sort" class="small"><img src="picture:sort"/></button>
        <select id="filter" class="small"><option>All</option></select>
      </header>
        <object-grid id="grid" type="item" places="kept"/>
        <footer>
          <canvas id="weight" fills/>
          <span>
            <object-slot id="drop" accepts="item"/>
            <object-slot id="bin" accepts="item"/>
          </span>
        </footer>
      </container>
      <canvas id="ground"/>
    </span>
    <canvas id="abilities"/>
    <canvas id="skills"/>
  </panel>
</form>
"#
);

/// A tab of the Character panel, in the order they are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::inventory) enum Page {
    Character,
    #[default]
    Inventory,
    Abilities,
    Skills,
}

impl Page {
    const ALL: [Self; 4] = [
        Self::Character,
        Self::Inventory,
        Self::Abilities,
        Self::Skills,
    ];
    /// As the tabs are written in `CHARACTER` (a test checks).
    const NAMES: [&str; 4] = ["Character", "Inventory", "Abilities", "Skills"];
    /// Each page's element.
    const IDS: [&str; 4] = [
        "inventory/sheet",
        "inventory/carried",
        "inventory/abilities",
        "inventory/skills",
    ];

    fn index(self) -> usize {
        self as usize
    }

    /// Shows this page and its tab, and hides the other pages.
    pub(super) fn show(self, dom: &mut Dom) {
        for (index, id) in Self::IDS.iter().enumerate() {
            dom.set_flag(id, "hidden", index != self.index());
        }
        dom.set("inventory/tabs", "value", Self::NAMES[self.index()]);
    }
}

/// Fills the pages' places once they are built: the clothes, the ground's
/// section, and the pages other modules fill (`character` writes the sheet). (The carried grid's
/// frame is `frame::bind_frames`'.)
pub(in crate::inventory) fn bind_pages(
    new: Query<(Entity, &UiId), Added<UiId>>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        let ("inventory", name) = id.parts() else {
            continue;
        };
        let mut place = commands.entity(entity);
        match name {
            "clothes" => {
                spawn_rows(&mut commands, entity);
                continue;
            }
            "wear_note" => {
                place.with_child(wear_note());
            }
            "ground" => {
                place.with_children(spawn_section);
            }
            "abilities" => {
                place.insert(AbilitiesPage);
            }
            "skills" => {
                place.insert(SkillsPage);
            }
            _ => {}
        }
    }
}

/// A tab pressed shows its page, and is remembered for next time.
pub(in crate::inventory) fn switch_pages(
    mut events: MessageReader<UiEvent>,
    mut open: ResMut<super::OpenGrids>,
    mut dom: Dom,
) {
    for event in events.read() {
        if event.id != UiId::new("inventory", "tabs") {
            continue;
        }
        let UiValue::Choice { index, .. } = event.value else {
            continue;
        };
        let Some(&page) = Page::ALL.get(index) else {
            continue;
        };
        if open.page != page {
            open.page = page;
            page.show(&mut dom);
        }
    }
}

#[cfg(test)]
mod tests;
