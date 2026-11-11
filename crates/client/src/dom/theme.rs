//! How elements look: one table from an element (and its `class`) to its
//! writing, colours and spacing. Markup never says how anything looks, so
//! changing this changes every screen.

use std::collections::HashMap;

use bevy::prelude::*;

use super::tag::Tag;
use crate::widgets::{ALLOWED, GRID_COLOUR, INK, ROW_TINT, TextSize, WARNING_TEXT};

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
        };
        for tag in [Tag::Span, Tag::Label, Tag::Td, Tag::Th] {
            theme.set(tag, "", inline);
        }
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
        theme
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
