//! What can be seen of weapons and armour pieces: what their pictures are kept by.

use shared::armour::arms::ArmsForm;
use shared::armour::hands::{Hand, HandsForm};
use shared::armour::head::HeadForm;
use shared::armour::shoulders::ShouldersForm;
use shared::armour::torso::TorsoForm;
use shared::blade::BladeForm;
use shared::item::Item;
use shared::item_form::ItemForm;

use super::Look;

/// What can be seen of a weapon: lengths to the nearest centimetre, so that nearly the
/// same weapons share a picture, and without what doesn't show.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BladeLook {
    pub(in crate::item_art) form: BladeForm,
    /// What it takes up, wide and long, not turned.
    pub(in crate::item_art) cells: (u8, u8),
}

/// A length to the nearest centimetre.
fn to_centimetre(millimetres: u16) -> u16 {
    (millimetres + 5) / 10 * 10
}

/// How a weapon looks, from its form and the cells it takes up.
pub(super) fn blade_look(item: &Item, form: &BladeForm) -> Look {
    let shape = item.shape();
    Look::Blade {
        blade: BladeLook {
            form: BladeForm {
                blade: to_centimetre(form.blade),
                handle: to_centimetre(form.handle),
                generic_only: false,
                ..*form
            },
            cells: (shape.width, shape.height),
        },
        quality: item.quality,
    }
}

/// What can be seen of a pair of hand pieces: how many digits each hand has shows.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HandsLook {
    pub(in crate::item_art) form: HandsForm,
    pub(in crate::item_art) digits: u8,
    /// What it takes up, wide and long, not turned.
    pub(in crate::item_art) cells: (u8, u8),
}

impl HandsLook {
    /// Hand pieces with this many digits on each hand, taking up these cells.
    pub fn new(form: HandsForm, digits: u8, cells: (u8, u8)) -> Self {
        Self {
            form,
            digits,
            cells,
        }
    }
}

/// What can be seen of a pair of shoulder pieces.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShouldersLook {
    pub(in crate::item_art) form: ShouldersForm,
    /// What it takes up, wide and long, not turned.
    pub(in crate::item_art) cells: (u8, u8),
}

impl ShouldersLook {
    /// Shoulder pieces taking up these cells.
    pub fn new(form: ShouldersForm, cells: (u8, u8)) -> Self {
        Self { form, cells }
    }
}

/// What can be seen of a piece of torso armour.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TorsoLook {
    pub(in crate::item_art) form: TorsoForm,
    /// What it takes up, wide and long, not turned.
    pub(in crate::item_art) cells: (u8, u8),
}

impl TorsoLook {
    /// A torso piece taking up these cells.
    pub fn new(form: TorsoForm, cells: (u8, u8)) -> Self {
        Self { form, cells }
    }
}

/// What can be seen of a head piece.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HeadLook {
    pub(in crate::item_art) form: HeadForm,
    /// What it takes up, wide and long, not turned.
    pub(in crate::item_art) cells: (u8, u8),
}

impl HeadLook {
    /// A head piece taking up these cells.
    pub fn new(form: HeadForm, cells: (u8, u8)) -> Self {
        Self { form, cells }
    }
}

/// What can be seen of arm pieces, for one arm or both.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArmsLook {
    pub(in crate::item_art) form: ArmsForm,
    /// What it takes up, wide and long, not turned.
    pub(in crate::item_art) cells: (u8, u8),
}

impl ArmsLook {
    /// Arm pieces taking up these cells.
    pub fn new(form: ArmsForm, cells: (u8, u8)) -> Self {
        Self { form, cells }
    }
}

/// How an armour item looks, from its form and the cells it takes up.
pub(super) fn armour_look(item: &Item) -> Option<Look> {
    let quality = item.quality;
    let shape = item.shape();
    let cells = (shape.width, shape.height);
    Some(match item.form.as_ref()? {
        ItemForm::Hands(form) => Look::Hands {
            hands: HandsLook::new(*form, Hand::digits(item.made.fingers), cells),
            quality,
        },
        ItemForm::Shoulders(form) => Look::Shoulders {
            shoulders: ShouldersLook::new(*form, cells),
            quality,
        },
        ItemForm::Torso(form) => Look::Torso {
            torso: TorsoLook::new(*form, cells),
            quality,
        },
        ItemForm::Head(form) => Look::Head {
            head: HeadLook::new(*form, cells),
            quality,
        },
        ItemForm::Arms(form) => Look::Arms {
            arms: ArmsLook::new(*form, cells),
            quality,
        },
        _ => return None,
    })
}
