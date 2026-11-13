//! The Gender toggle row: Male, Female or Neutral, into the draft. Its bonus
//! shows in the triangles' gold layer and the values below the choices. Female
//! has no beard: the looks are made again without its list.

use bevy::prelude::*;
use shared::gender::Gender;

use super::draft::Draft;
use super::screen::FORM;
use crate::dom::{UiEvent, UiValue};

/// The toggle row's name in the form.
const GENDER: &str = "gender";

/// Takes the gender chosen into the draft.
pub(super) fn choose_gender(mut events: MessageReader<UiEvent>, mut draft: ResMut<Draft>) {
    for event in events.read() {
        if event.id.parts() != (FORM, GENDER) {
            continue;
        }
        if let UiValue::Choice { index, .. } = event.value
            && let Some(&gender) = Gender::ALL.get(index)
            && draft.gender != gender
        {
            draft.choose_gender(gender);
        }
    }
}
