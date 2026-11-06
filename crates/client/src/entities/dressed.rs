//! Tokens of kinds standing on two legs show a picture of their body, dressed in
//! whatever is drawn, in place of their icon; other kinds keep the icon.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::sheet::Appearance;
use shared::wear::WornLook;

use crate::character::CharacterSheet;
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
    /// The picture of a creature of this kind, looking like this, wearing these, if
    /// its kind has a body to draw.
    pub(super) fn dressed(
        &mut self,
        kind: CreatureKind,
        appearance: &Appearance,
        worn: &[WornLook],
    ) -> Option<IconPicture> {
        let look = character_look(kind, appearance, worn)?;
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
    sheet: Res<CharacterSheet>,
    mut drawing: Drawing,
    mut drawn: Local<Option<(CreatureKind, Appearance, Vec<WornLook>)>>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    if !carried.is_changed() && !sheet.is_changed() && drawn.is_some() {
        return;
    }
    let kind = carried.0.creature;
    let appearance = sheet.0.appearance.clone();
    let worn = carried.0.shown();
    let now = (kind, appearance, worn);
    if drawn.as_ref() == Some(&now) {
        return;
    }
    match drawing.dressed(now.0, &now.1, &now.2) {
        Some(picture) => commands.entity(*player).insert(picture),
        None => commands.entity(*player).remove::<IconPicture>(),
    };
    *drawn = Some(now);
}
