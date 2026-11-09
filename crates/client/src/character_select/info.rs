//! The right-hand column: the chosen character's name, what they are like (as the
//! Character tab says), when they were made, and Play and Delete only while one is
//! chosen.

use bevy::prelude::*;

use super::chosen::Chosen;
use super::screen::{Button, InfoCreated, InfoName, InfoWords};
use crate::character::describe_listed;
use crate::common::{CharacterSelect, local_date_time};
use crate::widgets::ButtonState;

#[allow(clippy::type_complexity)]
pub(super) fn show_info(
    select: Res<CharacterSelect>,
    chosen: Res<Chosen>,
    mut name: Single<&mut Text, (With<InfoName>, Without<InfoWords>, Without<InfoCreated>)>,
    mut words: Single<&mut Text, (With<InfoWords>, Without<InfoName>, Without<InfoCreated>)>,
    mut created: Single<&mut Text, (With<InfoCreated>, Without<InfoName>, Without<InfoWords>)>,
    mut buttons: Query<(&Button, &mut ButtonState)>,
) {
    if !select.is_changed() && !chosen.is_changed() {
        return;
    }
    let listed = chosen
        .id
        .and_then(|id| select.list.iter().find(|c| c.summary.id == id));
    let (now_name, now_words, now_created) = match listed {
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
    name.0 = now_name;
    words.0 = now_words;
    created.0 = now_created;
    for (button, mut state) in &mut buttons {
        let usable = *button == Button::Cancel || listed.is_some();
        state.set_if_neq(if usable {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
}
