//! Suggesting names: one when the screen opens, a new one for another kind if the
//! player hasn't typed their own, and another whenever the random button is
//! pressed. Only a suggestion: the server checks whatever name is sent.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::rng::{Rng, Seed};

use super::draft::Draft;
use super::screen::{NameBox, RandomName};
use crate::common::MakingCharacter;
use crate::widgets::{ButtonPressed, TextBox};

/// Where suggestions come from: seeded from the clock once, then rolling on.
#[derive(Resource, Default)]
pub(super) struct NameRolls(Option<Rng>);

impl NameRolls {
    fn next(&mut self, kind: CreatureKind, time: &Time<Real>) -> String {
        let rng = self
            .0
            .get_or_insert_with(|| Seed::new(time.elapsed().as_nanos() as u64).rng());
        kind.random_name(rng)
    }
}

/// Whether the name is still the last one suggested (or empty), so a new kind
/// may change it.
fn untouched(draft: &Draft) -> bool {
    draft.name.is_empty() || draft.name == draft.suggested
}

#[allow(clippy::too_many_arguments)]
pub(super) fn suggest_names(
    making: Res<MakingCharacter>,
    mut pressed: MessageReader<ButtonPressed>,
    button: Single<Entity, With<RandomName>>,
    mut name_box: Single<&mut TextBox, With<NameBox>>,
    mut draft: ResMut<Draft>,
    mut rolls: ResMut<NameRolls>,
    mut named_for: Local<Option<CreatureKind>>,
    time: Res<Time<Real>>,
) {
    let opened = making.is_changed() && making.0;
    let new_kind = *named_for != Some(draft.kind) && untouched(&draft);
    let asked = pressed.read().any(|press| press.0 == *button);
    if !(opened || new_kind || asked) {
        return;
    }
    *named_for = Some(draft.kind);
    let name = rolls.next(draft.kind, &time);
    draft.suggested = name.clone();
    draft.name = name.clone();
    name_box.text = name;
}
