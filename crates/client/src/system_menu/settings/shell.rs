//! Settings' layer, as markup: its title and close button, the tabs, a page
//! for each, and the Gameplay and Interface pages' rows (what Gameplay's
//! show is `gameplay`'s). The other pages are filled by their own files,
//! into the pages here.

use bevy::prelude::*;

use super::super::{Action, Layer, SystemMenu};
use super::difficulty::DifficultyList;
use super::shown_role::ShownList;
use super::{
    Page, PageTabs, SCROLL_BARS, ScrollBarsSwitch, SizeDropdown, TOUCH, TOUCH_ONLY, TouchSwitch,
    controls, gameplay, graphics, size_index, size_options,
};
use crate::common::Preferences;
use crate::dom::{Dom, Template, document};
use crate::html;
use crate::widgets::{TouchOnly, UiId};

/// The layer. A page not chosen is hidden, so the chosen one has the room.
/// The sizes on offer are checked against `size_options` by a test.
static SETTINGS: Template = html!(
    "system_menu.settings",
    r#"
<form id="settings">
  <dialog id="layer" title="Settings" modal closable depth="1" width="50%" height="75%" hidden>
    <tab-bar id="page">
      <option>Gameplay</option>
      <option>Controls</option>
      <option>Interface</option>
      <option>Graphics</option>
    </tab-bar>
    <div id="gameplay_page" fills>
      <form id="settings.gameplay" class="rows">
        <span id="difficulty_row" class="labelled" nowrap>
          <label for="difficulty" width="240">Difficulty</label>
          <select id="difficulty" width="200">
            <option>Normal</option>
          </select>
        </span>
        <span id="shown_as_row" class="labelled" nowrap hidden>
          <label for="shown_as" width="240">Shown As</label>
          <select id="shown_as" width="200">
            <option>Player</option>
          </select>
        </span>
        <span id="pause_unfocused_row" class="labelled" nowrap>
          <label for="pause_unfocused" width="240">Pause When Not in Focus</label>
          <input id="pause_unfocused" type="checkbox" switch/>
        </span>
      </form>
    </div>
    <div id="controls_page" fills hidden>
      <form id="settings.controls" fills>
        <scroll id="scroll" fills>
          <div class="groups">
            <div id="groups" class="groups"/>
            <span align="right">
              <button id="reset">Reset to Defaults</button>
            </span>
          </div>
        </scroll>
        <template id="group">
          <div class="rows">
            <div class="tight">
              <h2 id="heading"/>
              <hr/>
            </div>
            <table id="keys" class="striped" cellpadding="4" width="100%"/>
          </div>
        </template>
        <template id="row">
          <tr>
            <td id="name" fills valign="middle"/>
            <td><key-box id="first" width="200"/></td>
            <td><key-box id="second" width="200"/></td>
          </tr>
        </template>
      </form>
    </div>
    <div id="interface_page" fills hidden>
      <form id="settings.interface" class="rows">
        <span class="labelled" nowrap
              title="How big the interface is drawn. Auto follows the size of the window.">
          <label for="size" width="240">Interface Size</label>
          <select id="size" width="160">
            <option>Auto</option>
            <option>50%</option>
            <option>75%</option>
            <option>100%</option>
            <option>125%</option>
            <option>150%</option>
            <option>175%</option>
            <option>200%</option>
            <option>225%</option>
            <option>250%</option>
            <option>275%</option>
            <option>300%</option>
          </select>
        </span>
        <span id="touch_row" class="labelled" nowrap>
          <label for="touch" width="240">Enable Touch</label>
          <input id="touch" type="checkbox" switch/>
        </span>
        <span id="scroll_bars_row" class="labelled" nowrap>
          <label for="scroll_bars" width="240">Always Show Scroll Bars</label>
          <input id="scroll_bars" type="checkbox" switch/>
        </span>
      </form>
    </div>
    <div id="graphics_page" class="rows" fills hidden/>
  </dialog>
</form>
"#
);

pub(in crate::system_menu) fn spawn_settings(commands: &mut Commands, dom: &mut Dom) {
    commands.spawn(document(&SETTINGS));
    dom.set("settings.interface/scroll_bars_row", "title", SCROLL_BARS);
}

/// Gives the built layer, its tabs, pages and the Interface page's controls
/// what Settings' systems know them by, found by their ids.
pub(in crate::system_menu) fn bind_settings(
    mut commands: Commands,
    new: Query<(Entity, &UiId), Added<UiId>>,
) {
    const SHOWN: &[SystemMenu] = &[
        SystemMenu::Settings,
        SystemMenu::KeepGraphics,
        SystemMenu::Ironman,
    ];
    for (entity, id) in &new {
        let mut bound = commands.entity(entity);
        match id.parts() {
            ("settings", "layer") => bound.insert((Layer(SHOWN), Action::Back)),
            ("settings", "close") => bound.insert(Action::Back),
            ("settings", "page") => bound.insert(PageTabs),
            ("settings", "gameplay_page") => bound.insert(Page(0)),
            ("settings.gameplay", "difficulty") => bound.insert(DifficultyList::default()),
            ("settings.gameplay", "shown_as") => bound.insert(ShownList::default()),
            ("settings.gameplay", "pause_unfocused") => bound.insert(gameplay::PauseSwitch),
            ("settings", "controls_page") => bound.insert(Page(1)),
            ("settings.controls", _) => {
                controls::bind_controls(&mut bound, id);
                continue;
            }
            ("settings", "interface_page") => bound.insert(Page(2)),
            ("settings", "graphics_page") => bound.insert((Page(3), graphics::GraphicsPage)),
            ("settings.interface", "size") => bound.insert(SizeDropdown),
            ("settings.interface", "touch") => bound.insert(TouchSwitch),
            ("settings.interface", "scroll_bars") => bound.insert(ScrollBarsSwitch),
            _ => continue,
        };
    }
}

/// Shows the Interface page's choices as the preferences have them, when
/// they change (and at the start).
pub(in crate::system_menu) fn show_interface_page(
    preferences: Res<Preferences>,
    touch_only: Res<TouchOnly>,
    mut dom: Dom,
    mut shown: Local<bool>,
) {
    if *shown && !preferences.is_changed() && !touch_only.is_changed() {
        return;
    }
    *shown = true;
    let size = &size_options()[size_index(preferences.interface_size)];
    dom.set("settings.interface/size", "value", size);
    // A device with only a touch screen has it on, and can't turn it off.
    let touch = preferences.touch || touch_only.0;
    dom.set_flag("settings.interface/touch", "checked", touch);
    dom.set_flag("settings.interface/touch", "disabled", touch_only.0);
    let tip = match touch_only.0 {
        true => format!("{TOUCH}\n\n{TOUCH_ONLY}"),
        false => TOUCH.to_owned(),
    };
    dom.set("settings.interface/touch_row", "title", &tip);
    let bars = preferences.always_show_scroll_bars;
    dom.set_flag("settings.interface/scroll_bars", "checked", bars);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{CheckedNode, Tag};

    /// The words of every `option` of the size's select, in order.
    fn options(nodes: &[CheckedNode], found: &mut Vec<String>) {
        for node in nodes {
            let CheckedNode::Element(element) = node else {
                continue;
            };
            let size = element.id.as_ref().is_some_and(|id| id.parts().1 == "size");
            if element.tag == Tag::Select && size {
                for option in &element.children {
                    if let CheckedNode::Element(option) = option
                        && let Some(CheckedNode::Text(words)) = option.children.first()
                    {
                        found.push(words.clone());
                    }
                }
            }
            options(&element.children, found);
        }
    }

    #[test]
    fn the_layer_is_good_markup_offering_every_size() {
        let nodes = SETTINGS.checked().expect("good markup");
        let mut offered = Vec::new();
        options(&nodes, &mut offered);
        assert_eq!(offered, size_options());
    }
}
