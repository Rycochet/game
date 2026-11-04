//! What a set weighs: its pieces together.

use super::SetForm;
use crate::creature::Size;

impl SetForm {
    /// Its weight made at this size: every piece's, at the same size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let pieces = [
            Some(self.torso.grams_at(size)),
            self.head.map(|piece| piece.grams_at(size)),
            self.neck.map(|piece| piece.grams_at(size)),
            self.shoulders.map(|piece| piece.grams_at(size)),
            self.arms.map(|piece| piece.grams_at(size)),
            self.waist.map(|piece| piece.grams_at(size)),
            self.hands.map(|piece| piece.grams_at(size)),
            self.legs.map(|piece| piece.grams_at(size)),
            self.feet.map(|piece| piece.grams_at(size)),
        ];
        pieces.into_iter().flatten().sum()
    }

    /// Whether each piece makes sense: what a set's tests check.
    pub fn allowed(&self) -> bool {
        use crate::armour::hands::HandsPiece;
        self.torso.allowed()
            && self.head.is_none_or(|piece| piece.allowed())
            && self.neck.is_none_or(|piece| piece.allowed())
            && self.shoulders.is_none_or(|piece| piece.allowed())
            && self.arms.is_none_or(|piece| piece.allowed())
            && self.waist.is_none_or(|piece| piece.allowed())
            && self
                .hands
                .is_none_or(|piece| piece.allowed(HandsPiece::Gauntlets))
            && self.legs.is_none_or(|piece| piece.allowed())
            && self.feet.is_none_or(|piece| piece.allowed())
    }
}
