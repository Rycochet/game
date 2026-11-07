//! The Skills page (K, or the Abilities page's Skills button): a full-screen layer
//! showing the skills the character knows as a radial tree round them, moved by
//! dragging and zoomed by the wheel or a pinch. See
//! `docs/context/client/skills_page.md`.

use bevy::prelude::*;

use crate::common::GameView;
use crate::widgets::WidgetSystems;

mod build;
mod details;
mod icons;
mod input;
mod layout;
mod name;
mod place;
mod spawn;
mod view;

pub struct SkillsPagePlugin;

impl Plugin for SkillsPagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<view::SkillsView>()
            .init_resource::<name::OwnName>()
            .add_systems(Startup, spawn::spawn_page)
            .add_systems(
                PreUpdate,
                input::take_input
                    .in_set(WidgetSystems::Modal)
                    .run_if(not(in_state(GameView::Connecting))),
            )
            .add_systems(
                Update,
                (
                    name::read_name,
                    build::build_tree,
                    place::place_tree,
                    details::show_details,
                )
                    .chain(),
            );
    }
}
