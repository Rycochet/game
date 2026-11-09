//! A character's picture: its body with everything that can be seen of what it
//! wears, each piece built in place on its frame, drawn as an item's picture is.

use bevy::prelude::*;
use shared::armour::hands::Hand;
use shared::body::{Hair, Layout};
use shared::creature::CreatureKind;
use shared::item_form::ItemForm;
use shared::sheet::Appearance;
use shared::wear::WornLook;

mod cover;
mod portrait;

pub use portrait::portrait_look;
pub(super) use portrait::{PORTRAIT_CELLS, portrait_scale};

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

/// How a creature of this kind looks wearing these, if its body is drawn: what its
/// token shows in place of its icon, its body in its own colours, bare when nothing it
/// wears is drawn. An animal is its bare body, as nothing is drawn on it yet.
pub fn character_look(
    kind: CreatureKind,
    appearance: &Appearance,
    worn: &[WornLook],
) -> Option<Look> {
    let layout = Layout::of(kind)?;
    let hair = appearance.hair(kind);
    if layout.worn_frame().is_none() {
        return Some(Look::Body {
            layout: Box::new(layout),
            height: layout.spine.base[1],
            colours: Colours::of(kind, appearance),
            hair,
        });
    }
    let worn: Vec<Look> = worn.iter().filter_map(piece_look).collect();
    Some(Look::Character {
        kind,
        worn,
        colours: Colours::of(kind, appearance),
        hair,
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
        ItemForm::Gem(_) | ItemForm::Ring(_) | ItemForm::Blade(_) | ItemForm::Key(_) => {
            return None;
        }
    })
}

/// A body of this kind wearing these, filling its picture (or, with `standing`,
/// in a portrait at that scale: `portrait_scale`), in its own colours, with what
/// can be seen of its hair.
#[allow(clippy::too_many_arguments)]
pub(super) fn character_parts(
    kind: CreatureKind,
    worn: &[Look],
    colours: Colours,
    hair: Hair,
    standing: Option<f32>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let Some(layout) = Layout::of(kind) else {
        return Vec::new();
    };
    // What is worn is built on the frame and moved into place: on a taur, up onto its
    // lower body.
    let Some((frame, offset)) = layout.worn_frame() else {
        return Vec::new();
    };
    let offset = Vec3::new(offset[0] as f32, offset[1] as f32, offset[2] as f32);
    let (skin, coat) = body::build_coated(&layout);
    let mut pieces: set::Coloured = vec![
        (skin, material(colours.skin)),
        (coat, material(colours.coat)),
        (body::wings(&layout), material(colours.feathers)),
    ];
    // The hair on the upper body's head, for a taur, under whatever covers it.
    let head = layout.upper_body().unwrap_or(layout);
    let cover = cover::cover_of(worn, &frame, offset.y);
    pieces.push((body::hair(&head, hair, cover), material(colours.hair)));
    pieces.push((body::coat_hair(&layout), material(colours.mane)));
    // An aventail hangs from the helmet worn with it.
    let rim = worn.iter().find_map(|look| match look {
        Look::Head { head, .. } => Some(head.form.rim(&frame)),
        _ => None,
    });
    for look in worn {
        for (triangles, stuff) in worn::worn(look, &frame, rim.as_deref()) {
            let moved = triangles
                .into_iter()
                .map(|triangle| triangle.map(|corner| corner + offset))
                .collect();
            pieces.push((moved, stuff));
        }
    }
    pieces.retain(|(triangles, _)| !triangles.is_empty());
    let cells = if standing.is_some() {
        PORTRAIT_CELLS
    } else {
        CELLS
    };
    let turned = body::turn_of(&layout);
    set::fit_standing(pieces, cells, turned, standing, meshes, materials)
}

#[cfg(test)]
mod tests;
