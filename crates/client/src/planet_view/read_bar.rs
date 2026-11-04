//! What the bar's controls change: another seed or generator builds the globe
//! again; another layer only colours the picture again.

use bevy::prelude::*;
use shared::rng::Seed;
use shared::surface::SurfaceGenerator;

use super::layer::Layer;
use super::showing::{Choice, PlanetView, SceneAssets, build_globe};
use super::texture::Fill;
use crate::common::CurrentWorld;
use crate::widgets::{ButtonPressed, Dropdown, DropdownChosen, TextBox, TextEdited, TextFocus};

#[allow(clippy::too_many_arguments)]
pub(super) fn read_bar(
    mut view: ResMut<PlanetView>,
    world: Option<Res<CurrentWorld>>,
    mut chosen: MessageReader<DropdownChosen>,
    mut edited: MessageReader<TextEdited>,
    mut pressed: MessageReader<ButtonPressed>,
    time: Res<Time<Real>>,
    mut boxes: Query<&mut TextBox>,
    mut dropdowns: Query<&mut Dropdown>,
    focus: Res<TextFocus>,
    mut commands: Commands,
    mut assets: SceneAssets,
) {
    let (Some(showing), Some(world)) = (view.showing.as_mut(), world) else {
        return;
    };
    let bar = &showing.bar;
    showing.typing = focus.0 == Some(bar.seed);
    let mut choice = showing.choice;
    for &DropdownChosen(entity, index) in chosen.read() {
        if entity == bar.generator {
            choice.generator = SurfaceGenerator::ALL[index];
        } else if entity == bar.layer {
            choice.layer = Layer::ALL[index];
        }
    }
    for &TextEdited(entity) in edited.read() {
        if entity == bar.seed
            && let Ok(mut seed_box) = boxes.get_mut(entity)
        {
            let typed = seed_box.text.parse::<u64>().ok();
            seed_box.error = typed.is_none();
            if let Some(typed) = typed {
                choice.seed = Seed::new(typed);
            }
        }
    }
    let presses: Vec<Entity> = pressed.read().map(|press| press.0).collect();
    if presses.contains(&bar.random) {
        // Any seed at all, as worlds have: mixed from the time, which keeps changing.
        choice.seed = Seed::new(Seed::new(time.elapsed().as_nanos() as u64).at(&[0]));
        if let Ok(mut seed_box) = boxes.get_mut(bar.seed) {
            seed_box.text = choice.seed.value().to_string();
            seed_box.error = false;
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if presses.contains(&bar.copy) {
        crate::widgets::copy_to_clipboard(&choice.seed.value().to_string());
    }
    if presses.contains(&bar.reset) {
        choice = Choice {
            layer: choice.layer,
            ..Choice::of(&world)
        };
        if let Ok(mut seed_box) = boxes.get_mut(bar.seed) {
            seed_box.text = choice.seed.value().to_string();
            seed_box.error = false;
        }
        if let Ok(mut generator) = dropdowns.get_mut(bar.generator) {
            generator.selected = 0;
        }
    }
    showing.choice = choice;
    let globe = &mut showing.globe;
    if (choice.seed, choice.generator) != globe.built {
        commands.entity(globe.scene.root).despawn();
        let picture = showing.picture.clone();
        *globe = build_globe(&mut commands, &mut assets, &world, choice, picture);
    } else if choice.layer != globe.layer {
        globe.layer = choice.layer;
        globe.fill = Fill::default();
    }
}
