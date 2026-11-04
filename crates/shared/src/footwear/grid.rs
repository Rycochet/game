//! Every pair of footwear that may be made: the grid rolls keep to, checked by the
//! tests.

use super::finish::{contrast, contrasts, decorations, dyes, tops, unders};
use super::holding::fastenings;
use super::materials::{sole_of, uppers};
use super::tables::{builds, has, heels, heights, may_be_backless, toes};
use super::{BROAD, FootwearForm, LONG_POINT, Toe};

impl FootwearForm {
    /// Whether these choices go together.
    pub fn allowed(&self) -> bool {
        let (piece, build, height, toe) = (self.piece, self.build, self.height, self.toe);
        let shape = has(builds(piece), build)
            && has(heights(piece, build), height)
            && within(self.shaft, height.shaft())
            && has(toes(piece, build, height), toe)
            && match toe {
                Toe::LongPoint => within(self.point, LONG_POINT),
                _ => self.point == 0,
            }
            && match toe {
                Toe::Broad => within(self.width, BROAD),
                _ => self.width == 100,
            }
            && has(heels(piece, build, height, toe), self.heel)
            && within(self.heel_mm, self.heel.millimetres())
            && (!self.backless || may_be_backless(piece, build));
        let fastening = self.fastening;
        let held = has(fastenings(piece, build, height, toe), fastening)
            && has(&tops(piece, build, height, toe, fastening), self.top)
            && has(uppers(piece, build, toe, fastening), self.upper)
            && self.sole == sole_of(build, self.upper);
        let decoration = self.decoration;
        let finished = has(
            &decorations(build, toe, fastening, self.top, self.upper),
            decoration,
        ) && has(unders(piece, build, height, toe), self.under)
            && has(dyes(self.upper), self.dye);
        let (may, must) = contrast(piece, fastening, self.top, decoration, self.upper);
        let coloured = match self.contrast {
            Some(second) => may && has(&contrasts(self.dye), second),
            None => !must,
        };
        shape && held && finished && coloured
    }
}

fn within<T: PartialOrd>(value: T, (least, most): (T, T)) -> bool {
    (least..=most).contains(&value)
}
