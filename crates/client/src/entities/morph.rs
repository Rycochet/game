//! A token whose creature changes its kind shows it changing: a few models of its
//! body blending into the new one, each `STEP` long, then its new model. Worn things
//! aren't drawn while it changes; they come back on the new body.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::sheet::Appearance;

use crate::common::TokenModel;
use crate::item_art::{MORPH_STEPS, morph_look};

use super::dressed::Drawing;

/// How long each model of the change shows: the whole change takes about a second.
const STEP: f64 = 1.0 / MORPH_STEPS as f64;

/// How another creature's token was last seen to look, so a change of kind can be
/// shown from it.
#[derive(Component)]
pub(super) struct SeenLooks(pub(super) Appearance);

/// A token changing from one body into another.
#[derive(Component)]
pub(super) struct Morphing {
    steps: Vec<TokenModel>,
    /// What it shows once changed, `None` for its tile.
    pub(super) then: Option<TokenModel>,
    /// When it started, and the step shown.
    started: f64,
    shown: Option<usize>,
}

impl Drawing<'_> {
    /// The change from one creature to another, ending on `then`, if both have
    /// bodies to draw. Every step is built at once (each look only once ever).
    pub(super) fn morph(
        &mut self,
        from: (CreatureKind, &Appearance),
        to: (CreatureKind, &Appearance),
        then: Option<TokenModel>,
        now: f64,
    ) -> Option<Morphing> {
        let steps = (1..MORPH_STEPS)
            .map(|step| self.model(&morph_look(from, to, step)?))
            .collect::<Option<Vec<_>>>()?;
        Some(Morphing {
            steps,
            then,
            started: now,
            shown: None,
        })
    }
}

/// Shows each token's change a step at a time, then its new model.
pub(super) fn play_morphs(
    time: Res<Time>,
    mut tokens: Query<(Entity, &mut Morphing)>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    for (entity, mut morph) in &mut tokens {
        let step = ((now - morph.started) / STEP) as usize;
        if step >= morph.steps.len() {
            let mut token = commands.entity(entity);
            match morph.then.take() {
                Some(model) => token.insert(model),
                None => token.remove::<TokenModel>(),
            };
            token.remove::<Morphing>();
            continue;
        }
        if morph.shown != Some(step) {
            morph.shown = Some(step);
            commands.entity(entity).insert(morph.steps[step].clone());
        }
    }
}
