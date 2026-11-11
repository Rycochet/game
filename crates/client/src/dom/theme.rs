//! How elements look: one table from an element (and its `class`) to its
//! writing, colours and spacing. Markup never says how anything looks, so
//! changing this changes every screen.

use std::collections::HashMap;

use bevy::prelude::*;

use super::tag::Tag;
use crate::widgets::{ALLOWED, GRID_COLOUR, INK, PARCHMENT, ROW_TINT, TextSize, WARNING_TEXT};

/// How one kind of element looks. What is `None` is as its parent's.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Look {
    pub text: Option<TextSize>,
    pub colour: Option<Color>,
    /// The room round it: top, right, bottom, left. Multiples of 4.
    pub margin: [f32; 4],
    /// Between the things flowing along inside it.
    pub gap: f32,
    pub background: Option<Color>,
}

/// Every element's look, by its `class` (`""` for none).
#[derive(Resource, Clone, Debug)]
pub struct Theme {
    looks: HashMap<Tag, Vec<(&'static str, Look)>>,
    /// Writing with nothing round it saying otherwise.
    pub text: TextSize,
    pub colour: Color,
    /// Every other row of a `striped` table.
    pub row_tint: Color,
    /// A progress bar's track, and what fills it.
    pub progress: (Color, Color),
    /// How tall a line of writing is, in font sizes.
    pub line: f32,
    /// How tall a control is at least, by the size of the writing round it:
    /// the menus' bigger writing has taller buttons and boxes.
    pub rows: Vec<(TextSize, f32)>,
    /// How wide a button in a dialog's `footer` is at least, so its buttons
    /// match.
    pub footer_button: f32,
}

impl Theme {
    /// The game's own look: ink on parchment.
    pub fn parchment() -> Self {
        let inline = Look {
            gap: 4.0,
            ..default()
        };
        let mut theme = Self {
            looks: HashMap::new(),
            text: TextSize::Medium,
            colour: INK,
            row_tint: ROW_TINT,
            progress: (GRID_COLOUR, ALLOWED),
            line: 1.2,
            rows: vec![(TextSize::Large, 38.0)],
            footer_button: 120.0,
        };
        for tag in [Tag::Span, Tag::Label, Tag::Td, Tag::Th] {
            theme.set(tag, "", inline);
        }
        // Things side by side with nothing between them.
        theme.set(Tag::Span, "joined", Look::default());
        let paragraph = Look {
            margin: [0.0, 0.0, 8.0, 0.0],
            ..inline
        };
        theme.set(Tag::P, "", paragraph);
        let warning = Look {
            colour: Some(WARNING_TEXT),
            ..paragraph
        };
        theme.set(Tag::P, "warning", warning);
        // What something is, above what it says: smaller and fainter.
        let faint = Look {
            text: Some(TextSize::Medium),
            colour: Some(INK.with_alpha(0.7)),
            ..paragraph
        };
        theme.set(Tag::P, "faint", faint);
        // Ordinary writing where what is round it is bigger (a dialog).
        let medium = Look {
            text: Some(TextSize::Medium),
            ..paragraph
        };
        theme.set(Tag::P, "medium", medium);
        let title = Look {
            text: Some(TextSize::Title),
            ..paragraph
        };
        theme.set(Tag::H1, "", title);
        let heading = Look {
            text: Some(TextSize::Large),
            margin: [0.0, 0.0, 4.0, 0.0],
            ..inline
        };
        theme.set(Tag::H2, "", heading);
        let small = Look {
            text: Some(TextSize::Small),
            ..inline
        };
        theme.set(Tag::Small, "", small);
        let rule = Look {
            margin: [4.0, 0.0, 4.0, 0.0],
            background: Some(INK.with_alpha(0.5)),
            ..default()
        };
        theme.set(Tag::Hr, "", rule);
        theme.set(Tag::Table, "striped", Look::default());
        let [red, green, blue] = PARCHMENT;
        let parchment = Look {
            background: Some(Color::srgb_u8(red, green, blue)),
            ..default()
        };
        theme.set(Tag::Panel, "", parchment);
        let footer = Look {
            gap: 8.0,
            ..default()
        };
        theme.set(Tag::Footer, "", footer);
        // Dialogs are the menus': their writing is the menus' size.
        let dialog = Look {
            text: Some(TextSize::Large),
            ..default()
        };
        theme.set(Tag::Dialog, "", dialog);
        // Controls keep a little room under them, as paragraphs do. Their
        // classes pick among the widgets' own looks (`controls`).
        let control = Look {
            margin: [0.0, 0.0, 4.0, 0.0],
            ..default()
        };
        let controls = [
            (
                Tag::Button,
                &["", "main", "quiet", "danger", "folder", "small"][..],
            ),
            (Tag::Input, &["", "error"]),
            (Tag::KeyBox, &["", "warned"]),
            (Tag::Select, &[""]),
            (Tag::ToggleRow, &[""]),
            (Tag::TabBar, &[""]),
            (Tag::ColourPicker, &[""]),
        ];
        for (tag, classes) in controls {
            for class in classes {
                theme.set(tag, class, control);
            }
        }
        theme
    }

    /// How tall a control is at least where the writing is this size, if
    /// that is more than its own height.
    pub fn row(&self, size: TextSize) -> Option<f32> {
        let found = self.rows.iter().find(|(known, _)| *known == size);
        found.map(|(_, height)| *height)
    }

    /// Gives `tag` elements of this `class` (`""` for none) a look.
    pub fn set(&mut self, tag: Tag, class: &'static str, look: Look) {
        let looks = self.looks.entry(tag).or_default();
        looks.retain(|(known, _)| *known != class);
        looks.push((class, look));
    }

    /// Whether `tag` elements have a look of their own for this `class`.
    pub fn has(&self, tag: Tag, class: &str) -> bool {
        self.find(tag, class).is_some()
    }

    /// How a `tag` element of this `class` looks: its class's look, else the
    /// element's plain one, else nothing of its own.
    pub fn look(&self, tag: Tag, class: Option<&str>) -> Look {
        let classed = class.and_then(|class| self.find(tag, class));
        classed.or_else(|| self.find(tag, "")).unwrap_or_default()
    }

    fn find(&self, tag: Tag, class: &str) -> Option<Look> {
        let looks = self.looks.get(&tag)?;
        let found = looks.iter().find(|(known, _)| *known == class);
        found.map(|(_, look)| *look)
    }
}

#[cfg(test)]
mod tests;
