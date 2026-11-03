//! Which choices go together on shoulder armour.

use super::{
    CapeCut, Cut, Finish, HautePiece, Lacing, ShouldersForm, ShouldersPiece, Wings, adornments,
    constructions_in, finishes, laced, materials, may_have_trim, needs_cloth,
};
use crate::armour::{Construction, Style};

impl ShouldersForm {
    /// Every pair with these choices: the grid a roll keeps to.
    pub fn allowed(&self) -> bool {
        let kind = self.kind();
        let construction = self.construction;
        let has = |list: &[(Construction, u64)]| list.iter().any(|&(c, _)| c == construction);
        let piece_fits = match self.piece {
            ShouldersPiece::Pauldrons {
                cut,
                wings,
                lames,
                upper_lames,
                haute_piece,
                gardbrace,
                besagews,
            } => {
                let plate = construction == Construction::Plate;
                let hanging = !matches!(construction, Construction::Plate | Construction::Leather);
                (plate || cut == Cut::Matched)
                    && (!hanging || wings == Wings::Small)
                    && if hanging {
                        lames == 0
                    } else {
                        (3..=6).contains(&lames)
                    }
                    && (plate || upper_lames == 0)
                    && upper_lames <= 2
                    && (plate || haute_piece == HautePiece::None)
                    && (plate || !gardbrace)
                    && (!besagews || (wings == Wings::Small && !hanging))
            }
            ShouldersPiece::Spaulders {
                lames,
                besagews,
                pteruges,
            } => {
                let dendra = self.dendra();
                (if dendra {
                    lames == 0
                } else {
                    (2..=5).contains(&lames)
                }) && (!besagews
                    || matches!(construction, Construction::Plate | Construction::Leather)
                        && !dendra)
                    && (pteruges == 0
                        || matches!(construction, Construction::Laminar | Construction::Leather))
                    && pteruges <= 2
            }
            ShouldersPiece::Guards {
                size, rows, lacing, ..
            } => {
                let (fewest, most) = size.rows();
                (fewest..=most).contains(&rows) && (lacing == Lacing::Dense || laced(construction))
            }
            ShouldersPiece::Cape { cut, .. } => match cut {
                CapeCut::Round => {
                    matches!(construction, Construction::Mail | Construction::Scale)
                        || construction == Construction::Leather && self.finish == Finish::RingMail
                }
                CapeCut::Yoke => {
                    matches!(construction, Construction::Padded | Construction::Leather)
                        && self.finish != Finish::RingMail
                        && self.style == Style::Classical
                }
            },
            ShouldersPiece::Discs { diameter, .. } => {
                (110..=160).contains(&diameter) && diameter % 10 == 0
            }
            ShouldersPiece::Ailettes { heraldry, .. } => heraldry.allowed(),
            ShouldersPiece::Wing => true,
        };
        has(&constructions_in(kind, self.style))
            && piece_fits
            && adornments(kind, construction)
                .iter()
                .any(|&(a, _)| a == self.adornment)
            && finishes(kind, construction, &self.piece)
                .iter()
                .any(|&(f, _)| f == self.finish)
            && materials(kind, construction, self.style)
                .iter()
                .any(|&(m, _)| m == self.material)
            && self.cloth.is_some() == needs_cloth(construction)
            && (self.trim.is_none() || may_have_trim(kind, construction, self.material))
    }
}
