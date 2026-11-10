//! What the screen's presses do: a cell chooses its character (or "+" goes on to
//! making one), Play plays the chosen one, Delete asks first, Cancel goes back
//! to the startup menu, and Change User logs in again.

use bevy::prelude::*;

use super::cells::Cell;
use super::chosen::Chosen;
use super::requester::Answer;
use super::screen::Button;
use crate::common::{CharacterChoice, LoginChoice};
use crate::widgets::ButtonPressed;

pub(super) fn press_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    cells: Query<&Cell>,
    buttons: Query<&Button>,
    answers: Query<&Answer>,
    mut chosen: ResMut<Chosen>,
    mut choices: MessageWriter<CharacterChoice>,
    mut login: MessageWriter<LoginChoice>,
) {
    for press in pressed.read() {
        let entity = press.0;
        if chosen.deleting {
            // Only the requester's answers while it asks.
            match answers.get(entity) {
                Ok(Answer::Delete) => {
                    if let Some(id) = chosen.id {
                        choices.write(CharacterChoice::Delete(id));
                    }
                    chosen.deleting = false;
                }
                Ok(Answer::Keep) => chosen.deleting = false,
                Err(_) => {}
            }
            continue;
        }
        if let Ok(cell) = cells.get(entity) {
            match *cell {
                Cell::Character(id) if chosen.id != Some(id) => chosen.id = Some(id),
                Cell::Character(_) => {}
                Cell::New => {
                    choices.write(CharacterChoice::New);
                }
            }
            continue;
        }
        match (buttons.get(entity), chosen.id) {
            (Ok(Button::Play), Some(id)) => {
                choices.write(CharacterChoice::Play(id));
            }
            (Ok(Button::Delete), Some(_)) => chosen.deleting = true,
            (Ok(Button::Cancel), _) => {
                choices.write(CharacterChoice::Cancel);
            }
            (Ok(Button::ChangeUser), _) => {
                login.write(LoginChoice::ChangeUser);
            }
            _ => {}
        }
    }
}
