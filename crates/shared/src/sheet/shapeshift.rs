//! A body changing into another kind's for a while (owner, 2026-11-06): the form's
//! Body row (Strength, Agility, Toughness) takes over, leaned by the creature's own
//! triangle, while Mind and Spirit, its triangles, looks and skills stay its own. Its
//! own kind and Body row are kept, so changing back gives exactly what it had.

use serde::{Deserialize, Serialize};

use super::{Attributes, Row, Sheet};
use crate::creature::CreatureKind;

/// What a shapeshifted creature really is: the kind it was before changing, and its
/// own Body row, to have back on changing back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrueForm {
    pub kind: CreatureKind,
    pub body: [u16; 3],
}

impl Sheet {
    /// Its body changes from `current` into `form`'s, for a while: back into its own
    /// kind, it has its own Body row again.
    pub fn shapeshift(&mut self, current: CreatureKind, form: CreatureKind) {
        let body = Row::Body.attributes();
        let own = self.true_form.unwrap_or(TrueForm {
            kind: current,
            body: body.map(|which| self.attributes.get(which)),
        });
        if form == own.kind {
            for (which, value) in body.into_iter().zip(own.body) {
                self.attributes.set(which, value);
            }
            self.true_form = None;
            return;
        }
        let shaped = Attributes::for_kind(form, &self.build);
        for which in body {
            self.attributes.set(which, shaped.get(which));
        }
        self.true_form = Some(own);
    }

    /// The kind it really is, given the kind its body is now.
    pub fn own_kind(&self, current: CreatureKind) -> CreatureKind {
        self.true_form.map_or(current, |own| own.kind)
    }
}

#[cfg(test)]
mod tests;
