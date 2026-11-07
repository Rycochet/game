//! The moveable panels the menu opens: Previous and Next for what a place draws,
//! the world's time, and one showing every widget.

use bevy::prelude::*;

use crate::widgets::TextSize;

mod armour;
mod arms;
mod feet;
mod footwear;
mod hair;
mod legs;
mod neck;
mod page;
mod sets;
mod waist;

// Split into files (`cargo xtask lint` keeps files small).
mod change_creature;
mod every_widget;
mod generate;
mod generate_form;
mod give;
mod give_form;
mod give_option;
mod give_options;
mod give_preview;
mod give_systems;
mod go_to_world;
mod layout;
mod open;
mod pictures_form;
mod pictures_spawn;
mod pictures_systems;
mod ring_samples;
mod steps;
mod time_text;
mod world_rate;
mod world_time;

use change_creature::spawn_creature_buttons;
pub(super) use change_creature::{ChosenChange, press_creature_buttons, update_creature_buttons};
use every_widget::spawn_every_widget;
pub(super) use every_widget::{give_sample_icons, show_typed_items};
pub(super) use generate::{open_generate_panel, press_generate_buttons, update_generate_buttons};
pub(super) use generate_form::{
    GenerateButton, GenerateForm, GenerateKind, NewSeedButton, RuinButton, SeedBox,
};
use generate_form::{SEED_WIDTH, new_seed};
use give::spawn_give_panel;
use give_form::give_kinds;
pub(super) use give_form::{
    GiveButton, GiveChoice, GiveCount, GiveCountStep, GiveForm, GiveNoSpace, GiveOption,
    GiveOptionsLine, GivePreview, GiveRandomSeed, GiveSeed, SetChoice,
};
pub(super) use give_option::OptionValue;
use give_option::{FIELDS, any, ui_name};
pub(super) use give_options::{choose_give_option, fill_give_options};
pub(super) use give_preview::show_give_preview;
pub(super) use give_systems::{
    choose_give_options, edit_give_boxes, press_give_button, start_give_panel,
};
use go_to_world::spawn_go_to_world;
pub(super) use go_to_world::{press_go_to_world_buttons, update_go_to_world_button};
use layout::{row, words};
pub(super) use open::{OpenPanels, forget_closed};
use pictures_form::{
    BLADE_PICTURE, HANDS_PICTURE, LARGE_PICTURE, LARGE_PICTURE_PIXELS, PAGES_WIDTH, PICTURE,
    PICTURE_LABEL, PICTURE_PIXELS, PICTURE_ROW_LABEL, PICTURES_HEIGHT, RING_COLUMNS, RING_ROWS,
    Shown, gem_to_show,
};
pub(super) use pictures_form::{
    GemPicture, LargePicture, PicturePages, PictureQualities, PicturesForm, PicturesGrid,
};
use pictures_spawn::{
    picture_label, spawn_blade_row, spawn_body_row, spawn_item_pictures, spawn_picture,
    spawn_picture_heading, spawn_picture_row, spawn_ring_row,
};
pub(super) use pictures_systems::{choose_pictures, fill_pictures, show_pictures};
use steps::spawn_step_buttons;
pub(super) use steps::{press_panel_buttons, update_panel_buttons};
pub(super) use world_rate::{press_rate_buttons, update_rate_buttons};
use world_time::spawn_world_time;
pub(super) use world_time::{edit_time_box, press_time_steps};

const FONT: TextSize = TextSize::Medium;
/// How much of the window's height Show Widgets may take before it scrolls.
const EVERY_WIDGET_HEIGHT: f32 = 80.0;
const CHECK_ROW: f32 = 34.0;
/// Where the first panel goes, and how far along each one after it.
const FIRST_AT: Vec2 = Vec2::new(60.0, 110.0);
const NEXT_AT: Vec2 = Vec2::new(30.0, 30.0);

#[cfg(test)]
mod tests;
