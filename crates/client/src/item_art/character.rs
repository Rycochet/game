//! A character's picture: its body with everything that can be seen of what it
//! wears, each piece built in place on its frame, drawn as an item's picture is.

use bevy::prelude::*;
use shared::armour::hands::Hand;
use shared::body::Frame;
use shared::creature::CreatureKind;
use shared::item_form::ItemForm;
use shared::sheet::Appearance;
use shared::wear::WornLook;

use super::armour::{set, worn};
use super::colours::{Colours, material};
use super::{
    ArmsLook, FeetLook, FootwearLook, HandsLook, HeadLook, LegsLook, Look, NeckLook, Part, SetLook,
    ShouldersLook, TorsoLook, WaistLook, body,
};

/// A character's picture is square, to stand in place of a token's icon.
pub(super) const CELLS: (u8, u8) = (2, 2);

/// A piece's own picture never comes into it: only its form.
const ANY: (u8, u8) = (1, 1);

/// How a creature of this kind looks wearing these, if it stands on two legs and
/// anything it wears is drawn: what its token shows in place of its icon, its body in
/// its own colours.
pub fn character_look(
    kind: CreatureKind,
    appearance: &Appearance,
    worn: &[WornLook],
) -> Option<Look> {
    Frame::of(kind)?;
    let worn: Vec<Look> = worn.iter().filter_map(piece_look).collect();
    (!worn.is_empty()).then_some(Look::Character {
        kind,
        worn,
        colours: Colours::of(kind, appearance),
    })
}

/// How one worn thing looks, for its form.
fn piece_look(worn: &WornLook) -> Option<Look> {
    let quality = worn.quality;
    Some(match &worn.form {
        ItemForm::Hands(form) => Look::Hands {
            hands: HandsLook::new(*form, Hand::digits(worn.fingers), ANY),
            quality,
        },
        ItemForm::Shoulders(form) => Look::Shoulders {
            shoulders: ShouldersLook::new(*form, ANY),
            quality,
        },
        ItemForm::Torso(form) => Look::Torso {
            torso: TorsoLook::new(*form, ANY),
            quality,
        },
        ItemForm::Head(form) => Look::Head {
            head: HeadLook::new(*form, ANY),
            quality,
        },
        ItemForm::Arms(form) => Look::Arms {
            arms: ArmsLook::new(*form, ANY),
            quality,
        },
        ItemForm::Legs(form) => Look::Legs {
            legs: LegsLook::new(*form, ANY),
            quality,
        },
        ItemForm::Waist(form) => Look::Waist {
            waist: WaistLook::new(*form, ANY),
            quality,
        },
        ItemForm::Feet(form) => Look::Feet {
            feet: FeetLook::new(*form, ANY),
            quality,
        },
        ItemForm::Neck(form) => Look::Neck {
            neck: NeckLook::new(*form, ANY),
            quality,
        },
        ItemForm::Footwear(form) => Look::Footwear {
            footwear: FootwearLook::new(*form, ANY),
            quality,
        },
        ItemForm::Set(form) => Look::Set {
            set: SetLook::new(**form, Hand::digits(worn.fingers), ANY),
            quality,
        },
        ItemForm::Gem(_) | ItemForm::Ring(_) | ItemForm::Blade(_) => return None,
    })
}

/// A body of this kind wearing these, filling its picture, in its own colours.
pub(super) fn character_parts(
    kind: CreatureKind,
    worn: &[Look],
    colours: Colours,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let Some(frame) = Frame::of(kind) else {
        return Vec::new();
    };
    let parts = shared::body::Parts::of(kind);
    let mut pieces: set::Coloured = vec![
        (body::build(&frame, &parts), material(colours.skin)),
        (body::wings(&frame, &parts), material(colours.feathers)),
    ];
    // An aventail hangs from the helmet worn with it.
    let rim = worn.iter().find_map(|look| match look {
        Look::Head { head, .. } => Some(head.form.rim(&frame)),
        _ => None,
    });
    for look in worn {
        pieces.extend(worn::worn(look, &frame, rim.as_deref()));
    }
    set::fit(pieces, CELLS, meshes, materials)
}
