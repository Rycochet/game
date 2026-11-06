//! `client.ui.*`: widgets by their `UiId`, so a caller can read what the
//! interface shows and change it without moving the mouse. Changes go through
//! the widgets' own messages, as a player's would, and only to what a player
//! could use: on show and not disabled.

mod describe;
mod set;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use server::debug_api::{BrpResult, refused};

use super::fullscreen::Darkening;
use super::moveable::MoveablePanel;
use super::scroll::ScrollArea;
use super::slider::Slider;
use super::tabs::{Tab, Tabs};
use super::{
    ButtonState, ButtonTooltip, Checked, ColourPicker, Dropdown, Pressable, Switch, TextBox,
    TextFocus, UiId,
};

pub(super) fn add(app: &mut App) {
    use server::debug_api::AddDebugMethod;
    app.add_debug_method("client.ui.list", describe::list)
        .add_debug_method("client.ui.get", describe::get)
        .add_debug_method("client.ui.set", set::set)
        .add_debug_method("client.ui.press", set::press)
        .add_debug_method("client.ui.close", set::close);
}

/// What kind of widget something with a `UiId` is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    TextBox,
    Dropdown,
    ColourPicker,
    Slider,
    Tabs,
    Check,
    Switch,
    Panel,
    Scroll,
    Button,
    Other,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::TextBox => "text_box",
            Self::Dropdown => "dropdown",
            Self::ColourPicker => "colour_picker",
            Self::Slider => "slider",
            Self::Tabs => "tabs",
            Self::Check => "check",
            Self::Switch => "switch",
            Self::Panel => "panel",
            Self::Scroll => "scroll",
            Self::Button => "button",
            Self::Other => "other",
        }
    }
}

/// Everything with a `UiId`, and what tells what each is and holds.
#[derive(SystemParam)]
struct Widgets<'w, 's> {
    ids: Query<
        'w,
        's,
        (
            Entity,
            &'static UiId,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static InheritedVisibility,
        ),
    >,
    states: Query<'w, 's, &'static ButtonState>,
    pressable: Query<'w, 's, (), With<Pressable>>,
    text_boxes: Query<'w, 's, &'static TextBox>,
    dropdowns: Query<'w, 's, &'static Dropdown>,
    pickers: Query<'w, 's, &'static ColourPicker>,
    swatches: Query<'w, 's, (Entity, &'static super::colour_picker::SwatchOf)>,
    sliders: Query<'w, 's, &'static Slider>,
    rows: Query<'w, 's, &'static Tabs>,
    tabs: Query<'w, 's, (Entity, &'static Tab)>,
    checked: Query<'w, 's, &'static Checked>,
    switches: Query<'w, 's, &'static Switch>,
    panels: Query<'w, 's, (), With<MoveablePanel>>,
    scrolls: Query<'w, 's, &'static ScrollPosition, With<ScrollArea>>,
    children: Query<'w, 's, &'static Children>,
    texts: Query<'w, 's, &'static Text>,
    tooltips: Query<'w, 's, &'static ButtonTooltip>,
    parents: Query<'w, 's, &'static ChildOf>,
    depths: Query<'w, 's, &'static GlobalZIndex>,
    layers: Query<
        'w,
        's,
        (Entity, &'static GlobalZIndex, &'static InheritedVisibility),
        With<Darkening>,
    >,
    focus: Res<'w, TextFocus>,
    scale: Res<'w, UiScale>,
}

impl Widgets<'_, '_> {
    fn kind(&self, entity: Entity) -> Kind {
        if self.text_boxes.contains(entity) {
            Kind::TextBox
        } else if self.dropdowns.contains(entity) {
            Kind::Dropdown
        } else if self.pickers.contains(entity) {
            Kind::ColourPicker
        } else if self.sliders.contains(entity) {
            Kind::Slider
        } else if self.rows.contains(entity) {
            Kind::Tabs
        } else if self.checked.contains(entity) {
            Kind::Check
        } else if self.switches.contains(entity) {
            Kind::Switch
        } else if self.panels.contains(entity) {
            Kind::Panel
        } else if self.scrolls.contains(entity) {
            Kind::Scroll
        } else if self.pressable.contains(entity) {
            Kind::Button
        } else {
            Kind::Other
        }
    }

    /// Whether a player could see it: shown, and laid out with some size.
    fn shown(&self, entity: Entity) -> bool {
        self.ids
            .get(entity)
            .is_ok_and(|(_, _, node, _, seen)| seen.get() && node.size() != Vec2::ZERO)
    }

    /// Whether a full-screen layer it isn't in is in front of it, so a player
    /// couldn't reach it.
    fn covered(&self, entity: Entity) -> bool {
        let mut depth = 0;
        let mut within = vec![entity];
        let mut at = entity;
        while let Ok(parent) = self.parents.get(at) {
            at = parent.parent();
            within.push(at);
        }
        if let Some(z) = within.iter().find_map(|&e| self.depths.get(e).ok()) {
            depth = z.0;
        }
        self.layers
            .iter()
            .any(|(layer, z, seen)| seen.get() && z.0 > depth && !within.contains(&layer))
    }

    fn disabled(&self, entity: Entity) -> bool {
        self.states
            .get(entity)
            .is_ok_and(|state| state.is_disabled())
    }

    /// The one widget on show with this id, or why there isn't one.
    fn find(&self, id: &str) -> BrpResult<Entity> {
        let named: Vec<Entity> = self
            .ids
            .iter()
            .filter(|(_, ui_id, ..)| ui_id.to_string() == id)
            .map(|(entity, ..)| entity)
            .collect();
        let shown: Vec<Entity> = named.iter().copied().filter(|&e| self.shown(e)).collect();
        match (shown.as_slice(), named.is_empty()) {
            ([one], _) if self.covered(*one) => {
                Err(refused(format!("{id} is behind something in front of it")))
            }
            ([one], _) => Ok(*one),
            ([], true) => Err(refused(format!("no widget is called {id}"))),
            ([], false) => Err(refused(format!("{id} isn't on show"))),
            (_, _) => Err(refused(format!(
                "{} widgets on show are called {id}",
                shown.len()
            ))),
        }
    }

    /// Its name: the first writing in it or under it, or for a button showing
    /// a picture, its tooltip.
    fn label(&self, entity: Entity) -> Option<String> {
        self.words(entity)
            .or_else(|| self.tooltips.get(entity).ok().map(|tip| tip.0.clone()))
    }

    /// The first writing in it or under it.
    fn words(&self, entity: Entity) -> Option<String> {
        if let Ok(text) = self.texts.get(entity) {
            return Some(text.0.clone());
        }
        let children = self.children.get(entity).ok()?;
        children.iter().find_map(|child| self.words(child))
    }

    /// A row's tabs, in order, with their words.
    fn tabs_of(&self, row: Entity) -> Vec<(Entity, String)> {
        let mut tabs: Vec<(usize, Entity)> = self
            .tabs
            .iter()
            .filter(|(_, tab)| tab.row == row)
            .map(|(entity, tab)| (tab.index, entity))
            .collect();
        tabs.sort();
        tabs.into_iter()
            .map(|(_, entity)| (entity, self.words(entity).unwrap_or_default()))
            .collect()
    }
}
