//! What footwear may be shaped like, and how often: the lists rolls pick from and the
//! grid checks against. Weights are first guesses from the research's styles.

use super::{Build, FootwearPiece, Heel, Height, Toe};

/// How each kind of footwear is made.
pub(super) fn builds(piece: FootwearPiece) -> &'static [(Build, u64)] {
    use Build as B;
    match piece {
        FootwearPiece::Shoes => &[
            (B::Sewn, 60),
            (B::Soft, 20),
            (B::Rawhide, 8),
            (B::Woven, 12),
        ],
        FootwearPiece::Boots => &[
            (B::Sewn, 70),
            (B::Soft, 15),
            (B::Felted, 10),
            (B::Wooden, 5),
        ],
        FootwearPiece::Sandals => &[(B::Strapped, 75), (B::Woven, 25)],
        FootwearPiece::Clogs => &[(B::Wooden, 1)],
        FootwearPiece::Slippers => &[(B::Sewn, 50), (B::Soft, 40), (B::Felted, 10)],
    }
}

/// How high it comes. Only boots and sandal boots (caligae) rise over the ankle, and
/// only the cavalier boot over the knee.
pub(super) fn heights(piece: FootwearPiece, build: Build) -> &'static [(Height, u64)] {
    use Height as H;
    match (piece, build) {
        (FootwearPiece::Boots, Build::Sewn) => {
            &[(H::Ankle, 40), (H::Calf, 22), (H::Knee, 33), (H::Thigh, 5)]
        }
        (FootwearPiece::Boots, Build::Soft) => &[(H::Ankle, 35), (H::Calf, 45), (H::Knee, 20)],
        (FootwearPiece::Boots, Build::Felted) => &[(H::Calf, 50), (H::Knee, 50)],
        (FootwearPiece::Boots, _) => &[(H::Ankle, 1)],
        (FootwearPiece::Sandals, Build::Strapped) => &[(H::Low, 85), (H::Ankle, 10), (H::Calf, 5)],
        _ => &[(H::Low, 1)],
    }
}

/// The toes it may have. Long points and broad toes only on sewn shoes (and long points
/// on ankle boots), as in the 14th to 16th centuries.
pub(super) fn toes(piece: FootwearPiece, build: Build, height: Height) -> &'static [(Toe, u64)] {
    use FootwearPiece as P;
    use Toe as T;
    match (piece, build, height) {
        (P::Sandals, _, _) => &[(T::Open, 1)],
        (P::Clogs, _, _) => &[
            (T::Round, 40),
            (T::Upturned, 15),
            (T::Pointed, 10),
            (T::Open, 35),
        ],
        (_, Build::Rawhide, _) => &[(T::Round, 60), (T::Upturned, 40)],
        (P::Shoes, Build::Soft, _) => &[(T::Round, 55), (T::Pointed, 15), (T::Upturned, 30)],
        (P::Boots, Build::Soft, _) => &[(T::Round, 70), (T::Upturned, 30)],
        (P::Slippers, Build::Soft, _) => &[(T::Round, 50), (T::Pointed, 30), (T::Upturned, 20)],
        (P::Slippers, Build::Sewn, _) => &[
            (T::Round, 35),
            (T::Almond, 15),
            (T::Pointed, 35),
            (T::Upturned, 15),
        ],
        (P::Shoes, Build::Sewn, _) => &[
            (T::Round, 32),
            (T::Almond, 22),
            (T::Pointed, 12),
            (T::LongPoint, 8),
            (T::Square, 12),
            (T::Broad, 8),
            (T::Upturned, 6),
        ],
        (P::Boots, Build::Sewn, Height::Ankle) => &[
            (T::Round, 45),
            (T::Almond, 25),
            (T::Pointed, 15),
            (T::LongPoint, 5),
            (T::Square, 10),
        ],
        (P::Boots, Build::Sewn, Height::Calf) => &[
            (T::Round, 40),
            (T::Almond, 20),
            (T::Pointed, 25),
            (T::Upturned, 15),
        ],
        (P::Boots, Build::Sewn, Height::Knee) => &[
            (T::Round, 50),
            (T::Almond, 20),
            (T::Square, 20),
            (T::Upturned, 10),
        ],
        (P::Boots, Build::Sewn, Height::Thigh) => &[(T::Round, 60), (T::Square, 40)],
        _ => &[(T::Round, 1)],
    }
}

/// The heels it may have: only sewn footwear has a stacked heel, never with the
/// medieval toes, and only wooden soles stand on teeth.
pub(super) fn heels(
    piece: FootwearPiece,
    build: Build,
    height: Height,
    toe: Toe,
) -> &'static [(Heel, u64)] {
    use Heel as E;
    if build == Build::Wooden && toe == Toe::Open {
        return &[(E::Flat, 40), (E::Teeth, 60)];
    }
    if build != Build::Sewn || matches!(toe, Toe::LongPoint | Toe::Broad | Toe::Upturned) {
        return &[(E::Flat, 1)];
    }
    match (piece, height) {
        (_, Height::Thigh) => &[(E::Riding, 1)],
        (FootwearPiece::Slippers, _) => &[(E::Flat, 75), (E::Louis, 25)],
        (_, Height::Ankle) => &[(E::Flat, 20), (E::Low, 50), (E::Riding, 10), (E::Louis, 20)],
        (_, Height::Calf) => &[(E::Flat, 15), (E::Low, 35), (E::Riding, 45), (E::Louis, 5)],
        (_, Height::Knee) => &[(E::Flat, 10), (E::Low, 45), (E::Riding, 30), (E::Louis, 15)],
        _ => &[(E::Flat, 25), (E::Low, 50), (E::Louis, 25)],
    }
}

/// Only slippers may leave the heel bare: mules and babouches.
pub(super) fn may_be_backless(piece: FootwearPiece, build: Build) -> bool {
    piece == FootwearPiece::Slippers && matches!(build, Build::Sewn | Build::Soft)
}

/// Whether a list offers this, at a weight above nothing.
pub(super) fn has<T: PartialEq>(list: &[(T, u64)], wanted: T) -> bool {
    list.iter()
        .any(|(have, weight)| *have == wanted && *weight > 0)
}
