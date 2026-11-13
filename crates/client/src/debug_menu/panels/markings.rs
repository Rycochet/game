//! Item Pictures' Bodies page: each kind with markings (`shared::sheet::Marks`) in
//! every one of its markings, over each coat in turn, to see bellies, faces and how
//! patches differ between animals.

use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::body::Layout;
use shared::creature::CreatureKind;
use shared::sheet::{Appearance, Trait};

use super::armour::spawn_row;
use super::pictures_form::Shown;
use crate::item_art::{Colours, Look};

/// A kind in one of its markings over one of its coats.
#[derive(Clone, Copy)]
pub(super) struct MarkedShown {
    kind: CreatureKind,
    coat: u8,
    markings: u8,
}

impl MarkedShown {
    pub(super) fn look(self) -> Option<Look> {
        let appearance = Appearance(BTreeMap::from([
            (Trait::CoatColour, self.coat),
            (Trait::Markings, self.markings),
        ]));
        let layout = Layout::of(self.kind)?;
        Some(Look::Body {
            layout: Box::new(layout),
            height: shared::body::Frame::usual_height(self.kind),
            colours: Colours::of(self.kind, &appearance),
            hair: appearance.hair(self.kind),
        })
    }
}

/// Two rows for each kind with markings: every marking over its first coat, then
/// over its third.
pub(super) fn spawn_marking_rows(grid: &mut ChildSpawnerCommands) {
    for kind in CreatureKind::ALL {
        let markings = Trait::Markings.choices_for(kind).len() as u8;
        if markings == 0 {
            continue;
        }
        for (coat, label) in [(0, kind.def().name), (2, "")] {
            let shown = (0..markings).map(move |markings| {
                Shown::Marked(MarkedShown {
                    kind,
                    coat,
                    markings,
                })
            });
            spawn_row(grid, label, shown);
        }
    }
}
