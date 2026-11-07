//! The Skills page, the Character panel's Skills tab (K): the skills the character
//! knows as a radial tree round them, moved by dragging and zoomed by the wheel or
//! a pinch. See
//! `docs/context/client/skills_page.md`.

use bevy::prelude::*;

use crate::common::GameView;
use crate::widgets::WidgetSystems;

mod build;
mod details;
mod icons;
mod input;
mod layout;
mod place;
mod spawn;
mod values;
mod view;

pub struct SkillsPagePlugin;

impl Plugin for SkillsPagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<view::SkillsView>()
            .add_systems(
                PreUpdate,
                input::take_input
                    .after(WidgetSystems::Modal)
                    .run_if(not(in_state(GameView::Connecting))),
            )
            .add_systems(
                Update,
                (
                    spawn::fill_page,
                    build::build_tree,
                    values::show_values,
                    place::place_tree,
                    details::show_details,
                )
                    .chain(),
            );
    }
}
