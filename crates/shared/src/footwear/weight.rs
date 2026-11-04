//! What footwear weighs and takes up. Weights are the research's estimates for a pair
//! (all but the kolhapuri's 100 g are guesses), changed by the upper's material and
//! what is added: heels, nails, irons, wide tops.

use super::{Build, FootwearForm, FootwearPiece, Heel, Height, Toe, Top, Under};
use crate::creature::Size;
use crate::item::Shape;
use crate::material::{Category, Material};

impl FootwearForm {
    /// Grams for a plain pair of this kind, build and height in leather.
    fn base(&self) -> u32 {
        use Height as H;
        match (self.build, self.height) {
            (Build::Sewn, H::Low) if self.piece == FootwearPiece::Slippers => 350,
            (Build::Sewn, H::Low) => 600,
            (Build::Sewn, H::Ankle) => 1000,
            (Build::Sewn, H::Calf) => 1350,
            (Build::Sewn, H::Knee) => 1800,
            (Build::Sewn, H::Thigh) => 2400,
            (Build::Soft, H::Low) => 300,
            (Build::Soft, H::Ankle) => 500,
            (Build::Soft, H::Calf) => 700,
            (Build::Soft, _) => 900,
            (Build::Rawhide, _) => 300,
            (Build::Woven, _) => 250,
            (Build::Felted, H::Low) => 350,
            (Build::Felted, H::Calf) => 1150,
            (Build::Felted, _) => 1300,
            (Build::Strapped, H::Low) => 300,
            (Build::Strapped, H::Ankle) => 450,
            (Build::Strapped, _) => 550,
            (Build::Wooden, _) if self.piece == FootwearPiece::Boots => 1300,
            (Build::Wooden, _) if self.open() => 500,
            (Build::Wooden, _) if self.upper == Material::Wood => 1000,
            (Build::Wooden, _) => 900,
        }
    }

    /// Hundredths of the leather weight its upper makes it.
    fn upper_factor(&self) -> u32 {
        match self.upper {
            _ if self.build == Build::Wooden => 100,
            Material::Suede => 90,
            Material::Rawhide => 110,
            Material::Fur => 120,
            Material::Felt => 100,
            Material::Straw => 80,
            Material::Rattan | Material::Coir => 90,
            upper if upper.category() == Category::Textile => 70,
            _ => 100,
        }
    }

    /// Grams added for a pair's heels, nails, irons, toes and tops.
    fn extras(&self) -> u32 {
        let heel = match self.heel {
            Heel::Flat => 0,
            Heel::Low => 40,
            Heel::Riding => 80,
            Heel::Louis => 60,
            Heel::Teeth => 150,
        };
        let under = match self.under {
            Under::Nothing => 0,
            Under::Hobnails => 150,
            Under::Irons => 100,
        };
        let toe = match self.toe {
            Toe::LongPoint | Toe::Broad => 50,
            _ => 0,
        };
        let top = match self.top {
            Top::Bucket => 200,
            Top::Cuff | Top::Fur => 80,
            _ => 0,
        };
        heel + under + toe + top
    }

    /// The cells it takes up at an ordinary size, wide and long: low footwear 2 × 1,
    /// ankle and calf boots 2 × 2, knee and cavalier boots 2 × 3 (the owner's decision).
    pub fn shape(&self) -> Shape {
        match self.height {
            Height::Low => Shape::rectangle(2, 1),
            Height::Ankle | Height::Calf => Shape::rectangle(2, 2),
            Height::Knee | Height::Thigh => Shape::rectangle(2, 3),
        }
    }

    /// What the pair weighs at an ordinary size, in grams.
    pub fn grams(&self) -> u32 {
        self.base() * self.upper_factor() / 100 + self.extras()
    }

    /// What it weighs made at this size: bigger across, as thick (as foot armour), so
    /// by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
