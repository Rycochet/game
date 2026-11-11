//! The right-hand column: the chosen character's name, what they are like (as the
//! Character tab says), when they were made, and Play and Delete only while one is
//! chosen.

use bevy::prelude::*;

use super::chosen::Chosen;
use super::screen::Button;
use crate::character::describe_listed;
use crate::common::{CharacterSelect, local_date_time};
use crate::dom::Dom;
use crate::widgets::ButtonState;

pub(super) fn show_info(
    select: Res<CharacterSelect>,
    chosen: Res<Chosen>,
    mut dom: Dom,
    mut buttons: Query<(&Button, &mut ButtonState)>,
) {
    if !select.is_changed() && !chosen.is_changed() {
        return;
    }
    let listed = chosen
        .id
        .and_then(|id| select.list.iter().find(|c| c.summary.id == id));
    let (name, words, created) = match listed {
        Some(listed) => (
            listed.summary.name.clone(),
            describe_listed(listed),
            format!("Created {}", local_date_time(listed.summary.created)),
        ),
        None if select.can_create => (
            String::new(),
            "No characters yet: press + to make one.".to_owned(),
            String::new(),
        ),
        None => Default::default(),
    };
    dom.set_text("character_select/name", name);
    dom.set_text("character_select/words", words);
    dom.set_text("character_select/created", created);
    for (button, mut state) in &mut buttons {
        let usable = *button == Button::Cancel || listed.is_some();
        state.set_if_neq(if usable {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}
