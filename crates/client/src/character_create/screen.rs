//! The screen, as markup: a full-screen layer with the choices on the left
//! and the triangles on the right, and above it the requester asking before
//! Ironman is chosen. Built once and shown while a character is made.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::sheet::Attribute;

use super::difficulty::{DifficultyBox, IronmanAnswer, IronmanRequester};
use super::picture::PIXELS;
use crate::dom::{Template, document};
use crate::html;
use crate::widgets::{INK, Pressable, TextSize, UiId};

/// The form all its widgets' ids are in.
pub(super) const FORM: &str = "character_create";
/// How big the triangles are drawn, in interface pixels.
pub(super) const CHOOSER_SIDE: f32 = 480.0;

/// The kind's looks, the difficulty's dropdown and the triangles are drawn
/// by code, in the canvases.
pub(super) static CHARACTER_CREATE: Template = html!(
    "character_create",
    r#"
<form id="character_create">
  <dialog id="layer" modal depth="0" width="90%" height="90%" hidden>
    <span class="apart" nowrap align="center" valign="middle" fills>
      <div width="300">
        <h1>New character</h1>
        <div class="plain">
          <label class="field" for="name">Name</label>
          <span nowrap valign="middle">
            <input id="name" type="text"/>
            <button id="random_name" title="Random name"><img src="picture:new_seed"/></button>
          </span>
          <label class="field" for="kind">Kind</label>
          <select id="kind"><option>Human</option></select>
          <label class="field" for="gender">Gender</label>
          <toggle-row id="gender">
            <option>Male</option>
            <option>Female</option>
            <option selected>Neutral</option>
          </toggle-row>
          <canvas id="looks"/>
          <label class="field">Difficulty</label>
          <canvas id="difficulty_box"/>
          <small id="values" align="left"/>
        </div>
        <span nowrap>
          <button id="begin" class="main" width="148" disabled>Begin</button>
          <button id="cancel" width="148">Cancel</button>
        </span>
      </div>
      <canvas id="triangles" width="480" height="480"/>
    </span>
  </dialog>
  <dialog id="ironman" title="Ironman" depth="1" hidden>
    <p id="ironman_question" class="medium" width="420"/>
    <footer>
      <button id="ironman_yes" class="main">Yes, Ironman</button>
      <button id="ironman_no">No</button>
    </footer>
  </dialog>
</form>
"#
);

#[derive(Component)]
pub(super) struct NameBox;
#[derive(Component)]
pub(super) struct KindList;
/// The button beside the name box suggesting another.
#[derive(Component)]
pub(super) struct RandomName;
/// Holds a list for each of the kind's looks, filled again for each kind.
#[derive(Component)]
pub(super) struct LooksBox;
#[derive(Component)]
pub(super) struct BeginButton;
/// Back to where the player came from, making nothing.
#[derive(Component)]
pub(super) struct CancelButton;
/// The picture of the triangles, which takes presses on them.
#[derive(Component)]
pub(super) struct Chooser(pub Handle<Image>);
/// A short name by its corner of a triangle.
#[derive(Component)]
pub(super) struct CornerLabel(pub Attribute);

pub(super) fn spawn_screen(mut commands: Commands) {
    commands.spawn(document(&CHARACTER_CREATE));
}

/// A column for what code spawns into a canvas, its things a little apart.
fn column() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        ..default()
    }
}

/// Gives the built parts what the screen's systems know them by, found by
/// their ids, and fills the triangles' canvas.
pub(super) fn bind_screen(
    mut commands: Commands,
    new: Query<(Entity, &UiId), Added<UiId>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, id) in &new {
        let (FORM, name) = id.parts() else {
            continue;
        };
        let mut bound = commands.entity(entity);
        match name {
            "name" => bound.insert(NameBox),
            "random_name" => bound.insert(RandomName),
            "kind" => bound.insert(KindList),
            "looks" => bound.with_child((LooksBox, column())),
            "difficulty_box" => bound.with_child((DifficultyBox, Node::default())),
            "begin" => bound.insert(BeginButton),
            "cancel" => bound.insert(CancelButton),
            "ironman" => bound.insert(IronmanRequester),
            "ironman_yes" => bound.insert(IronmanAnswer(true)),
            "ironman_no" => bound.insert(IronmanAnswer(false)),
            "triangles" => {
                let picture = images.add(blank_picture());
                let chooser = (Chooser(picture.clone()), ImageNode::new(picture), Pressable);
                bound.insert(chooser).with_children(spawn_corner_labels)
            }
            _ => continue,
        };
    }
}

/// The triangles' picture before it is painted: clear.
fn blank_picture() -> Image {
    Image::new_fill(
        Extent3d {
            width: PIXELS,
            height: PIXELS,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// The corners' names over the picture, placed as the triangles turn.
fn spawn_corner_labels(chooser: &mut ChildSpawnerCommands) {
    for which in Attribute::ALL {
        chooser.spawn((
            CornerLabel(which),
            Text::new(&which.name()[..3]),
            TextSize::Small.font(),
            TextColor(INK),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_screen_is_good_markup() {
        super::CHARACTER_CREATE.assert_good();
    }
}
