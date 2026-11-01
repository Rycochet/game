//! The game's default font. See `docs/context/client/fonts.md`.

use bevy::prelude::*;

/// Lexend, every weight in one file (a variable font). Built into the program
/// so that text never waits for it, not even on the first frame.
const DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/Lexend-Variable.ttf");

pub struct FontsPlugin;

impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        // Text with no font of its own uses the font at the default id. Bevy
        // puts its own there while `DefaultPlugins` is built, and only reads
        // fonts once the app runs, so replacing it now means it is never used.
        app.world_mut()
            .resource_mut::<Assets<Font>>()
            .insert(
                AssetId::default(),
                Font::from_bytes(DEFAULT_FONT_BYTES.to_vec()),
            )
            .expect("the default font id is always valid");
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
