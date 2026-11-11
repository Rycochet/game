//! `input type="text"`, `input type="range"` and `key-box`.

use bevy::prelude::*;

use super::state_of;
use crate::common::{Binding, Mods};
use crate::dom::attrs::{Attrs, Length};
use crate::dom::build::Builder;
use crate::dom::pictures::picture_for;
use crate::widgets::{KeyBox, TextBox, TextSize, spawn_key_box, spawn_slider, spawn_text_box};

/// A slider's width, with none given.
const SLIDER: f32 = 160.0;

/// Where a slider's `value` is between its `min` and `max`, from 0 to 1.
pub(super) fn share_of(attrs: &Attrs) -> f32 {
    let (least, most) = range_of(attrs);
    let value = attrs.number("value").unwrap_or(least);
    ((value - least) / (most - least).max(f32::EPSILON)).clamp(0.0, 1.0)
}

/// The value at a share of the way along a slider, on one of its `step`s.
pub(super) fn value_at(attrs: &Attrs, share: f32) -> f32 {
    let (least, most) = range_of(attrs);
    let value = least + share.clamp(0.0, 1.0) * (most - least);
    match attrs.number("step").filter(|step| *step > 0.0) {
        Some(step) => (least + ((value - least) / step).round() * step).min(most),
        None => value,
    }
}

/// A slider's `min` and `max`: 0 to 100 unless it says, as in HTML.
fn range_of(attrs: &Attrs) -> (f32, f32) {
    let least = attrs.number("min").unwrap_or(0.0);
    (least, attrs.number("max").unwrap_or(100.0).max(least))
}

/// A text box as its element's attributes have it.
pub(super) fn text_box_of(attrs: &Attrs) -> TextBox {
    TextBox {
        text: attrs.text("value").unwrap_or_default().to_owned(),
        prefix: attrs.static_text("prefix").unwrap_or_default(),
        digits_only: attrs.text("inputmode") == Some("numeric"),
        takes_keys: true,
        error: attrs.class.as_deref() == Some("error"),
        suffix: attrs.text("suffix").unwrap_or_default().to_owned(),
        accepts_items: attrs.has("accepts-items"),
    }
}

/// A key box as its element's attributes have it: `value` is the key's
/// saved name (`Shift+KeyG`).
pub(super) fn key_box_of(attrs: &Attrs) -> KeyBox {
    // Esc can never be bound, so it has no saved name, but a box may show it.
    let named = |name: &str| match name {
        "Escape" => Some(Binding::new(KeyCode::Escape, Mods::NONE)),
        _ => Binding::from_name(name),
    };
    KeyBox {
        binding: attrs.text("value").and_then(named),
        warned: attrs.class.as_deref() == Some("warned"),
    }
}

impl Builder<'_, '_, '_> {
    pub(super) fn text_box(
        &mut self,
        holder: Entity,
        attrs: &Attrs,
        size: TextSize,
        node: Node,
    ) -> Entity {
        let text_box = text_box_of(attrs);
        self.spawn_in(holder, |inside| {
            spawn_text_box(inside, text_box, size, node).id()
        })
    }

    /// A slider: `src` gives its track a picture.
    pub(super) fn range(&mut self, holder: Entity, attrs: &Attrs) -> Entity {
        let track = attrs.text("src");
        let track = track.and_then(|src| picture_for(src, self.pictures, self.assets));
        let track = track.map(|(picture, _)| picture);
        let width = match attrs.width {
            Some(Length::Pixels(pixels)) => pixels,
            _ => SLIDER,
        };
        let share = share_of(attrs);
        self.spawn_in(holder, |inside| {
            spawn_slider(inside, share, track, width).id()
        })
    }

    pub(super) fn key_box(
        &mut self,
        holder: Entity,
        attrs: &Attrs,
        size: TextSize,
        node: Node,
    ) -> Entity {
        let (key_box, state) = (key_box_of(attrs), state_of(attrs));
        self.spawn_in(holder, |inside| {
            spawn_key_box(inside, key_box, state, size, node).id()
        })
    }
}
