//! Rolling armour or footwear to a request: rolled again until its style and
//! construction are those asked for, then given the material, finish and dye asked
//! for wherever its rules allow them, as a set's pieces are given their torso's.

use super::ItemForm;
use super::armour::Piece;
use super::request::{Field, Refused, RollRequest};
use crate::armour::Style;
use crate::armour::set::{self, SetKind};
use crate::armour::torso::TorsoKind;
use crate::item::Quality;
use crate::rng::Rng;

/// How many pieces are rolled before a request is given up on. Rolls are cheap, and
/// a construction can be rare in a style.
pub(super) const TRIES: usize = 400;

/// How many sets are rolled before giving up on a dye: their pieces may have no cloth.
const SET_TRIES: usize = 20;

/// A piece of armour or footwear as asked for.
pub(super) fn armour_form_to(
    piece: Piece,
    rng: &mut Rng,
    quality: Quality,
    request: &RollRequest,
) -> Result<ItemForm, Refused> {
    if request
        .finish
        .is_some_and(|finish| !finish.allowed(quality))
    {
        return Err(Refused(Field::Finish));
    }
    if let (Some(style), Some(construction)) = (request.style, request.construction)
        && !style.allows(construction)
    {
        return Err(Refused(Field::Construction));
    }
    match piece {
        Piece::Set(kind) => roll_set(rng, kind, quality, request),
        _ => roll_piece(piece, rng, quality, request, |_| true),
    }
}

/// Rolls until a piece keeps to the request and `fits`. Refused with the option the
/// rolls got furthest towards and no further.
fn roll_piece(
    piece: Piece,
    rng: &mut Rng,
    quality: Quality,
    request: &RollRequest,
    fits: impl Fn(&ItemForm) -> bool,
) -> Result<ItemForm, Refused> {
    let mut furthest = Field::Style;
    for _ in 0..TRIES {
        let style = match (request.style, request.construction) {
            (Some(style), _) => Some(style),
            (None, Some(construction)) => Some(Style::roll_allowing(rng, construction)),
            (None, None) => None,
        };
        let mut form = piece.roll(rng, quality, style);
        match keep_to(&mut form, piece, request, &fits) {
            Ok(()) => return Ok(form),
            Err(field) if stage(field) > stage(furthest) => furthest = field,
            Err(_) => {}
        }
    }
    Err(Refused(furthest))
}

/// Checks a rolled piece against the request, giving it what it asks for. The first
/// option it can't keep to, if any.
fn keep_to(
    form: &mut ItemForm,
    piece: Piece,
    request: &RollRequest,
    fits: &impl Fn(&ItemForm) -> bool,
) -> Result<(), Field> {
    if request
        .style
        .is_some_and(|style| form.armour_style() != Some(style))
    {
        return Err(Field::Style);
    }
    let construction = request.construction;
    if construction.is_some_and(|wanted| form.armour_construction() != Some(wanted)) {
        return Err(Field::Construction);
    }
    if !fits(form) {
        return Err(Field::Construction);
    }
    if let Some(material) = request.material
        && !form.adopt_material(piece, material)
    {
        return Err(Field::Material);
    }
    if let Some(finish) = request.finish
        && !form.adopt_finish(piece, finish)
    {
        return Err(Field::Finish);
    }
    if let Some(dye) = request.dye
        && !form.adopt_dye(piece, dye)
    {
        return Err(Field::Dye);
    }
    Ok(())
}

/// The order options are checked in, so the furthest one reached is named.
fn stage(field: Field) -> u8 {
    match field {
        Field::Style => 0,
        Field::Construction => 1,
        Field::Material => 2,
        Field::Finish => 3,
        _ => 4,
    }
}

/// A set: its torso piece rolled to the request, then the rest around it, which takes
/// the torso's style, material and finish; then dyed, if asked.
fn roll_set(
    rng: &mut Rng,
    kind: SetKind,
    quality: Quality,
    request: &RollRequest,
) -> Result<ItemForm, Refused> {
    let torso_request = request.without(Field::Dye);
    let fits =
        |form: &ItemForm| matches!(form, ItemForm::Torso(torso) if set::torso_fits(kind, torso));
    for _ in 0..SET_TRIES {
        let torso = roll_piece(
            Piece::Torso(TorsoKind::Armour),
            rng,
            quality,
            &torso_request,
            fits,
        )?;
        let ItemForm::Torso(torso) = torso else {
            unreachable!("a torso piece rolls a torso form")
        };
        let set = set::roll_around(rng, kind, quality, set::with_back(torso));
        let mut form = ItemForm::Set(Box::new(set));
        match request.dye {
            Some(dye) if !form.adopt_dye(Piece::Set(kind), dye) => continue,
            _ => return Ok(form),
        }
    }
    Err(Refused(Field::Dye))
}
