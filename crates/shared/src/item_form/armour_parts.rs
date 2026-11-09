//! What a request can ask of a piece of armour or footwear, reached the same way
//! whatever the piece: its style and construction, and giving it a material, finish
//! or dye where it still makes sense with them.

use super::ItemForm;
use super::armour::Piece;
use crate::armour::{Construction, Dye, Finish, Style};
use crate::footwear::sole_of;
use crate::material::Material;

impl ItemForm {
    /// Its style, for armour.
    pub(super) fn armour_style(&self) -> Option<Style> {
        Some(match self {
            Self::Arms(form) => form.style,
            Self::Hands(form) => form.style,
            Self::Shoulders(form) => form.style,
            Self::Torso(form) => form.style,
            Self::Head(form) => form.style,
            Self::Legs(form) => form.style,
            Self::Waist(form) => form.style,
            Self::Feet(form) => form.style,
            Self::Neck(form) => form.style,
            Self::Set(form) => form.style(),
            _ => return None,
        })
    }

    /// Its construction, for armour.
    pub(super) fn armour_construction(&self) -> Option<Construction> {
        Some(match self {
            Self::Arms(form) => form.construction,
            Self::Hands(form) => form.construction,
            Self::Shoulders(form) => form.construction,
            Self::Torso(form) => form.construction,
            Self::Head(form) => form.construction,
            Self::Legs(form) => form.construction,
            Self::Waist(form) => form.construction,
            Self::Feet(form) => form.construction,
            Self::Neck(form) => form.construction,
            Self::Set(form) => form.construction(),
            _ => return None,
        })
    }

    /// Gives it this material, if it still makes sense; footwear's upper, with the sole
    /// that goes with it. A set's comes from its torso when it is rolled, so not here.
    pub(super) fn adopt_material(&mut self, piece: Piece, material: Material) -> bool {
        let mut tried = self.clone();
        match &mut tried {
            Self::Arms(form) => form.material = material,
            Self::Hands(form) => form.material = material,
            Self::Shoulders(form) => form.material = material,
            Self::Torso(form) => form.material = material,
            Self::Head(form) => form.material = material,
            Self::Legs(form) => form.material = material,
            Self::Waist(form) => form.material = material,
            Self::Feet(form) => form.material = material,
            Self::Neck(form) => form.material = material,
            Self::Footwear(form) => {
                form.upper = material;
                form.sole = sole_of(form.build, material);
            }
            Self::Under(form) => form.material = material,
            _ => return false,
        }
        self.keep_if_allowed(tried, piece)
    }

    /// Gives it this finish, if it has one and it still makes sense.
    pub(super) fn adopt_finish(&mut self, piece: Piece, finish: Finish) -> bool {
        let mut tried = self.clone();
        match &mut tried {
            Self::Arms(form) => form.finish = finish,
            Self::Torso(form) => form.finish = finish,
            Self::Head(form) => form.finish = finish,
            Self::Legs(form) => form.finish = finish,
            Self::Waist(form) => form.finish = finish,
            Self::Feet(form) => form.finish = finish,
            Self::Neck(form) => form.finish = finish,
            _ => return false,
        }
        self.keep_if_allowed(tried, piece)
    }

    /// Dyes all its cloth, cover and lacing (a set's, every piece's) or footwear this
    /// colour, if it has any and it still makes sense. A second colour on footwear stays.
    pub(super) fn adopt_dye(&mut self, piece: Piece, dye: Dye) -> bool {
        let mut tried = self.clone();
        let dyes = tried.dyes_mut();
        if dyes.is_empty() {
            return false;
        }
        for each in dyes {
            *each = dye;
        }
        self.keep_if_allowed(tried, piece)
    }

    /// Everything dyed on it.
    fn dyes_mut(&mut self) -> Vec<&mut Dye> {
        let dyes = match self {
            Self::Arms(form) => vec![form.cover.as_mut()],
            Self::Torso(form) => vec![form.cover.as_mut(), form.lacing.as_mut()],
            Self::Head(form) => vec![form.cover.as_mut(), form.lacing.as_mut()],
            Self::Legs(form) => vec![form.cloth.as_mut()],
            Self::Waist(form) => vec![form.cloth.as_mut()],
            Self::Feet(form) => vec![form.cloth.as_mut()],
            Self::Neck(form) => vec![form.cloth.as_mut()],
            Self::Footwear(form) => vec![Some(&mut form.dye)],
            Self::Under(form) => vec![Some(&mut form.dye)],
            Self::Set(set) => {
                let set = &mut **set;
                let mut dyes = vec![set.torso.cover.as_mut(), set.torso.lacing.as_mut()];
                if let Some(head) = &mut set.head {
                    dyes.extend([head.cover.as_mut(), head.lacing.as_mut()]);
                }
                dyes.push(set.neck.as_mut().and_then(|neck| neck.cloth.as_mut()));
                dyes.push(set.arms.as_mut().and_then(|arms| arms.cover.as_mut()));
                dyes.push(set.waist.as_mut().and_then(|waist| waist.cloth.as_mut()));
                dyes.push(set.legs.as_mut().and_then(|legs| legs.cloth.as_mut()));
                dyes.push(set.feet.as_mut().and_then(|feet| feet.cloth.as_mut()));
                dyes
            }
            _ => Vec::new(),
        };
        dyes.into_iter().flatten().collect()
    }

    /// Takes `tried` if its piece's rules allow it.
    fn keep_if_allowed(&mut self, tried: Self, piece: Piece) -> bool {
        let allowed = match (&tried, piece) {
            (Self::Arms(form), _) => form.allowed(),
            (Self::Hands(form), Piece::Hands(hands)) => form.allowed(hands),
            (Self::Shoulders(form), _) => form.allowed(),
            (Self::Torso(form), _) => form.allowed(),
            (Self::Head(form), _) => form.allowed(),
            (Self::Legs(form), _) => form.allowed(),
            (Self::Waist(form), _) => form.allowed(),
            (Self::Feet(form), _) => form.allowed(),
            (Self::Neck(form), _) => form.allowed(),
            (Self::Footwear(form), _) => form.allowed(),
            (Self::Under(form), _) => form.allowed(),
            (Self::Set(form), _) => form.allowed(),
            _ => false,
        };
        if allowed {
            *self = tried;
        }
        allowed
    }
}
