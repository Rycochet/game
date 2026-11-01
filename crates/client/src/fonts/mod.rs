//! The game's default font. See `docs/context/client/fonts.md`.

use bevy::asset::uuid_handle;
use bevy::prelude::*;

/// Lexend, every weight in one file (a variable font). Built into the program
/// so that text never waits for it, not even on the first frame.
const DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/Lexend-Variable.ttf");

/// Noto Emoji, for the symbols Lexend doesn't have. Built in for the same reason.
const ICON_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/NotoEmoji.ttf");

const ICON_FONT: Handle<Font> = uuid_handle!("5b0f7a52-3c1e-4a86-9d27-6e4c1f8a2b90");

/// The default font, falling back to symbols for what it doesn't have.
pub fn with_icons() -> FontSource {
    FontSource::list([Handle::<Font>::default(), ICON_FONT])
}

pub struct FontsPlugin;

impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        // Text with no font of its own uses the font at the default id. Bevy
        // puts its own there while `DefaultPlugins` is built, and only reads
        // fonts once the app runs, so replacing it now means it is never used.
        let mut fonts = app.world_mut().resource_mut::<Assets<Font>>();
        fonts
            .insert(
                AssetId::default(),
                Font::from_bytes(DEFAULT_FONT_BYTES.to_vec()),
            )
            .expect("the default font id is always valid");
        fonts
            .insert(&ICON_FONT, Font::from_bytes(ICON_FONT_BYTES.to_vec()))
            .expect("the icon font's id is always valid");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexend_replaces_whatever_font_was_the_default() {
        let mut app = App::new();
        let mut fonts = Assets::<Font>::default();
        fonts
            .insert(AssetId::default(), Font::from_bytes(vec![1, 2, 3]))
            .unwrap();
        app.insert_resource(fonts);

        app.add_plugins(FontsPlugin);

        let fonts = app.world().resource::<Assets<Font>>();
        let font = fonts.get(AssetId::default()).unwrap();
        assert_eq!(font.data.data(), DEFAULT_FONT_BYTES);
    }
}
