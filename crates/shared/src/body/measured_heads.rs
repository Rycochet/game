//! Animals' heads as measured from MakeHuman's meshes by `cargo xtask shapes`, which
//! `cargo xtask test` checks these figures still match: JALdMIC's wolf, cow and dog
//! heads (each on a person's body) and the horse's own (CC-BY; see `docs/credits.md`).
//! Every figure is in thousandths of the head's length from the back of the skull to
//! the nose, as the tool prints it, so only the proportions come from the meshes: each
//! head is scaled to our own animal's (`MeasuredHead::shape`).

use super::head_shape::HeadShape;
use super::measured::MeasuredRing;
use super::rings::{HEAD_RINGS, Ring};

/// A head as measured: how far its line from where it sits on the neck to the nose
/// leans nose down, and its rings from the back of the skull to the nose. A ring's
/// `belly` and `back` are how far up from the head's line its underside and top are,
/// as the tool prints them (an underside is below, so negative).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeasuredHead {
    /// The mesh it was measured from, under `spare_assets/makehuman/`.
    pub source: &'static str,
    /// Its length in the mesh, in millimetres: heads on the same person's body
    /// (`on_frame`) compare by it.
    pub length: i32,
    pub pitch: i32,
    pub rings: [MeasuredRing; HEAD_RINGS],
}

impl MeasuredHead {
    /// Its shape on a head of ours this long from the back of the skull to the nose,
    /// leaning `pitch` degrees nose down (our own carriage, not the mesh's).
    pub fn shape(&self, length: i32, pitch: i32) -> HeadShape {
        let long = |share: i32| share * length / 1000;
        HeadShape {
            pitch,
            rings: self.rings.map(|ring| Ring {
                along: long(ring.along),
                half: long(ring.half),
                belly: -long(ring.belly),
                back: long(ring.back),
            }),
        }
    }
}

const fn ring(along: i32, half: i32, belly: i32, back: i32) -> MeasuredRing {
    MeasuredRing {
        along,
        half,
        belly,
        back,
    }
}

/// The wolf's: a broad skull and cheeks, a long muzzle tapering to the nose.
pub const WOLF_HEAD: MeasuredHead = MeasuredHead {
    source: "animal03/jaldmic_wolf_headv3.target",
    length: 257,
    pitch: 1,
    rings: [
        ring(-172, 32, 104, 124),
        ring(-32, 258, -324, 399),
        ring(108, 331, -324, 454),
        ring(248, 284, -324, 432),
        ring(388, 261, -324, 345),
        ring(528, 189, -311, 194),
        ring(668, 140, -289, 100),
        ring(808, 72, -88, 43),
    ],
};

/// The cow's: a deep, broad brow narrowing a little to a wide muzzle.
pub const COW_HEAD: MeasuredHead = MeasuredHead {
    source: "animal03/jaldmic_vaca_cow_head.target",
    length: 331,
    pitch: 11,
    rings: [
        ring(-125, 2, 35, 43),
        ring(15, 320, -229, 310),
        ring(155, 258, -229, 300),
        ring(295, 234, -229, 259),
        ring(435, 214, -229, 190),
        ring(575, 155, -229, 140),
        ring(715, 130, -196, 91),
        ring(855, 57, -129, 32),
    ],
};

/// The dog's, kept for beast peoples' heads: rounder than the wolf's.
pub const DOG_HEAD: MeasuredHead = MeasuredHead {
    source: "animal03/jaldmic_perro_dog_head.target",
    length: 332,
    pitch: 1,
    rings: [
        ring(-200, 99, -236, -232),
        ring(-60, 176, -236, 67),
        ring(80, 249, -236, 255),
        ring(220, 237, -230, 286),
        ring(360, 209, -220, 232),
        ring(500, 154, -236, 139),
        ring(640, 112, -221, 81),
        ring(780, 42, -60, 36),
    ],
};

/// The horse's, from its whole body: long and narrow, its first rings still in the
/// neck, as a horse's head grows out of it with no back of the skull to find.
pub const HORSE_HEAD: MeasuredHead = MeasuredHead {
    source: "animal04/jaldmic_horse.target",
    length: 380,
    pitch: 20,
    rings: [
        ring(-322, 3, -169, -168),
        ring(-182, 143, -169, -13),
        ring(-42, 138, -169, 202),
        ring(98, 206, -169, 352),
        ring(238, 189, -133, 331),
        ring(378, 128, -105, 247),
        ring(518, 110, -129, 143),
        ring(658, 63, -139, 32),
    ],
};

/// A person's head with a reptile's muzzle, for the kobold and lizardfolk: the skull a
/// person's, the jaws long and deep.
pub const REPTILE_HEAD: MeasuredHead = MeasuredHead {
    source: "animal02/jaldmic_furry_mouth_reptile.target",
    length: 274,
    pitch: 6,
    rings: [
        ring(-225, 48, 204, 259),
        ring(-85, 230, -143, 504),
        ring(55, 275, -260, 560),
        ring(195, 268, -260, 560),
        ring(335, 241, -260, 500),
        ring(475, 193, -252, 212),
        ring(615, 171, -233, 115),
        ring(755, 114, -75, 62),
    ],
};

/// The person's own head, the base mesh's: what the heads on a person are fitted to a
/// frame's oval head by (`MeasuredHead::on_frame`).
pub const PERSON_HEAD: MeasuredHead = MeasuredHead {
    source: "base.obj",
    length: 210,
    pitch: 2,
    rings: [
        ring(-275, 25, 282, 311),
        ring(-135, 272, -138, 624),
        ring(5, 340, -368, 700),
        ring(145, 402, -368, 726),
        ring(285, 343, -368, 717),
        ring(425, 309, -366, 661),
        ring(565, 223, -364, 522),
        ring(705, 41, -29, 50),
    ],
};

/// Every measured head, by the name `cargo xtask shapes` prints.
pub const MEASURED_HEADS: [(&str, MeasuredHead); 6] = [
    ("Wolf", WOLF_HEAD),
    ("Cow", COW_HEAD),
    ("Dog", DOG_HEAD),
    ("Horse", HORSE_HEAD),
    ("Reptile", REPTILE_HEAD),
    ("Person", PERSON_HEAD),
];
