//! Rolling a head piece, each choice from what the ones before allow, so nothing is
//! thrown away.

use super::materials::{COIFS, materials};
use super::table::{Cheeks, ROWS, Row};
use super::{Back, Bowl, Brim, Face, HeadForm, HeadKind, Ornament, Skull, Top};
use crate::armour::{Construction, Dye, Finish, Style, in_style};
use crate::item::Quality;
use crate::material::{Category, Material};
use crate::rng::Rng;

/// One in so many.
fn one_in(rng: &mut Rng, chance: u64) -> bool {
    rng.range_u64(0, chance) == 0
}

/// Any of a list, alike.
fn any<T: Copy>(rng: &mut Rng, list: &[T]) -> T {
    list[rng.range_u64(0, list.len() as u64) as usize]
}

/// Rolls a piece of a kind. A set passes its style; a piece on its own rolls one.
pub fn roll(rng: &mut Rng, kind: HeadKind, style: Option<Style>, quality: Quality) -> HeadForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    match kind {
        HeadKind::Cap => soft(rng, kind, Construction::Padded, style),
        HeadKind::Coif => {
            let construction = rng.pick_weighted(&in_style(&COIFS, style));
            soft(rng, kind, construction, style)
        }
        HeadKind::Helmet | HeadKind::BevoredHelm => helmet(rng, kind, style, quality),
    }
}

/// The rows a helmet of this kind may be rolled from in this style: those made in it,
/// or every row of the kind where none are.
pub(super) fn rows_for(kind: HeadKind, style: Style) -> Vec<(usize, u64)> {
    let bevor = kind == HeadKind::BevoredHelm;
    let of_kind = || {
        ROWS.iter()
            .enumerate()
            .filter(move |(_, row)| row.bevor == bevor)
    };
    let in_style: Vec<_> = of_kind()
        .filter(|(_, row)| row.styles.contains(&style))
        .map(|(at, row)| (at, row.weight))
        .collect();
    if in_style.is_empty() {
        of_kind().map(|(at, row)| (at, row.weight)).collect()
    } else {
        in_style
    }
}

/// A row's constructions this style allows, or all of them if it allows none.
pub(super) fn constructions(row: &Row, style: Style) -> Vec<Construction> {
    let allowed: Vec<_> = row
        .constructions
        .iter()
        .copied()
        .filter(|&construction| style.allows(construction))
        .collect();
    if allowed.is_empty() {
        row.constructions.to_vec()
    } else {
        allowed
    }
}

/// Whether a bowl can be built this way.
pub(super) fn bowl_fits(construction: Construction, bowl: Bowl) -> bool {
    match construction {
        Construction::Leather => bowl == Bowl::Moulded,
        Construction::Woven => bowl == Bowl::Woven,
        Construction::Lamellar => matches!(bowl, Bowl::Laced | Bowl::Plates),
        _ => matches!(
            bowl,
            Bowl::Raised | Bowl::Ridged | Bowl::Segmented | Bowl::Plates
        ),
    }
}

fn helmet(rng: &mut Rng, kind: HeadKind, style: Style, quality: Quality) -> HeadForm {
    let row = &ROWS[rng.pick_weighted(&rows_for(kind, style))];
    let construction = any(rng, &constructions(row, style));
    let material = rng.pick_weighted(materials(construction, style));
    let bowls: Vec<_> = row
        .bowls
        .iter()
        .copied()
        .filter(|&bowl| bowl_fits(construction, bowl))
        .collect();
    let bowl = any(rng, &bowls);
    let skull = any(rng, row.skulls);
    let face = any(rng, row.faces);
    let cheeks = match row.cheeks {
        Cheeks::Either => one_in(rng, 2),
        cheeks => cheeks == Cheeks::Yes,
    };
    let top = any(rng, row.tops);
    let mut form = HeadForm {
        kind,
        construction,
        style,
        material,
        bowl,
        skull,
        face,
        cheeks,
        back: any(rng, row.backs),
        brim: any(rng, row.brims),
        top,
        ornament: ornament(rng, style, skull, top),
        studded: false,
        finish: Finish::Bright,
        cover: matches!(top, Top::Crest | Top::Plume).then(|| Dye::roll_paint(rng)),
        lacing: None,
    };
    form.studded = construction == Construction::Leather && !form.tusks() && one_in(rng, 3);
    form.finish = finish(rng, &form, quality);
    form.lacing = form.laced().then(|| Dye::roll_paint(rng));
    form
}

/// Bright metal is finished as its style has it; bronze and copper stay bright, and
/// leather is sometimes lacquered (always in Japan).
fn finish(rng: &mut Rng, form: &HeadForm, quality: Quality) -> Finish {
    match form.material {
        Material::Bronze | Material::Copper => Finish::Bright,
        material if material.category() == Category::Metal => {
            Finish::roll(rng, form.style, quality)
        }
        Material::HardenedLeather | Material::Leather
            if form.style == Style::Japanese || one_in(rng, 4) =>
        {
            Finish::Lacquered(Dye::roll_paint(rng))
        }
        _ => Finish::Bright,
    }
}

/// Fantasy pieces and European ones now and then carry horns, wings or a crown; a
/// Japanese helmet often has a crest at the brow.
fn ornament(rng: &mut Rng, style: Style, skull: Skull, top: Top) -> Ornament {
    use Ornament as O;
    let list: &[(Ornament, u64)] = match style {
        Style::Japanese => &[(O::FrontCrest, 60), (O::None, 40)],
        Style::Elven => &[(O::Wings, 20), (O::Crown, 5), (O::None, 75)],
        Style::Dwarven => &[(O::Horns, 20), (O::Crown, 5), (O::None, 75)],
        _ if style.european() => &[
            (O::None, 400),
            (O::Horns, 6),
            (O::Wings, 3),
            (O::Crown, 3),
            (O::Antlers, 1),
            (O::Dragon, 1),
        ],
        _ => &[(O::None, 1)],
    };
    let fits: Vec<_> = list
        .iter()
        .copied()
        .filter(|&(ornament, _)| ornament_fits(ornament, skull, top))
        .collect();
    rng.pick_weighted(&fits)
}

/// Horns and antlers need room at the temples, and a dragon the top to itself.
pub(super) fn ornament_fits(ornament: Ornament, skull: Skull, top: Top) -> bool {
    match ornament {
        Ornament::Horns | Ornament::Antlers => skull != Skull::Onion,
        Ornament::Dragon => top == Top::None,
        _ => true,
    }
}

/// A cap or coif: no bowl, open, nothing added. Padding is undyed half the time.
fn soft(rng: &mut Rng, kind: HeadKind, construction: Construction, style: Style) -> HeadForm {
    let material = rng.pick_weighted(materials(construction, style));
    let padded = construction == Construction::Padded;
    let cover = padded.then(|| {
        if one_in(rng, 2) {
            Dye::Natural
        } else {
            Dye::roll(rng)
        }
    });
    let finish = if construction == Construction::Mail && material != Material::Bronze {
        rng.pick_weighted(&[
            (Finish::Bright, 75),
            (Finish::Blackened, 17),
            (Finish::Tinned, 8),
        ])
    } else {
        Finish::Bright
    };
    HeadForm {
        kind,
        construction,
        style,
        material,
        bowl: Bowl::Soft,
        skull: Skull::Round,
        face: Face::Open,
        cheeks: false,
        back: Back::None,
        brim: Brim::None,
        top: Top::None,
        ornament: Ornament::None,
        studded: false,
        finish,
        cover,
        lacing: None,
    }
}
