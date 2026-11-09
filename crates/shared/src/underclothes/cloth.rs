//! What underclothes are made of, how fine the cloth is, how they are decorated and
//! dyed: each list goes by the garment rolled before it.

use serde::{Deserialize, Serialize};

use super::table::when;
use super::{BandForm, CoifForm, Garment, HoseMake, LoinForm, Neck, SkirtMake, SockMake};
use crate::armour::Dye;
use crate::material::Material;

/// How the cloth is woven (owner, 2026-11-09: lawn and net as finishes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cloth {
    Plain,
    /// Fine linen.
    Lawn,
    /// The net of a hairnet or caul.
    Net,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Decoration {
    Plain,
    /// Embroidered: blackwork at the collar and cuffs, or bright silks.
    Embroidered,
    /// A lace edge (16th century).
    Lace,
    /// Gold thread: a caul, Rare and better.
    GoldThread,
}

use Material::{Cotton, Felt, Haircloth, Hemp, Leather, Linen, Ramie, Silk, Wool};

/// The materials it can be made of, the usual one first.
pub(super) fn materials(garment: &Garment) -> Vec<(Material, u64)> {
    match garment {
        Garment::Shirt(shirt) if shirt.neck == Neck::Crossover => {
            vec![
                (Cotton, 40),
                (Silk, 30),
                (Ramie, 15),
                (Hemp, 10),
                (Linen, 5),
            ]
        }
        Garment::Shirt(shirt) => vec![
            (Linen, 60),
            (Hemp, 10),
            (Cotton, 8),
            (Silk, 8),
            (Wool, 6),
            (Ramie, 3),
            (Haircloth, when(shirt.may_be_haircloth(), 5)),
        ],
        Garment::Braies(_) => vec![(Linen, 70), (Hemp, 12), (Wool, 8), (Cotton, 6), (Silk, 4)],
        Garment::Band(band) if band.form == BandForm::Panel => {
            vec![(Silk, 50), (Cotton, 40), (Linen, 10)]
        }
        Garment::Band(band) => vec![
            (Linen, 55),
            (Cotton, 20),
            (Silk, 15),
            (Wool, 5),
            (
                Leather,
                when(matches!(band.form, BandForm::Wound | BandForm::Twisted), 5),
            ),
        ],
        Garment::Loincloth(loin) if loin.form.cut() => {
            vec![(Linen, 50), (Leather, 30), (Cotton, 10), (Wool, 10)]
        }
        Garment::Loincloth(loin) => vec![
            (Cotton, 45),
            (Linen, 35),
            (Silk, when(loin.form != LoinForm::Narrow, 10)),
            (Hemp, 10),
        ],
        Garment::Hose(hose) => match hose.make {
            HoseMake::BiasCut => vec![(Wool, 70), (Linen, 15), (Silk, 10), (Cotton, 5)],
            HoseMake::Leather => vec![(Leather, 1)],
            HoseMake::Knitted => vec![(Wool, 60), (Silk, 30), (Cotton, 10)],
        },
        Garment::Socks(socks) => match socks.make {
            SockMake::Sewn => vec![(Wool, 45), (Linen, 25), (Cotton, 20), (Silk, 10)],
            SockMake::Nalbound => vec![(Wool, 1)],
            SockMake::Felt => vec![(Felt, 1)],
            SockMake::Wrapped => vec![(Linen, 40), (Wool, 40), (Cotton, 20)],
            SockMake::Knitted => vec![(Wool, 70), (Silk, 15), (Cotton, 15)],
        },
        Garment::Petticoat(skirt) => match skirt.make {
            SkirtMake::Horsehair => vec![(Haircloth, 1)],
            SkirtMake::Quilted => vec![(Wool, 50), (Silk, 30), (Linen, 20)],
            SkirtMake::Plain => vec![(Linen, 35), (Wool, 35), (Cotton, 12), (Silk, 12), (Hemp, 6)],
        },
        Garment::Coif(coif) => match coif.form {
            CoifForm::Net => vec![(Silk, 60), (Linen, 40)],
            CoifForm::BrimmedNightcap => vec![(Linen, 70), (Silk, 30)],
            CoifForm::PointedNightcap | CoifForm::TiedNightcap => {
                vec![(Linen, 50), (Wool, 30), (Cotton, 20)]
            }
            _ => vec![(Linen, 80), (Silk, 10), (Cotton, 10)],
        },
        Garment::Wimple(_) => vec![(Linen, 85), (Silk, 15)],
    }
}

/// Lawn for some linen; net for hairnets and cauls.
pub(super) fn cloths(garment: &Garment, material: Material) -> Vec<(Cloth, u64)> {
    let fine = match garment {
        Garment::Coif(coif) if coif.form == CoifForm::Net => return vec![(Cloth::Net, 1)],
        Garment::Shirt(_) | Garment::Band(_) | Garment::Wimple(_) => true,
        Garment::Coif(coif) => matches!(coif.form, CoifForm::Coif | CoifForm::Undercap),
        _ => false,
    };
    vec![
        (Cloth::Plain, 3),
        (Cloth::Lawn, when(fine && material == Linen, 1)),
    ]
}

/// How it may be decorated. Gold thread is only rolled at Rare and better (`roll`).
pub(super) fn decorations(garment: &Garment, material: Material) -> Vec<(Decoration, u64)> {
    use Decoration::{Embroidered, GoldThread, Lace, Plain};
    let haircloth = material == Haircloth || material == Leather;
    match garment {
        _ if haircloth => vec![(Plain, 1)],
        Garment::Shirt(shirt) => {
            let lace = matches!(shirt.neck, Neck::Gathered | Neck::Collar);
            vec![(Plain, 70), (Embroidered, 20), (Lace, when(lace, 8))]
        }
        Garment::Coif(coif) => match coif.form {
            CoifForm::Net => vec![(Plain, 70), (GoldThread, 30)],
            CoifForm::BrimmedNightcap => vec![(Plain, 40), (Embroidered, 60)],
            CoifForm::Coif | CoifForm::Undercap => vec![(Plain, 70), (Embroidered, 20), (Lace, 10)],
            _ => vec![(Plain, 80), (Embroidered, 20)],
        },
        Garment::Band(band) if band.form == BandForm::Panel => vec![(Plain, 50), (Embroidered, 50)],
        Garment::Band(_) | Garment::Wimple(_) => vec![(Plain, 88), (Embroidered, 12)],
        Garment::Petticoat(skirt) if skirt.make == SkirtMake::Plain => {
            vec![(Plain, 70), (Embroidered, 20), (Lace, 10)]
        }
        Garment::Socks(socks) if socks.make == SockMake::Knitted => {
            vec![(Plain, 70), (Embroidered, 30)]
        }
        Garment::Loincloth(loin) if !loin.form.cut() => vec![(Plain, 85), (Embroidered, 15)],
        _ => vec![(Plain, 1)],
    }
}

/// Its colours: underlinen mostly undyed or white, hose, socks and petticoats brighter.
pub(super) fn dyes(garment: &Garment, material: Material) -> Vec<(Dye, u64)> {
    use Dye::{Black, Blue, Brown, Green, Natural, Purple, Red, White, Yellow};
    match material {
        Haircloth => return vec![(Natural, 4), (Black, 2), (Brown, 2)],
        Leather | Felt => return vec![(Natural, 5), (Brown, 3), (Black, 2), (Red, 1)],
        _ => {}
    }
    match garment {
        Garment::Hose(_) | Garment::Socks(_) | Garment::Petticoat(_) => vec![
            (Natural, 25),
            (Red, 16),
            (Blue, 14),
            (Black, 12),
            (Brown, 10),
            (Green, 8),
            (White, 6),
            (Yellow, 6),
            (Purple, 3),
        ],
        _ => vec![
            (Natural, 50),
            (White, 35),
            (Black, 4),
            (Blue, 3),
            (Red, 3),
            (Yellow, 3),
            (Purple, 1),
        ],
    }
}

/// A second colour for striped or parti-coloured hose: any but the first.
pub(super) fn contrasts(dye: Dye) -> Vec<(Dye, u64)> {
    [
        (Dye::White, 20),
        (Dye::Red, 20),
        (Dye::Blue, 18),
        (Dye::Black, 15),
        (Dye::Green, 12),
        (Dye::Yellow, 10),
        (Dye::Natural, 5),
    ]
    .into_iter()
    .filter(|&(second, _)| second != dye)
    .collect()
}
