//! The interface as markup: HTML's look with XML's closing rule, no CSS and
//! no scripts. Templates are checked strictly, built into Bevy UI nodes, and
//! changed by id through `Dom`. See `docs/context/client/dom.md`, and
//! `docs/context/html-interface.md` for the whole plan this is the start of.

mod attrs;
mod build;
mod change;
mod check;
mod error;
mod layout;
mod markup;
mod parts;
mod pictures;
mod restyle;
mod tag;
mod template;
mod theme;

use bevy::prelude::*;

pub use change::Dom;
pub use parts::{Document, document};
pub use template::Template;
// What screens will use as they become markup: not all of it is yet.
#[allow(unused_imports)]
pub use {
    attrs::{Align, Attrs, Length, VAlign},
    check::{Checked, CheckedNode},
    error::{MarkupError, Place},
    parts::{Canvas, Element, Inert},
    pictures::Pictures,
    tag::Tag,
    theme::{Look, Theme},
};

pub struct DomPlugin;

impl Plugin for DomPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<change::DomChange>()
            .insert_resource(Theme::parchment())
            .init_resource::<Pictures>()
            // After `Update`, where documents are spawned and changed, and in
            // time to be laid out this frame.
            .add_systems(
                PostUpdate,
                (
                    build::build_documents,
                    change::apply_changes,
                    restyle::restyle,
                )
                    .chain()
                    .before(bevy::ui::UiSystems::Layout),
            );
    }
}
