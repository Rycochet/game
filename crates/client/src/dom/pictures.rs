//! What an `img`'s `src` shows: `icon:<name>` (an icon from `assets/icons`,
//! drawn in the colour of the writing round it), `picture:<name>` (a picture
//! made in code and given a name here), or the path of a picture in `assets`.

use std::collections::HashMap;

use bevy::prelude::*;

/// Pictures made in code, by the names markup shows them by.
#[derive(Resource, Default)]
pub struct Pictures(HashMap<String, Handle<Image>>);

impl Pictures {
    /// Names a picture, for `<img src="picture:<name>"/>`.
    pub fn name(&mut self, name: impl Into<String>, picture: Handle<Image>) {
        self.0.insert(name.into(), picture);
    }
}

/// The picture `src` asks for, and whether it takes the colour of writing.
pub(super) fn picture_for(
    src: &str,
    pictures: &Pictures,
    assets: &AssetServer,
) -> Option<(Handle<Image>, bool)> {
    if let Some(icon) = src.strip_prefix("icon:") {
        Some((assets.load(format!("icons/{icon}.png")), true))
    } else if let Some(name) = src.strip_prefix("picture:") {
        let found = pictures.0.get(name).cloned();
        if found.is_none() {
            warn!("no picture is named `{name}`");
        }
        found.map(|picture| (picture, false))
    } else {
        Some((assets.load(src.to_owned()), false))
    }
}
