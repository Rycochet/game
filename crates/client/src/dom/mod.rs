//! The interface as markup: HTML's look with XML's closing rule, no CSS and
//! no scripts. Templates are checked strictly, built into Bevy UI nodes, and
//! changed by id through `Dom`. See `docs/context/client/dom.md`, and
//! `docs/context/html-interface.md` for the whole plan this is the start of.

mod attrs;
mod build;
mod change;
mod check;
mod controls;
#[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
mod debug_api;
mod error;
mod input;
mod layout;
mod markup;
mod objects;
mod parts;
mod pictures;
mod restyle;
mod surfaces;
mod tag;
mod template;
mod theme;

use bevy::prelude::*;

pub use change::Dom;
pub use controls::{UiEvent, UiValue};
pub use input::Hit;
#[cfg(test)]
pub use input::find_now;
pub use parts::{AlwaysBars, Solid};
pub use parts::{Document, DocumentOf, document, document_in};
pub use template::Template;
// What screens will use as they become markup: not all of it is yet.
#[allow(unused_imports)]
pub use {
    attrs::{Align, Attrs, Length, VAlign},
    check::{Checked, CheckedNode},
    error::{MarkupError, Place},
    input::{
        Click, ContextMenu, DomInput, Finder, PointerCapture, PointerEnter, PointerLeave, Press,
        Release, Wheel,
    },
    parts::Hovered,
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
            // (`common` makes it too: here for an `object`'s items wherever
            // documents are built.)
            .init_resource::<crate::common::KnownItems>();
        input::add(app);
        controls::add(app);
        #[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
        debug_api::add(app);
        app
            // After `Update`, where documents are spawned and changed, and in
            // time to be laid out this frame.
            .add_systems(
                PostUpdate,
                (
                    build::build_documents,
                    change::apply_changes,
                    surfaces::tidy_surfaces,
                    (
                        restyle::restyle,
                        controls::follow_elements,
                        surfaces::follow_surfaces,
                        objects::show_objects,
                    ),
                )
                    .chain()
                    .before(bevy::ui::UiSystems::Layout),
            );
    }
}
