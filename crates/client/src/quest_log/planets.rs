//! The planet chooser: every planet the character has quest history on, the
//! one they are on first, each named by its runes (a picture made once per
//! planet), the one they are on also "Here".

use bevy::prelude::*;
use shared::lang::Lang;
use shared::protocol::PlanetQuests;
use shared::world::WorldId;

use super::runes::picture;
use super::state::LogPanel;
use crate::dom::Dom;
use crate::quests::plain;

const CHOOSER: &str = "quest_log/planet";

/// The chooser's planets in order: the one the player is on (`None`) first,
/// then the others as the server listed them.
pub(super) fn choices(planets: &[PlanetQuests]) -> Vec<Option<WorldId>> {
    let others = planets.iter().filter(|planet| !planet.here);
    std::iter::once(None)
        .chain(others.map(|planet| Some(planet.world)))
        .collect()
}

/// Puts the chooser's options in again, keeping the planet shown if it is
/// still among them.
pub(super) fn show_planets(
    panel: &mut LogPanel,
    planets: &[PlanetQuests],
    lang: &Lang,
    images: &mut Assets<Image>,
    dom: &mut Dom,
) {
    panel.choices = choices(planets);
    if !panel.choices.contains(&panel.planet) {
        panel.planet = None;
        panel.chosen = None;
    }
    let here = plain(lang, "log-here");
    let options = (panel.choices.iter().enumerate())
        .map(|(at, choice)| {
            let words = if choice.is_none() {
                here.clone()
            } else {
                String::new()
            };
            (at.to_string(), words)
        })
        .collect::<Vec<_>>();
    dom.set_options(CHOOSER, options);
    // Every planet's runes, or none: the options' pictures go by their order.
    let code_of = |choice: &Option<WorldId>| {
        let planet = planets
            .iter()
            .find(|planet| choice.map_or(planet.here, |world| planet.world == world));
        planet.and_then(|planet| Some((planet.world, planet.code?)))
    };
    let codes: Option<Vec<_>> = panel.choices.iter().map(code_of).collect();
    if let Some(codes) = codes {
        let pictures = codes
            .into_iter()
            .map(|(world, code)| {
                let made = panel.runes.entry(world);
                made.or_insert_with(|| images.add(picture(code))).clone()
            })
            .collect();
        dom.set_option_pictures(CHOOSER, pictures);
    }
    let shown = panel
        .choices
        .iter()
        .position(|&choice| choice == panel.planet);
    dom.set(CHOOSER, "value", &shown.unwrap_or(0).to_string());
}
