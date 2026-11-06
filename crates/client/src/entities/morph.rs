//! A token whose creature changes its kind shows it changing: a few pictures of its
//! body blending into the new one, each `STEP` long, then its new picture. Worn
//! things aren't drawn while it changes; they come back on the new body.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::sheet::Appearance;

use crate::common::IconPicture;
use crate::item_art::{ItemPictures, MORPH_STEPS, morph_look};

use super::dressed::{Drawing, PIXELS};

/// How long each picture of the change shows: the whole change takes about a second.
const STEP: f64 = 1.0 / MORPH_STEPS as f64;
/// The longest a change waits for its pictures to be drawn before giving up on them.
const MOST_WAIT: f64 = 3.0;

/// How another creature's token was last seen to look, so a change of kind can be
/// shown from it.
#[derive(Component)]
pub(super) struct SeenLooks(pub(super) Appearance);

/// A token changing from one body into another.
#[derive(Component)]
pub(super) struct Morphing {
    steps: Vec<Handle<Image>>,
    /// What it shows once changed, `None` for its tile.
    pub(super) then: Option<IconPicture>,
    /// When it was asked for, and when its first picture showed.
    asked: f64,
    started: Option<f64>,
    shown: Option<usize>,
}

impl Drawing<'_> {
    /// The change from one creature to another, ending on `then`, if both have
    /// bodies to draw. Every picture is asked for at once, so they are drawn together.
    pub(super) fn morph(
        &mut self,
        from: (CreatureKind, &Appearance),
        to: (CreatureKind, &Appearance),
        then: Option<IconPicture>,
        now: f64,
    ) -> Option<Morphing> {
        let steps = (1..MORPH_STEPS)
            .map(|step| {
                let look = morph_look(from, to, step)?;
                Some(self.pictures.picture(look, PIXELS, &mut self.images))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Morphing {
            steps,
            then,
            asked: now,
            started: None,
            shown: None,
        })
    }
}

/// Shows each token's change a step at a time, starting once its first picture is
/// drawn, then its new picture once that is drawn too.
pub(super) fn play_morphs(
    time: Res<Time>,
    pictures: Res<ItemPictures>,
    mut tokens: Query<(Entity, &mut Morphing)>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    for (entity, mut morph) in &mut tokens {
        let gave_up = now - morph.asked > MOST_WAIT;
        let started = match morph.started {
            Some(started) => started,
            None if pictures.is_drawn(&morph.steps[0]) => *morph.started.insert(now),
            None if gave_up => now - STEP * morph.steps.len() as f64,
            None => continue,
        };
        let step = ((now - started) / STEP) as usize;
        let then_drawn = morph
            .then
            .as_ref()
            .is_none_or(|picture| pictures.is_drawn(&picture.0));
        if step >= morph.steps.len() && (then_drawn || gave_up) {
            let mut token = commands.entity(entity);
            match morph.then.take() {
                Some(picture) => token.insert(picture),
                None => token.remove::<IconPicture>(),
            };
            token.remove::<Morphing>();
            continue;
        }
        // The latest step due that has been drawn.
        let due = step.min(morph.steps.len() - 1);
        let ready = (0..=due)
            .rev()
            .find(|&i| pictures.is_drawn(&morph.steps[i]));
        if let Some(ready) = ready
            && morph.shown != Some(ready)
        {
            morph.shown = Some(ready);
            commands
                .entity(entity)
                .insert(IconPicture(morph.steps[ready].clone()));
        }
    }
}
