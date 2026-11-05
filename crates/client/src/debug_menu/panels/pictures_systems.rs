//! What Item Pictures' dropdown, tabs and pictures do, and filling its grid.

use bevy::prelude::*;
use shared::item::Quality;

use super::page::PicturePage;
use super::{
    BLADE_PICTURE, GemPicture, HANDS_PICTURE, LARGE_PICTURE, LARGE_PICTURE_PIXELS, LargePicture,
    PICTURE, PICTURE_PIXELS, PicturePages, PictureQualities, PicturesForm, PicturesGrid, Shown,
};
use crate::common::Preferences;
use crate::item_art::ItemPictures;
use crate::widgets::{ButtonPressed, Dropdown, DropdownChosen, TabChosen};

/// What Item Pictures' page dropdown, quality tabs and pictures do.
#[allow(clippy::too_many_arguments)]
pub(in crate::debug_menu) fn choose_pictures(
    mut chosen: MessageReader<TabChosen>,
    mut pages: MessageReader<DropdownChosen>,
    mut pressed: MessageReader<ButtonPressed>,
    rows: Query<(), With<PictureQualities>>,
    dropdowns: Query<(), With<PicturePages>>,
    pictures: Query<&GemPicture>,
    mut form: ResMut<PicturesForm>,
    mut preferences: ResMut<Preferences>,
) {
    for &TabChosen(row, index) in chosen.read() {
        if rows.contains(row) {
            form.quality = Quality::ALL[index];
        }
    }
    for &DropdownChosen(dropdown, index) in pages.read() {
        if !dropdowns.contains(dropdown) {
            continue;
        }
        let name = PicturePage::ALL[index].name();
        if preferences.item_pictures.as_deref() != Some(name) {
            preferences.item_pictures = Some(name.to_owned());
        }
    }
    for press in pressed.read() {
        if let Ok(&GemPicture(shown)) = pictures.get(press.0) {
            form.chosen = shown;
        }
    }
}

/// Fills Item Pictures' grid with the page chosen, and shows its name in the dropdown.
/// Only that page's pictures are drawn.
pub(in crate::debug_menu) fn fill_pictures(
    mut commands: Commands,
    preferences: Res<Preferences>,
    mut grids: Query<(Entity, &mut PicturesGrid)>,
    mut dropdowns: Query<&mut Dropdown, With<PicturePages>>,
) {
    let page = PicturePage::chosen(&preferences);
    for (entity, mut grid) in &mut grids {
        if grid.0 != Some(page) {
            grid.0 = Some(page);
            commands
                .entity(entity)
                .despawn_children()
                .with_children(|grid| page.spawn(grid));
        }
    }
    let index = PicturePage::ALL
        .iter()
        .position(|&p| p == page)
        .unwrap_or(0);
    for mut dropdown in &mut dropdowns {
        if dropdown.selected != index {
            dropdown.selected = index;
        }
    }
}

/// Keeps Item Pictures' images to what is chosen.
pub(in crate::debug_menu) fn show_pictures(
    form: Res<PicturesForm>,
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
    mut grid: Query<(Ref<GemPicture>, &mut ImageNode, &mut Node), Without<LargePicture>>,
    mut large: Query<(Ref<LargePicture>, &mut ImageNode, &mut Node), Without<GemPicture>>,
) {
    for (picture, mut image, mut node) in &mut grid {
        if (form.is_changed() || picture.is_added())
            && let Some(look) = picture.0.look(form.quality)
        {
            // A weapon or gauntlets need their full size for the black edge to be right, and
            // more room.
            let (box_side, pixels) = match picture.0 {
                Shown::Armour(_) | Shown::Footwear(_) => (
                    HANDS_PICTURE,
                    look.longest() as u32 * crate::widgets::PICTURE_PIXELS_PER_CELL,
                ),
                Shown::Blade(..) | Shown::Body(..) => (
                    BLADE_PICTURE,
                    look.longest() as u32 * crate::widgets::PICTURE_PIXELS_PER_CELL,
                ),
                _ => (PICTURE, PICTURE_PIXELS),
            };
            let size = look.picture_size(1000).as_vec2() / 1000.0 * box_side;
            node.width = px(size.x);
            node.height = px(size.y);
            node.margin = UiRect::horizontal(px((PICTURE - size.x).max(0.0) / 2.0));
            image.image = pictures.picture(look, pixels, &mut images);
        }
    }
    for (picture, mut image, mut node) in &mut large {
        if (form.is_changed() || picture.is_added())
            && let Some(look) = form.chosen.look(form.quality)
        {
            let size = look.picture_size(1000).as_vec2() / 1000.0 * LARGE_PICTURE;
            node.width = px(size.x);
            node.height = px(size.y);
            node.margin = UiRect::horizontal(px((LARGE_PICTURE - size.x) / 2.0));
            image.image = pictures.picture(look, LARGE_PICTURE_PIXELS, &mut images);
        }
    }
}
