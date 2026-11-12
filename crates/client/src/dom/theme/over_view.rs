//! The looks of what stands over the game's view or the dark behind the
//! title, rather than on parchment: the loading screen's bar, the hover box,
//! the status text and popups.

use bevy::prelude::*;

use super::{Look, Theme};
use crate::dom::tag::Tag;
use crate::widgets::{TextSize, tooltip_colours};

/// A row's gap, as `theme`'s.
const ROW: f32 = 4.0;

pub(super) fn add(theme: &mut Theme) {
    // The loading screen's bar: gold on the dark behind the title, with
    // pale writing (its colours are copied in `index.html`).
    let loading = Look {
        text: Some(TextSize::Large),
        colour: Some(Color::srgb(0.95, 0.92, 0.85)),
        bar: Some((
            Color::srgb(0.13, 0.11, 0.16),
            Color::srgb(0.93, 0.75, 0.38).with_alpha(0.45),
        )),
        ..default()
    };
    theme.set(Tag::Progress, "loading", loading);
    // A tooltip's box in the flow: dark, with light writing, a picture and
    // its words a little apart (the hover box).
    let (tip_background, tip_writing) = tooltip_colours();
    let tip = Look {
        tip: true,
        background: Some(tip_background),
        colour: Some(tip_writing),
        text: Some(TextSize::Medium),
        gap: 8.0,
        ..default()
    };
    theme.set(Tag::Span, "tip", tip);
    // A gold picture: a staff member's mark.
    let gold = Look {
        colour: Some(Color::srgb(0.95, 0.75, 0.25)),
        ..default()
    };
    theme.set(Tag::Img, "gold", gold);
    // White writing over the game's view, with symbols: the status text,
    // and in red what it warns of.
    let status = Look {
        colour: Some(Color::WHITE),
        text: Some(TextSize::Large),
        icons: true,
        ..default()
    };
    theme.set(Tag::P, "status", status);
    let warns = Look {
        colour: Some(Color::srgb(1.0, 0.2, 0.2)),
        ..status
    };
    theme.set(Tag::P, "status-warning", warns);
    // A popup's dark box: light writing, its things close.
    let popup = Look {
        gap: ROW,
        // (As a tooltip's.)
        colour: Some(crate::common::NO_QUALITY_NAME),
        text: Some(TextSize::Medium),
        ..default()
    };
    theme.set(Tag::Popup, "", popup);
}
