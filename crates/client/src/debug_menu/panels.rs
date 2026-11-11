//! The moveable panels the menu opens: Drawing States, the world's time,
//! Teleport to World, Vitals, Give Item and the rest.

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
mod under;
mod waist;

// Split into files (`cargo xtask lint` keeps files small).
mod bind;
mod change_creature;
mod create_mob;
mod create_mob_systems;
mod dev_area;
mod dev_area_systems;
mod every_widget;
mod examine_mob;
mod generate;
mod generate_form;
mod give;
mod give_form;
mod give_option;
mod give_options;
mod give_preview;
mod give_systems;
mod give_target;
mod html_sample;
mod kept_choices;
mod layout;
mod learn_skills;
mod learn_skills_systems;
mod open;
mod pictures_form;
mod pictures_spawn;
mod pictures_systems;
mod ring_samples;
mod steps;
mod teleport_to_world;
mod teleport_to_world_systems;
mod time_text;
mod vitals;
mod world_rate;
mod world_time;

pub(super) use bind::bind_panels;
pub(super) use change_creature::{ChosenChange, press_creature_buttons, update_creature_buttons};
use create_mob::spawn_create_mob_panel;
pub(super) use create_mob::{MobChoice, MobClickToPlace, MobCreate, MobNewSeed, MobSeed, MobStill};
pub(super) use create_mob_systems::add_create_mob_panel;
use dev_area::spawn_dev_area_panel;
pub(super) use dev_area::{
    DevAreaChoice, DevAreaCount, DevAreaCountStep, DevAreaForm, DevAreaGenerate, DevAreaNewSeed,
    DevAreaSeed,
};
pub(super) use dev_area_systems::add_dev_area_panel;
use every_widget::spawn_every_widget;
pub(super) use every_widget::{give_sample_icons, show_typed_items};
pub(super) use examine_mob::add_examine_mob_panel;
use examine_mob::spawn_examine_panel;
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
pub(super) use give_target::{GiveTo, GiveToWords, follow_target};
pub(super) use html_sample::{fill_html_sample, show_html_events};
pub(super) use kept_choices::add_kept_choices;
use layout::{row, show_if, words};
use learn_skills::spawn_learn_skills;
pub(super) use learn_skills_systems::add_learn_skills_panel;
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
pub(super) use steps::{press_panel_buttons, update_panel_buttons};
pub(super) use teleport_to_world::TeleportForm;
use teleport_to_world::spawn_teleport_to_world;
pub(super) use teleport_to_world_systems::{
    open_teleport_asked, press_teleport_buttons, update_teleport_buttons,
};
pub(super) use vitals::{press_vitals_buttons, update_vitals_buttons};
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
