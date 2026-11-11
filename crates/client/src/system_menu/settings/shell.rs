//! Settings' layer, as markup: its title and close button, the tabs, a page
//! for each, and the Interface page's rows. The other pages are filled by
//! their own files, into the pages here.

use bevy::prelude::*;

use super::super::{Action, Layer, SystemMenu};
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
    <div id="gameplay_page" class="rows" fills/>
    <div id="controls_page" class="rows" fills hidden/>
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
            ("settings", "gameplay_page") => bound.insert((Page(0), gameplay::GameplayPage)),
            ("settings", "controls_page") => bound.insert((Page(1), controls::ControlsPage)),
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

    /// The words of every `option` in these nodes, in order.
    fn options(nodes: &[CheckedNode], found: &mut Vec<String>) {
        for node in nodes {
            let CheckedNode::Element(element) = node else {
                continue;
            };
            if element.tag == Tag::Select {
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
