//! Writing the open log out when what it shows changes.

use bevy::prelude::*;

use super::page::show_page;
use super::planets::show_planets;
use super::state::LogPanel;
use crate::common::{CurrentWorld, GameView, Wording};
use crate::dom::Dom;
use crate::quests::{QuestBook, QuestPlanets};

#[allow(clippy::too_many_arguments)]
pub(super) fn show_log(
    mut panel: ResMut<LogPanel>,
    book: Res<QuestBook>,
    planets: Res<QuestPlanets>,
    wording: Res<Wording>,
    world: Option<Res<CurrentWorld>>,
    view: Res<State<GameView>>,
    mut images: ResMut<Assets<Image>>,
    mut dom: Dom,
) {
    if !panel.is_open() {
        return;
    }
    let lang = &wording.0;
    if panel.planets_dirty {
        panel.planets_dirty = false;
        show_planets(&mut panel, &planets.0, lang, &mut images, &mut dom);
    }
    // The places' buttons work only from the world map.
    if view.is_changed() {
        panel.dirty = true;
    }
    if panel.dirty {
        panel.dirty = false;
        let on_world_map = *view.get() == GameView::WorldMap;
        show_page(
            &mut panel,
            &book,
            lang,
            world.as_deref(),
            on_world_map,
            &mut dom,
        );
    }
}
