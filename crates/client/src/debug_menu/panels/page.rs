//! The pages of Item Pictures' grid, one kind of item at a time.

use bevy::prelude::*;
use shared::blade::Family;
use shared::material::Material;

use super::{
    RING_ROWS, armour, arms, feet, footwear, legs, neck, sets, spawn_blade_row, spawn_body_row,
    spawn_picture_heading, spawn_picture_row, spawn_ring_row, waist,
};
use crate::common::Preferences;

/// What Item Pictures' grid shows: one kind of item at a time, chosen from a dropdown
/// and kept in `Preferences` by name.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(in crate::debug_menu) enum PicturePage {
    Bodies,
    Hands,
    Shoulders,
    Arms,
    Torso,
    Waist,
    Legs,
    Feet,
    Footwear,
    Head,
    Neck,
    Sets,
    Weapons,
    #[default]
    Gems,
    Rings,
}

impl PicturePage {
    pub(super) const ALL: [Self; 15] = [
        Self::Bodies,
        Self::Hands,
        Self::Shoulders,
        Self::Arms,
        Self::Torso,
        Self::Waist,
        Self::Legs,
        Self::Feet,
        Self::Footwear,
        Self::Head,
        Self::Neck,
        Self::Sets,
        Self::Weapons,
        Self::Gems,
        Self::Rings,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Bodies => "Bodies",
            Self::Hands => "Hands",
            Self::Shoulders => "Shoulders",
            Self::Arms => "Arms",
            Self::Legs => "Legs",
            Self::Feet => "Feet",
            Self::Footwear => "Footwear",
            Self::Neck => "Neck",
            Self::Sets => "Sets",
            Self::Waist => "Waist",
            Self::Torso => "Torso",
            Self::Head => "Head",
            Self::Weapons => "Weapons",
            Self::Gems => "Gems",
            Self::Rings => "Rings",
        }
    }

    /// The page last chosen, or Gems if none was, or it isn't one any more.
    pub(super) fn chosen(preferences: &Preferences) -> Self {
        let name = preferences.item_pictures.as_deref();
        Self::ALL
            .into_iter()
            .find(|page| Some(page.name()) == name)
            .unwrap_or_default()
    }

    /// This page's rows.
    pub(super) fn spawn(self, grid: &mut ChildSpawnerCommands) {
        match self {
            Self::Bodies => spawn_body_row(grid),
            Self::Hands => armour::spawn_hands_rows(grid),
            Self::Shoulders => armour::spawn_shoulders_rows(grid),
            Self::Arms => arms::spawn_arms_rows(grid),
            Self::Legs => legs::spawn_legs_rows(grid),
            Self::Feet => feet::spawn_feet_rows(grid),
            Self::Footwear => footwear::spawn_footwear_rows(grid),
            Self::Neck => neck::spawn_neck_rows(grid),
            Self::Sets => sets::spawn_set_rows(grid),
            Self::Waist => waist::spawn_waist_rows(grid),
            Self::Torso => armour::spawn_torso_rows(grid),
            Self::Head => armour::spawn_head_rows(grid),
            Self::Weapons => Family::ALL
                .into_iter()
                .for_each(|f| spawn_blade_row(grid, f)),
            Self::Gems => {
                spawn_picture_heading(grid);
                for material in Material::GEMS {
                    spawn_picture_row(grid, material);
                }
            }
            Self::Rings => (0..RING_ROWS).for_each(|row| spawn_ring_row(grid, row)),
        }
    }
}
