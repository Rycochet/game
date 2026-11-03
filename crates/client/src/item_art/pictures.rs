//! `ItemPictures`: the pictures drawn so far, and `ItemPicture`, one on screen.

use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use super::Look;

/// The pictures made or being made. Ask for one with `picture`: it is empty for the
/// frames until it has been drawn.
#[derive(Resource)]
pub struct ItemPictures {
    /// By the pixels they were drawn with.
    made: HashMap<(Look, u32), Handle<Image>>,
    pub(super) waiting: VecDeque<Waiting>,
    /// How many times over the pixels asked for they are drawn with, so they stay
    /// sharp on a large interface.
    pub(super) detail: u32,
    /// How many pictures have been drawn so far, and the time spent building their
    /// models.
    pub(super) drawn: u64,
    pub(super) building: Duration,
}

impl Default for ItemPictures {
    fn default() -> Self {
        Self {
            made: HashMap::default(),
            waiting: VecDeque::default(),
            detail: 1,
            drawn: 0,
            building: Duration::ZERO,
        }
    }
}

impl ItemPictures {
    /// The picture of this, this many pixels along its longest side (more, on a
    /// large interface).
    pub fn picture(&mut self, look: Look, asked: u32, images: &mut Assets<Image>) -> Handle<Image> {
        let pixels = asked * self.detail;
        if let Some(picture) = self.made.get(&(look.clone(), pixels)) {
            return picture.clone();
        }
        let size = look.picture_size(pixels);
        let picture = images.add(Image::new_target_texture(
            size.x,
            size.y,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        self.made.insert((look.clone(), pixels), picture.clone());
        self.waiting.push_back(Waiting {
            look,
            asked,
            pixels,
            picture: picture.clone(),
        });
        picture
    }
}

/// A picture still to be drawn.
pub(super) struct Waiting {
    pub(super) look: Look,
    /// The pixels asked for, which the black edge goes by, and those it is drawn with.
    pub(super) asked: u32,
    pub(super) pixels: u32,
    pub(super) picture: Handle<Image>,
}

/// On an `ImageNode`: it shows this picture once there is one to show.
#[derive(Component, Clone, Debug)]
pub struct ItemPicture {
    pub look: Look,
    /// How many pixels along its longest side it is asked for (`ItemPictures::picture`).
    pub pixels: u32,
}
