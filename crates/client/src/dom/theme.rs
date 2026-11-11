//! How elements look: one table from an element (and its `class`) to its
//! writing, colours and spacing. Markup never says how anything looks, so
//! changing this changes every screen.

use std::collections::HashMap;

use bevy::prelude::*;

use super::tag::Tag;
use crate::widgets::{
    ALLOWED, GRID_COLOUR, INK, PARCHMENT, REFUSED, ROW_TINT, TextSize, WARNING_TEXT,
};

/// Between things stacked in a block.
const STACK: f32 = 8.0;
/// Between things flowing in a row.
const ROW: f32 = 4.0;

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
        // Spacing belongs to what holds things, not to the things: whatever
        // is stacked is `STACK` apart, and whatever flows in a row `ROW`.
        let stacked = Look {
            gap: STACK,
            ..default()
        };
        for tag in [Tag::Div, Tag::Form, Tag::Scroll] {
            theme.set(tag, "", stacked);
        }
        // Closer together, further apart (a menu's sections), and touching.
        theme.set(
            Tag::Div,
            "tight",
            Look {
                gap: ROW,
                ..stacked
            },
        );
        theme.set(
            Tag::Div,
            "sections",
            Look {
                gap: 24.0,
                ..stacked
            },
        );
        // A page of settings: rows a little apart, each a label and what it sets.
        let rows = Look {
            gap: 12.0,
            ..stacked
        };
        theme.set(Tag::Div, "rows", rows);
        // Ordinary writing where what is round it is bigger (a dialog).
        theme.set(
            Tag::Div,
            "plain",
            Look {
                text: Some(TextSize::Medium),
                ..stacked
            },
        );
        // Groups of rows, each under its heading: well apart.
        theme.set(
            Tag::Div,
            "groups",
            Look {
                gap: 40.0,
                ..stacked
            },
        );
        theme.set(Tag::Form, "rows", rows);
        let inline = Look {
            gap: ROW,
            ..default()
        };
        for tag in [Tag::Span, Tag::Label, Tag::Td, Tag::Th, Tag::P] {
            theme.set(tag, "", inline);
        }
        // Things side by side with nothing between them.
        theme.set(Tag::Span, "joined", Look::default());
        // The columns of a whole screen, well apart.
        theme.set(
            Tag::Span,
            "apart",
            Look {
                gap: 24.0,
                ..inline
            },
        );
        // Columns of a list, side by side.
        theme.set(Tag::Span, "columns", Look { gap: 8.0, ..inline });
        // Somewhere a held thing can be let go: the colour of a grid.
        theme.set(
            Tag::Span,
            "place",
            Look {
                background: Some(GRID_COLOUR),
                ..inline
            },
        );
        let labelled = Look {
            gap: 16.0,
            ..inline
        };
        theme.set(Tag::Span, "labelled", labelled);
        let coloured = |colour: Color| Look {
            colour: Some(colour),
            ..inline
        };
        theme.set(Tag::P, "warning", coloured(WARNING_TEXT));
        let sized = |text: TextSize| Look {
            text: Some(text),
            ..inline
        };
        // The small words above a box, saying what it is for.
        theme.set(Tag::Label, "field", sized(TextSize::Small));
        // What something is, above what it says: smaller and fainter.
        let faint = Look {
            colour: Some(INK.with_alpha(0.7)),
            ..sized(TextSize::Medium)
        };
        theme.set(Tag::P, "faint", faint);
        // Ordinary writing where what is round it is bigger (a dialog).
        theme.set(Tag::P, "medium", sized(TextSize::Medium));
        // Why something asked for didn't happen, in a dialog.
        let refused = Look {
            colour: Some(REFUSED),
            ..sized(TextSize::Medium)
        };
        theme.set(Tag::P, "refused", refused);
        theme.set(Tag::H1, "", sized(TextSize::Title));
        theme.set(Tag::H2, "", sized(TextSize::Large));
        theme.set(Tag::Small, "", sized(TextSize::Small));
        // Small buttons in a row, touching.
        theme.set(
            Tag::Small,
            "joined",
            Look {
                text: Some(TextSize::Small),
                ..default()
            },
        );
        let rule = Look {
            background: Some(INK.with_alpha(0.5)),
            ..default()
        };
        theme.set(Tag::Hr, "", rule);
        theme.set(Tag::Table, "striped", Look::default());
        let [red, green, blue] = PARCHMENT;
        let parchment = Look {
            background: Some(Color::srgb_u8(red, green, blue)),
            ..stacked
        };
        theme.set(Tag::Panel, "", parchment);
        let footer = Look {
            gap: STACK,
            ..default()
        };
        theme.set(Tag::Footer, "", footer);
        // The frame round a grid: its rows stacked, its controls close.
        theme.set(Tag::Container, "", stacked);
        theme.set(Tag::Header, "", inline);
        // Dialogs are the menus': their writing is the menus' size.
        let dialog = Look {
            text: Some(TextSize::Large),
            ..stacked
        };
        theme.set(Tag::Dialog, "", dialog);
        theme.set(
            Tag::Dialog,
            "sections",
            Look {
                gap: 24.0,
                ..dialog
            },
        );
        // Controls' classes pick among the widgets' own looks (`controls`).
        let controls = [
            (
                Tag::Button,
                &["", "main", "quiet", "danger", "folder", "small"][..],
            ),
            (Tag::Input, &["", "error"]),
            (Tag::KeyBox, &["", "warned"]),
        ];
        for (tag, classes) in controls {
            for class in classes {
                theme.set(tag, class, Look::default());
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
