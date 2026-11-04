//! Tokens of characters wearing something drawn show a picture of them dressed in
//! place of their icon; with nothing drawn on, they keep the icon.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::wear::WornLook;

use crate::common::IconPicture;
use crate::inventory::Carried;
use crate::item_art::{ItemPictures, character_look};
use crate::player::Player;

/// How many pixels a dressed token's picture has on a side.
const PIXELS: u32 = 128;

/// What drawing a dressed picture needs.
#[derive(SystemParam)]
pub(super) struct Drawing<'w> {
    pictures: ResMut<'w, ItemPictures>,
    images: ResMut<'w, Assets<Image>>,
}

impl Drawing<'_> {
    /// The picture of a creature of this kind wearing these, if any is drawn.
    pub(super) fn dressed(&mut self, kind: CreatureKind, worn: &[WornLook]) -> Option<IconPicture> {
        let look = character_look(kind, worn)?;
        Some(IconPicture(self.pictures.picture(
            look,
            PIXELS,
            &mut self.images,
        )))
    }
}

/// The player's own token shows them as they are dressed, whenever that changes.
pub(super) fn show_own_dress(
    carried: Res<Carried>,
    mut drawing: Drawing,
    mut drawn: Local<Option<(CreatureKind, Vec<WornLook>)>>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    if !carried.is_changed() && drawn.is_some() {
        return;
    }
    let kind = carried.0.creature;
    let worn = carried.0.shown();
    if drawn.as_ref() == Some(&(kind, worn.clone())) {
        return;
    }
    match drawing.dressed(kind, &worn) {
        Some(picture) => commands.entity(*player).insert(picture),
        None => commands.entity(*player).remove::<IconPicture>(),
    };
    *drawn = Some((kind, worn));
}
