//! What footwear may be made of: the upper is rolled, and the sole follows from it and
//! how it is made.

use super::{Build, Fastening, FootwearPiece, Toe};
use crate::material::Material;

/// What the upper (or the straps, or a wooden sole's thong) may be made of.
pub(super) fn uppers(
    piece: FootwearPiece,
    build: Build,
    toe: Toe,
    fastening: Fastening,
) -> &'static [(Material, u64)] {
    use FootwearPiece as P;
    use Material as M;
    match (build, piece) {
        (Build::Sewn, P::Slippers) => &[
            (M::Leather, 35),
            (M::Silk, 20),
            (M::Wool, 15),
            (M::Cotton, 15),
            (M::Linen, 5),
            (M::Suede, 10),
        ],
        (Build::Sewn, P::Boots) => &[(M::Leather, 70), (M::Suede, 20), (M::BuffLeather, 10)],
        (Build::Sewn, _) => &[
            (M::Leather, 60),
            (M::Suede, 15),
            (M::BuffLeather, 5),
            (M::Cotton, 6),
            (M::Silk, 6),
            (M::Wool, 4),
            (M::Linen, 4),
        ],
        (Build::Soft, P::Boots) => &[(M::Leather, 40), (M::Suede, 15), (M::Fur, 45)],
        (Build::Soft, P::Slippers) => &[
            (M::Leather, 40),
            (M::Suede, 20),
            (M::Fur, 20),
            (M::Wool, 20),
        ],
        (Build::Soft, _) => &[(M::Leather, 55), (M::Suede, 30), (M::Fur, 15)],
        (Build::Rawhide, _) => &[(M::Rawhide, 1)],
        // Espadrilles have canvas uppers on rope soles.
        (Build::Woven, P::Shoes) => &[(M::Straw, 60), (M::Cotton, 40)],
        (Build::Woven, _) => &[(M::Straw, 70), (M::Rattan, 15), (M::Coir, 15)],
        (Build::Felted, _) => &[(M::Felt, 1)],
        (Build::Strapped, _) => &[(M::Leather, 80), (M::Rawhide, 20)],
        (Build::Wooden, P::Clogs) if toe == Toe::Open => match fastening {
            // Geta's cloth thongs, padukas' wooden knobs.
            Fastening::ToeThong => &[(M::Cotton, 55), (M::Wood, 45)],
            _ => &[(M::Leather, 45), (M::Cotton, 30), (M::Rattan, 25)],
        },
        // Sabots, carved whole.
        (Build::Wooden, P::Clogs) if fastening == Fastening::PullOn => {
            &[(M::Wood, 70), (M::Leather, 30)]
        }
        (Build::Wooden, _) => &[(M::Leather, 1)],
    }
}

/// What the sole is made of: leather under sewn and strapped footwear, the upper's own
/// under soft and woven, rope under espadrilles, wood under clogs.
pub(super) fn sole_of(build: Build, upper: Material) -> Material {
    match build {
        Build::Sewn | Build::Strapped => Material::Leather,
        Build::Soft => match upper {
            Material::Fur | Material::Wool => Material::Leather,
            upper => upper,
        },
        Build::Woven if upper == Material::Cotton => Material::Straw,
        Build::Woven | Build::Rawhide | Build::Felted => upper,
        Build::Wooden => Material::Wood,
    }
}
