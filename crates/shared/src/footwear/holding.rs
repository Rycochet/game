//! How footwear may be held on, from what it is and its shape.

use super::{Build, Fastening, FootwearPiece, Height, Toe};

/// The fastenings it may have.
pub(super) fn fastenings(
    piece: FootwearPiece,
    build: Build,
    height: Height,
    toe: Toe,
) -> &'static [(Fastening, u64)] {
    use Fastening as F;
    use FootwearPiece as P;
    match (piece, build) {
        (P::Slippers, _) | (_, Build::Felted) => &[(F::PullOn, 1)],
        (P::Sandals, Build::Strapped) if height == Height::Low => &[
            (F::Buckle, 30),
            (F::ToeThong, 35),
            (F::Loops, 20),
            (F::Strap, 15),
        ],
        // Caligae, laced up the shin.
        (P::Sandals, Build::Strapped) => &[(F::Thongs, 70), (F::Loops, 30)],
        (P::Sandals, _) => &[(F::ToeThong, 55), (F::Thongs, 45)],
        // Geta and padukas, nalins and bakya.
        (P::Clogs, _) if toe == Toe::Open => &[(F::ToeThong, 60), (F::Strap, 40)],
        (P::Clogs, _) => &[(F::PullOn, 60), (F::OpenLaces, 25), (F::Buckle, 15)],
        (_, Build::Wooden) => &[(F::OpenLaces, 1)],
        (_, Build::Rawhide) => &[(F::Thongs, 60), (F::Loops, 40)],
        (_, Build::Woven) => &[(F::Thongs, 60), (F::PullOn, 40)],
        (_, Build::Soft) => soft(height),
        (P::Shoes, _) => sewn_shoes(toe),
        _ => sewn_boots(height, toe),
    }
}

fn soft(height: Height) -> &'static [(Fastening, u64)] {
    use Fastening as F;
    match height {
        Height::Low => &[
            (F::PullOn, 45),
            (F::Toggles, 30),
            (F::Thongs, 10),
            (F::OpenLaces, 15),
        ],
        Height::Ankle => &[
            (F::Toggles, 40),
            (F::OpenLaces, 30),
            (F::Drawstring, 15),
            (F::PullOn, 15),
        ],
        _ => &[(F::Drawstring, 50), (F::PullOn, 40), (F::Thongs, 10)],
    }
}

fn sewn_shoes(toe: Toe) -> &'static [(Fastening, u64)] {
    use Fastening as F;
    match toe {
        // Duckbill shoes, strapped over the instep.
        Toe::Broad => &[(F::Strap, 50), (F::Buckle, 30), (F::PullOn, 20)],
        Toe::LongPoint => &[(F::Buckle, 40), (F::OpenLaces, 30), (F::PullOn, 30)],
        Toe::Upturned => &[(F::PullOn, 1)],
        _ => &[
            (F::PullOn, 22),
            (F::ClosedLaces, 22),
            (F::OpenLaces, 22),
            (F::Loops, 6),
            (F::Buckle, 22),
            (F::Strap, 6),
        ],
    }
}

fn sewn_boots(height: Height, toe: Toe) -> &'static [(Fastening, u64)] {
    use Fastening as F;
    if toe == Toe::Upturned {
        return &[(F::PullOn, 1)];
    }
    match height {
        Height::Ankle if toe == Toe::LongPoint => &[(F::OpenLaces, 50), (F::Buckle, 50)],
        Height::Ankle => &[
            (F::OpenLaces, 35),
            (F::ClosedLaces, 10),
            (F::PullOn, 10),
            (F::ElasticSides, 15),
            (F::Buckle, 15),
            (F::Buttons, 15),
        ],
        Height::Calf => &[
            (F::PullOn, 50),
            (F::OpenLaces, 15),
            (F::Buckle, 15),
            (F::Thongs, 15),
            (F::Buttons, 5),
        ],
        Height::Knee => &[
            (F::PullOn, 75),
            (F::OpenLaces, 10),
            (F::Buckle, 10),
            (F::Buttons, 5),
        ],
        _ => &[(F::PullOn, 1)],
    }
}
