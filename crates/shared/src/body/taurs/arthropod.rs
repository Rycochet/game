//! An eight-legged lower body, as a spider's or a scorpion's: a level front body with
//! four pairs of legs spread along it, each rising out to its knee and falling to the
//! ground well out to the side. Its abdomen and any tail are `Parts`.

use super::super::layout::{
    Bone, Girdle, Head, Layout, Limb, MAX_BONES, MAX_LIMBS, Neck, Role, Spine, Station,
};
use super::super::rings::{RINGS, Ring, from_stations};
use super::super::{Parts, Side};

/// The measurements of an eight-legged lower body, in millimetres.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Arthropod {
    /// How high the middle of its front body is.
    pub height: i32,
    /// The front body's length, from where the abdomen joins.
    pub front: i32,
    /// The front body's half width and half depth.
    pub barrel: (i32, i32),
    /// How far out from its side a leg reaches before it falls to the ground.
    pub reach: i32,
    /// The legs' thickness at the body, as a radius.
    pub leg: i32,
}

fn share(value: i32, hundredths: i32) -> i32 {
    value * hundredths / 100
}

/// One leg: up and out to the knee, down and out, then down to the ground.
fn leg(arthropod: &Arthropod, side: Side, along: i32, forward: i32) -> Limb {
    let (reach, r) = (arthropod.reach, arthropod.leg);
    let rise = share(reach, 40);
    let up = rise * 800 / 1000;
    let fall = share(arthropod.height + up, 60);
    let last = arthropod.height + up - fall;
    let bone = |span: i32, way: [i32; 3], radius: i32, end_radius: i32| {
        Some(Bone {
            span,
            way,
            radius,
            end_radius,
        })
    };
    let mut bones = [None; MAX_BONES];
    bones[0] = bone(rise, [1000, 800, forward], r, share(r, 80));
    bones[1] = bone(fall, [700, -1000, forward / 2], share(r, 80), share(r, 60));
    bones[2] = bone(last, [150, -1000, 0], share(r, 60), share(r, 30));
    Limb {
        role: Role::JointedLeg,
        girdle: Girdle::Along(along),
        side,
        apart: share(arthropod.barrel.0, 70),
        bones,
        foot: 0,
    }
}

impl Arthropod {
    /// Its layout: a level front body over eight legs, the front pair reaching forwards
    /// and the back pair back. It has no neck or head of its own to speak of.
    pub fn layout(&self, parts: Parts) -> Layout {
        let (across, deep) = self.barrel;
        let mut limbs = [None; MAX_LIMBS];
        let pairs = [(15, -600), (35, -200), (55, 200), (75, 600)];
        let mut slot = 0;
        for (hundredths, forward) in pairs {
            for side in Side::BOTH {
                limbs[slot] = Some(leg(self, side, share(self.front, hundredths), forward));
                slot += 1;
            }
        }
        let station = |along: i32, hundredths: i32| Station {
            along,
            half: (share(across, hundredths), share(deep, hundredths)),
        };
        let mut layout = Layout {
            spine: Spine {
                base: [0, self.height, -self.front / 2],
                pitch: 90,
                hips: station(0, 45),
                waist: station(share(self.front, 20), 85),
                chest: station(share(self.front, 55), 100),
                shoulders: share(self.front, 80),
                neck_base: self.front,
                rings: [Ring::default(); RINGS],
            },
            neck: Neck {
                length: 0,
                radius: share(deep, 50),
                pitch: 0,
            },
            head: Head {
                length: 0,
                half: (0, 0),
            },
            limbs,
            parts,
            upper: None,
        };
        layout.spine.rings = from_stations(&layout);
        layout
    }
}
