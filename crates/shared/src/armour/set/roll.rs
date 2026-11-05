//! Rolling a set: its torso piece first, then each other location's piece in the same
//! style, made of the torso's material where that piece can be.

use super::{SetForm, SetKind};
use crate::armour::hands::{self, HandsPiece};
use crate::armour::head::{self, HeadKind};
use crate::armour::legs::{self, LegsKind};
use crate::armour::neck::{self, NeckKind};
use crate::armour::torso::{self, TorsoForm, TorsoKind};
use crate::armour::waist::{self, WaistKind, WaistPiece};
use crate::armour::{Construction, Finish, Style, arms, feet, shoulders};
use crate::item::Quality;
use crate::material::Material;
use crate::rng::Rng;

/// How many pieces are rolled for a location before the best of them is kept.
const TRIES: usize = 40;

/// What the other pieces should be like: the torso's.
#[derive(Clone, Copy)]
struct Wanted {
    construction: Construction,
    material: Material,
    finish: Finish,
}

/// Rolls a set of this kind, at this quality, in a style or one of its own. Every
/// piece shares the style; the
/// material and finish are the torso's wherever a piece can take them, and a piece of
/// the torso's construction is kept over another.
pub fn roll(rng: &mut Rng, kind: SetKind, style: Option<Style>, quality: Quality) -> SetForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    let torso = roll_torso(rng, kind, style, quality);
    roll_around(rng, kind, quality, torso)
}

/// A set of this kind around its torso piece, which gives every other piece its
/// style, and its material and finish where they can take them.
pub(crate) fn roll_around(
    rng: &mut Rng,
    kind: SetKind,
    quality: Quality,
    torso: TorsoForm,
) -> SetForm {
    let style = torso.style;
    let wanted = Wanted {
        construction: torso.construction,
        material: torso.material,
        finish: torso.finish,
    };
    let covers = |location| kind.locations().contains(&location);
    use crate::creature::Location as L;
    // A coat's sleeves and skirt are of a piece with its body.
    let coat = kind == SetKind::Coat;
    let style = Some(style);
    let head = covers(L::Head).then(|| {
        let roll = |rng: &mut Rng| head::roll(rng, HeadKind::Helmet, style, quality);
        best(rng, roll, |form| {
            adopt(
                form,
                wanted,
                |f| (&mut f.construction, &mut f.material, Some(&mut f.finish)),
                |f| f.allowed(),
            )
        })
    });
    let neck = covers(L::Neck).then(|| {
        // Mail and other flexible bodies hang an aventail; rigid ones close with a gorget.
        let kind = if torso.rigid() || kind == SetKind::Corslet {
            NeckKind::Gorget
        } else {
            NeckKind::Aventail
        };
        let roll = |rng: &mut Rng| neck::roll(rng, kind, style, quality);
        best(rng, roll, |form| {
            adopt(
                form,
                wanted,
                |f| (&mut f.construction, &mut f.material, Some(&mut f.finish)),
                |f| f.allowed(),
            )
        })
    });
    let shoulders = covers(L::Shoulders).then(|| {
        let roll = |rng: &mut Rng| shoulders::roll(rng, None, style);
        best(rng, roll, |form| {
            adopt(
                form,
                wanted,
                |f| (&mut f.construction, &mut f.material, None),
                |f| f.allowed(),
            )
        })
    });
    let arms = covers(L::Arms).then(|| {
        let roll = |rng: &mut Rng| arms::roll(rng, style, quality);
        best(rng, roll, |form| {
            let score = adopt(
                form,
                wanted,
                |f| (&mut f.construction, &mut f.material, Some(&mut f.finish)),
                |f| f.allowed(),
            );
            if coat && form.construction != wanted.construction {
                0
            } else {
                score
            }
        })
    });
    let waist = covers(L::Waist).then(|| {
        let roll = |rng: &mut Rng| waist::roll(rng, WaistKind::Armour, style, quality);
        best(rng, roll, |form| {
            let score = adopt(
                form,
                wanted,
                |f| (&mut f.construction, &mut f.material, Some(&mut f.finish)),
                |f| f.allowed(),
            );
            let skirt = form.piece == WaistPiece::Skirt;
            if coat && (form.construction != wanted.construction || !skirt) {
                0
            } else {
                score
            }
        })
    });
    let hands = covers(L::Hands).then(|| {
        let roll = |rng: &mut Rng| hands::roll(rng, HandsPiece::Gauntlets, style);
        best(rng, roll, |form| {
            adopt(
                form,
                wanted,
                |f| (&mut f.construction, &mut f.material, None),
                |f| f.allowed(HandsPiece::Gauntlets),
            )
        })
    });
    let legs = covers(L::Legs).then(|| {
        let roll = |rng: &mut Rng| legs::roll(rng, LegsKind::Armour, style, quality);
        best(rng, roll, |form| {
            adopt(
                form,
                wanted,
                |f| (&mut f.construction, &mut f.material, Some(&mut f.finish)),
                |f| f.allowed(),
            )
        })
    });
    let feet = covers(L::Feet).then(|| {
        let roll = |rng: &mut Rng| feet::roll(rng, style, quality);
        best(rng, roll, |form| {
            adopt(
                form,
                wanted,
                |f| (&mut f.construction, &mut f.material, Some(&mut f.finish)),
                |f| f.allowed(),
            )
        })
    });
    SetForm {
        kind,
        torso,
        head,
        neck,
        shoulders,
        arms,
        waist,
        hands,
        legs,
        feet,
    }
}

/// The torso piece: never padding alone (that is a gambeson) and always with a back;
/// rigid for a corslet, never rigid for a coat, where the style makes one.
fn roll_torso(rng: &mut Rng, kind: SetKind, style: Style, quality: Quality) -> TorsoForm {
    let roll = |rng: &mut Rng| torso::roll(rng, TorsoKind::Armour, Some(style), quality);
    let torso = best(rng, roll, |form| {
        2 * u8::from(form.construction != Construction::Padded) + u8::from(shaped(kind, form))
    });
    with_back(torso)
}

/// Whether a torso piece will do for a set of this kind: not padding alone, and rigid
/// or not as the kind is.
pub(crate) fn torso_fits(kind: SetKind, torso: &TorsoForm) -> bool {
    torso.construction != Construction::Padded && shaped(kind, torso)
}

fn shaped(kind: SetKind, torso: &TorsoForm) -> bool {
    match kind {
        SetKind::Coat => !torso.rigid(),
        SetKind::Corslet => torso.rigid(),
        SetKind::Half | SetKind::Full => true,
    }
}

/// With a back, wherever it can have one.
pub(crate) fn with_back(torso: TorsoForm) -> TorsoForm {
    let mut backed = torso;
    backed.back = true;
    if backed.allowed() { backed } else { torso }
}

/// The best scored of up to `TRIES` rolls, or the first to score the most possible.
/// A score may change the piece it is given, which is what is kept.
fn best<F: Copy>(
    rng: &mut Rng,
    mut roll: impl FnMut(&mut Rng) -> F,
    mut score: impl FnMut(&mut F) -> u8,
) -> F {
    let mut kept: Option<(u8, F)> = None;
    for _ in 0..TRIES {
        let mut form = roll(rng);
        let scored = score(&mut form);
        if kept.is_none_or(|(have, _)| scored > have) {
            kept = Some((scored, form));
        }
        if scored >= TOP {
            break;
        }
    }
    kept.expect("at least one roll").1
}

/// The most a piece can score: the set's construction, material and finish.
const TOP: u8 = 7;

/// Gives a piece the set's material, and its finish if it has one, where the piece
/// still makes sense with them, and scores it: 4 for the material, 2 for the
/// construction, 1 for the finish.
fn adopt<F: Copy>(
    form: &mut F,
    wanted: Wanted,
    fields: impl Fn(&mut F) -> (&mut Construction, &mut Material, Option<&mut Finish>),
    allowed: impl Fn(&F) -> bool,
) -> u8 {
    let mut score = 0;
    let mut tried = *form;
    let (construction, material, _) = fields(&mut tried);
    let same = *construction == wanted.construction;
    *material = wanted.material;
    if allowed(&tried) {
        *form = tried;
        score += 4;
    }
    let mut tried = *form;
    if let (_, _, Some(finish)) = fields(&mut tried) {
        *finish = wanted.finish;
        if allowed(&tried) {
            *form = tried;
            score += 1;
        }
    } else {
        score += 1;
    }
    score + 2 * u8::from(same)
}
