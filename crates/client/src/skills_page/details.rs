//! The line under the tree: about the chosen skill, or how to use the page.

use bevy::prelude::*;
use shared::skill::{SkillId, Skills};

use super::build::tenths;
use super::spawn::Details;
use super::view::SkillsView;
use crate::character::CharacterSheet;

pub(super) fn show_details(
    view: Res<SkillsView>,
    sheet: Res<CharacterSheet>,
    mut texts: Query<&mut Text, With<Details>>,
) {
    if !view.shown {
        return;
    }
    let words = describe(&sheet.0.skills, view.hovered.or(view.chosen));
    for mut text in &mut texts {
        if text.0 != words {
            text.0 = words.clone();
        }
    }
}

/// What the line says.
pub(super) fn describe(skills: &Skills, chosen: Option<SkillId>) -> String {
    if skills.is_empty() {
        return "Nothing learned yet: skills are learned by doing.".into();
    }
    let Some(id) = chosen.filter(|&id| skills.knows(id)) else {
        return "Point at or choose a skill to see more. Drag to move the tree; the wheel or a pinch zooms."
            .into();
    };
    let mut path: Vec<_> = id.upwards().map(SkillId::name).collect();
    path.reverse();
    let leans: Vec<_> = id.leans().iter().map(|which| which.name()).collect();
    format!(
        "{}: {} ({} in checks), from {} practice. Leans on {}.",
        path.join(" › "),
        tenths(skills.value(id)),
        tenths(skills.total(id)),
        skills.practice(id),
        leans.join(" and "),
    )
}

#[cfg(test)]
mod tests;
