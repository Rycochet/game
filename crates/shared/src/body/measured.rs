//! Beasts' trunks as measured from MakeHuman's meshes by `cargo xtask shapes`, which
//! `cargo xtask test` checks these figures still match: JALdMIC's cow, dog and horse
//! (CC-BY; see `docs/credits.md`). Every figure is in thousandths of the height at the
//! withers, as the tool prints it, so only the proportions come from the meshes: each
//! beast is scaled to our own size (`Measured::rings`).

use super::rings::{RINGS, Ring};

/// One ring as measured: how far along from the hip joints, its half width, and the
/// belly's and the back's heights from the ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeasuredRing {
    pub along: i32,
    pub half: i32,
    pub belly: i32,
    pub back: i32,
}

/// A four-legged beast's trunk as measured: how high its hip joints are, how far in
/// front of them its shoulder joints are, and its rings from the rump to the neck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Measured {
    /// The mesh it was measured from, under `spare_assets/makehuman/`.
    pub source: &'static str,
    pub hips: i32,
    pub shoulders: i32,
    pub rings: [MeasuredRing; RINGS],
}

impl Measured {
    /// Its rings on a beast of ours this tall at the withers, its shoulder joints
    /// `body` in front of its hip joints and its spine `spine` above the ground: along
    /// the spine by its body's length, everything else by its height.
    pub fn rings(&self, withers: i32, body: i32, spine: i32) -> [Ring; RINGS] {
        let tall = |share: i32| share * withers / 1000;
        self.rings.map(|ring| Ring {
            along: ring.along * body / self.shoulders,
            half: tall(ring.half),
            belly: spine - tall(ring.belly),
            back: tall(ring.back) - spine,
        })
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

/// The cow: boxy and deep, its belly barely rising behind the ribs.
pub const COW: Measured = Measured {
    source: "animal04/jaldmic_vaca_cow.target",
    hips: 842,
    shoulders: 1003,
    rings: [
        ring(-152, 1, 856, 861),
        ring(-12, 199, 568, 1071),
        ring(128, 215, 502, 1053),
        ring(268, 221, 476, 1030),
        ring(408, 223, 398, 1001),
        ring(547, 220, 388, 996),
        ring(687, 198, 392, 992),
        ring(827, 174, 388, 1006),
        ring(967, 190, 347, 998),
        ring(1107, 67, 477, 989),
    ],
};

/// The dog, which the wolf is drawn from: a deep chest and a tucked belly.
pub const DOG: Measured = Measured {
    source: "animal04/jaldmic_perro_dog.target",
    hips: 908,
    shoulders: 991,
    rings: [
        ring(-93, 2, 910, 922),
        ring(37, 190, 776, 1060),
        ring(167, 158, 767, 1066),
        ring(298, 120, 759, 1067),
        ring(428, 126, 677, 1036),
        ring(558, 148, 576, 1011),
        ring(688, 151, 537, 996),
        ring(818, 171, 516, 1000),
        ring(949, 173, 562, 1114),
        ring(1079, 99, 838, 1238),
    ],
};

/// The horse: a barrel, its belly rising a little to the flank, the last two rings
/// already climbing into the neck.
pub const HORSE: Measured = Measured {
    source: "animal04/jaldmic_horse.target",
    hips: 800,
    shoulders: 812,
    rings: [
        ring(-127, 0, 845, 845),
        ring(-11, 179, 593, 974),
        ring(104, 193, 567, 983),
        ring(219, 181, 596, 977),
        ring(334, 170, 548, 939),
        ring(449, 155, 514, 930),
        ring(565, 157, 509, 956),
        ring(680, 162, 500, 974),
        ring(795, 151, 492, 1064),
        ring(910, 69, 796, 1121),
    ],
};

/// Every measured beast, by the name `cargo xtask shapes` prints.
pub const MEASURED: [(&str, Measured); 3] = [("Cow", COW), ("Dog", DOG), ("Horse", HORSE)];
