//! How footwear may be finished: a boot's top edge, decoration, what is under the sole
//! and its colours.

use super::{Build, Decoration, Fastening, FootwearPiece, Height, Toe, Top, Under};
use crate::armour::Dye;
use crate::material::{Category, Material};

/// The top edges a boot may have; everything else is plain. Cuffs, buckets and
/// notches go on pull-on boots, scallops on pull-on or buttoned ones, and an uneven
/// top only with an upturned toe (gutals).
pub(super) fn tops(
    piece: FootwearPiece,
    build: Build,
    height: Height,
    toe: Toe,
    fastening: Fastening,
) -> Vec<(Top, u64)> {
    use Top as T;
    let list: &[(Top, u64)] = match (piece, build, height) {
        (FootwearPiece::Boots, _, Height::Thigh) => &[(T::Bucket, 1)],
        (FootwearPiece::Boots, Build::Soft, _) => &[(T::Plain, 50), (T::Fur, 50)],
        (FootwearPiece::Boots, Build::Sewn, Height::Ankle) => {
            &[(T::Plain, 75), (T::Scalloped, 15), (T::Cuff, 10)]
        }
        (FootwearPiece::Boots, Build::Sewn, Height::Calf) => &[
            (T::Plain, 45),
            (T::Scalloped, 30),
            (T::Uneven, 60),
            (T::Cuff, 10),
        ],
        (FootwearPiece::Boots, Build::Sewn, Height::Knee) => &[
            (T::Plain, 40),
            (T::Cuff, 20),
            (T::Notch, 15),
            (T::Bucket, 15),
            (T::Uneven, 60),
        ],
        _ => &[(T::Plain, 1)],
    };
    let fits = |top: Top| match top {
        T::Cuff | T::Bucket | T::Notch => fastening == Fastening::PullOn,
        T::Scalloped => matches!(fastening, Fastening::PullOn | Fastening::Buttons),
        T::Uneven => toe == Toe::Upturned,
        _ => true,
    };
    list.iter().copied().filter(|&(top, _)| fits(top)).collect()
}

/// The decorations it may have. Broguing goes on laced leather, a pompom on an
/// upturned toe, tassels on slip-ons and notched tops, jewels on buckles.
pub(super) fn decorations(
    build: Build,
    toe: Toe,
    fastening: Fastening,
    top: Top,
    upper: Material,
) -> Vec<(Decoration, u64)> {
    use Decoration as D;
    let list: &[(Decoration, u64)] = match build {
        Build::Sewn if toe == Toe::Broad => &[(D::Slashing, 60), (D::None, 30), (D::Jewels, 10)],
        // Juttis and tsarouchia.
        Build::Sewn if toe == Toe::Upturned => {
            &[(D::Embroidery, 40), (D::Pompom, 25), (D::None, 35)]
        }
        Build::Sewn => &[
            (D::None, 45),
            (D::Broguing, 12),
            (D::Embroidery, 8),
            (D::Beading, 2),
            (D::Slashing, 3),
            (D::Stitching, 12),
            (D::Tassels, 6),
            (D::Jewels, 3),
            (D::Fringe, 2),
        ],
        Build::Soft => &[
            (D::None, 40),
            (D::Beading, 25),
            (D::Embroidery, 20),
            (D::Fringe, 10),
            (D::Pompom, 5),
        ],
        Build::Rawhide => &[(D::None, 85), (D::Fringe, 15)],
        Build::Felted => &[(D::None, 70), (D::Embroidery, 30)],
        Build::Strapped => &[
            (D::None, 55),
            (D::Plaited, 30),
            (D::Stitching, 10),
            (D::Beading, 5),
        ],
        Build::Wooden => &[(D::None, 50), (D::Carved, 50)],
        Build::Woven => &[(D::None, 1)],
    };
    let leather = matches!(upper, Material::Leather | Material::Suede);
    let fits = |decoration: Decoration| match decoration {
        D::Broguing => fastening.laced() && leather,
        D::Pompom => toe == Toe::Upturned,
        D::Tassels => fastening == Fastening::PullOn || top == Top::Notch,
        D::Jewels => matches!(fastening, Fastening::Buckle | Fastening::Strap),
        D::Fringe => upper.category() == Category::Leather,
        _ => true,
    };
    list.iter()
        .copied()
        .filter(|&(decoration, _)| fits(decoration))
        .collect()
}

/// What may be under the sole: hobnails and irons on sewn soles and caligae, clog
/// irons on closed clogs.
pub(super) fn unders(
    piece: FootwearPiece,
    build: Build,
    height: Height,
    toe: Toe,
) -> &'static [(Under, u64)] {
    use Under as U;
    match (piece, build) {
        (FootwearPiece::Slippers, _) => &[(U::Nothing, 1)],
        (_, Build::Sewn) => &[(U::Nothing, 75), (U::Hobnails, 15), (U::Irons, 10)],
        (_, Build::Strapped) if height > Height::Low => &[(U::Hobnails, 70), (U::Nothing, 30)],
        (_, Build::Strapped) => &[(U::Nothing, 80), (U::Hobnails, 20)],
        (_, Build::Wooden) if toe != Toe::Open => &[(U::Nothing, 60), (U::Irons, 40)],
        _ => &[(U::Nothing, 1)],
    }
}

/// The colours its upper may be: hides and plants as they are, wood bare or painted,
/// leather in its usual dyes, cloth in any.
pub(super) fn dyes(upper: Material) -> &'static [(Dye, u64)] {
    use Dye as D;
    match upper {
        Material::Leather | Material::Suede | Material::BuffLeather => &[
            (D::Natural, 35),
            (D::Black, 25),
            (D::Brown, 25),
            (D::Red, 8),
            (D::White, 4),
            (D::Green, 3),
        ],
        Material::Wood => &[
            (D::Natural, 70),
            (D::Red, 8),
            (D::Blue, 8),
            (D::Yellow, 6),
            (D::Black, 4),
            (D::White, 4),
        ],
        _ if upper.category() == Category::Textile => &[
            (D::Natural, 30),
            (D::Red, 16),
            (D::Blue, 14),
            (D::Green, 8),
            (D::Black, 10),
            (D::Brown, 10),
            (D::White, 5),
            (D::Yellow, 5),
            (D::Purple, 2),
        ],
        _ => &[(D::Natural, 1)],
    }
}

/// Whether it has a second colour, and must: a top boot's cuff and slashing's lining
/// always do; laced leather shoes may (saddle shoes).
pub(super) fn contrast(
    piece: FootwearPiece,
    fastening: Fastening,
    top: Top,
    decoration: Decoration,
    upper: Material,
) -> (bool, bool) {
    let must = top == Top::Cuff || decoration == Decoration::Slashing;
    let may = must
        || (piece == FootwearPiece::Shoes
            && fastening.laced()
            && matches!(upper, Material::Leather | Material::Suede));
    (may, must)
}

/// The second colours there are: any but the first.
pub(super) fn contrasts(dye: Dye) -> Vec<(Dye, u64)> {
    [
        (Dye::White, 30),
        (Dye::Black, 20),
        (Dye::Brown, 20),
        (Dye::Red, 15),
        (Dye::Natural, 10),
        (Dye::Blue, 5),
    ]
    .into_iter()
    .filter(|&(contrast, _)| contrast != dye)
    .collect()
}
