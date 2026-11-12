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
mod hold;
mod input;
mod layout;
mod markup;
mod object_ability;
mod object_grid;
mod object_slot;
mod objects;
mod parts;
mod pictures;
mod popup;
mod restyle;
mod strip;
mod surfaces;
mod tag;
mod template;
mod theme;

use bevy::prelude::*;

pub use change::Dom;
pub use controls::{UiEvent, UiValue};
pub use hold::{Dropped, Hand, HeldItem, HeldObject, HeldOver, HeldThing, HeldWhat, HoldObjects};
pub use input::Hit;
#[cfg(test)]
pub use input::find_now;
pub use object_ability::{
    AbilityCell, AbilityGridShown, AbilitySlotShown, AbilityThing, KnownAbilities, SlotPart,
};
pub use object_grid::{DrawObjects, GridAim, GridBox, GridShown, GridThing, ThingCell};
pub use object_slot::{SlotLook, SlotShown, SlotThing};
pub use parts::{AlwaysBars, Solid};
pub use parts::{Document, DocumentOf, document, document_in};
pub use popup::ClosePopups;
pub use strip::{bottom_strip, top_strip};
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
    popup::Popup,
    tag::Tag,
    theme::{Look, Theme},
};

/// For other modules' tests of their markup: an app that builds documents,
/// and the element with an id.
#[cfg(test)]
pub(crate) use build::tests::{app as test_app, find as test_find};

pub struct DomPlugin;

impl Plugin for DomPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<change::DomChange>()
            .insert_resource(Theme::parchment())
            .init_resource::<Pictures>()
            // (`common` makes it too: here for an `object`'s items wherever
            // documents are built.)
            .init_resource::<crate::common::KnownItems>()
            .init_resource::<KnownAbilities>();
        input::add(app);
        hold::add(app);
        popup::add(app);
        controls::add(app);
        #[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
        debug_api::add(app);
        app
            // After `Update`, where documents are spawned and changed, and in
            // time to be laid out this frame.
            .add_systems(
                PostUpdate,
                (
                    // (First: a panel whose document went this frame is gone
                    // before one built in its place, with the same ids, is
                    // changed by them.)
                    surfaces::tidy_surfaces,
                    build::build_documents,
                    change::apply_changes,
                    (
                        restyle::restyle,
                        controls::follow_elements,
                        surfaces::follow_surfaces,
                        objects::show_objects,
                        object_grid::draw_object_grids.in_set(DrawObjects),
                        object_slot::draw_object_slots.in_set(DrawObjects),
                        object_ability::draw_ability_grids.in_set(DrawObjects),
                        object_ability::draw_ability_slots.in_set(DrawObjects),
                    ),
                )
                    .chain()
                    .before(bevy::ui::UiSystems::Layout),
            );
    }
}
