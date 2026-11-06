//! A beast's layout worked out from its measurements: a level back over four legs, or
//! a bird's leaning back over two. Legs are bones hanging down to the ground, each a
//! share of the height they hang from, bent back and forth as the animal's are.

use super::super::layout::{
    Bone, Girdle, Head, Layout, Limb, MAX_BONES, MAX_LIMBS, Neck, Role, Spine, Station,
};
use super::super::{Feet, Parts, Side};
use super::{Bird, Fourfoot};

/// `value` in hundredths.
fn share(value: i32, hundredths: i32) -> i32 {
    value * hundredths / 100
}

/// A leg hanging `height` down to the ground: each bone a share of the height (the last
/// takes what is left) and leaning forwards (+) or back (-) so many millimetres per
/// metre, its radius at the top and the end of each bone.
fn leg(
    role: Role,
    girdle: Girdle,
    side: Side,
    apart: i32,
    height: i32,
    bones: &[(i32, i32, (i32, i32))],
    foot: i32,
) -> Limb {
    let mut slots = [None; MAX_BONES];
    let mut left = height;
    for (at, &(hundredths, forward, (radius, end_radius))) in bones.iter().enumerate() {
        let last = at + 1 == bones.len();
        let span = if last {
            left
        } else {
            share(height, hundredths)
        };
        left -= span;
        slots[at] = Some(Bone {
            span,
            way: [0, -1000, forward],
            radius,
            end_radius,
        });
    }
    Limb {
        role,
        girdle,
        side,
        apart,
        bones: slots,
        foot,
    }
}

/// Limbs in slots, in order.
fn slots(limbs: &[Limb]) -> [Option<Limb>; MAX_LIMBS] {
    let mut slots = [None; MAX_LIMBS];
    for (slot, limb) in slots.iter_mut().zip(limbs) {
        *slot = Some(*limb);
    }
    slots
}

/// A level back over four legs, its hips behind the middle and its shoulders in front.
pub(super) fn fourfoot(four: &Fourfoot, parts: Parts) -> Layout {
    let (across, deep) = four.barrel;
    let height = four.withers - deep;
    let r = four.leg;
    let thick = |hundredths: i32| share(r, hundredths);
    let hooves = parts.feet == Feet::Hooves;
    // Hind legs: thigh forwards, the hock bent back, then down to the hoof or the toes.
    let hind = if hooves {
        [(32, 350), (30, -450), (26, 0), (0, 150)]
    } else {
        [(35, 300), (33, -400), (22, -100), (0, 200)]
    };
    // Forelegs: the upper arm a little back, then straight down.
    let fore = if hooves {
        [(25, -150), (35, 0), (27, 0), (0, 150)]
    } else {
        [(25, -150), (38, 0), (25, 0), (0, 250)]
    };
    let radii = [
        (r, thick(55)),
        (thick(55), thick(40)),
        (thick(45), thick(50)),
        (thick(45), thick(40)),
    ];
    let bones = |shape: [(i32, i32); 4]| {
        let mut bones = [(0, 0, (0, 0)); 4];
        for (at, ((hundredths, forward), radius)) in shape.into_iter().zip(radii).enumerate() {
            bones[at] = (hundredths, forward, radius);
        }
        bones
    };
    let foot = if hooves { thick(80) } else { r * 2 };
    let mut limbs = Vec::new();
    for side in Side::BOTH {
        let hind_bones = bones(hind);
        let fore_bones = bones(fore);
        limbs.push(leg(
            Role::Hindleg,
            Girdle::Hips,
            side,
            share(across, 60),
            height,
            &hind_bones,
            foot,
        ));
        limbs.push(leg(
            Role::Foreleg,
            Girdle::Shoulders,
            side,
            share(across, 55),
            height,
            &fore_bones,
            foot,
        ));
    }
    let station = |along: i32, hundredths: i32| Station {
        along,
        half: (share(across, hundredths), share(deep, hundredths)),
    };
    Layout {
        spine: Spine {
            base: [0, height, -four.body / 2],
            pitch: 90,
            hips: station(0, 90),
            waist: station(share(four.body, 45), 95),
            chest: station(share(four.body, 80), 100),
            shoulders: four.body,
            neck_base: four.body + deep / 3,
        },
        neck: neck(four.neck),
        head: head(four.head),
        limbs: slots(&limbs),
        parts,
        upper: None,
    }
}

/// A bird's back leaning forwards from its hips, on two legs.
pub(super) fn bird(bird: &Bird, parts: Parts) -> Layout {
    let (across, deep) = bird.barrel;
    let (length, pitch) = bird.body;
    let r = bird.leg;
    let thick = |hundredths: i32| share(r, hundredths);
    let bones = [
        (30, 250, (r, thick(70))),
        (35, -250, (thick(60), thick(40))),
        (0, 0, (thick(35), thick(35))),
    ];
    let limbs: Vec<Limb> = Side::BOTH
        .map(|side| {
            leg(
                Role::Leg,
                Girdle::Hips,
                side,
                share(across, 50),
                bird.hips,
                &bones,
                r * 5,
            )
        })
        .to_vec();
    let station = |along: i32, hundredths: i32| Station {
        along,
        half: (share(across, hundredths), share(deep, hundredths)),
    };
    Layout {
        spine: Spine {
            base: [0, bird.hips, 0],
            pitch,
            hips: station(0, 80),
            waist: station(share(length, 40), 95),
            chest: station(share(length, 70), 100),
            shoulders: share(length, 85),
            neck_base: length,
        },
        neck: neck(bird.neck),
        head: head(bird.head),
        limbs: slots(&limbs),
        parts,
        upper: None,
    }
}

fn neck((length, pitch, radius): (i32, i32, i32)) -> Neck {
    Neck {
        length,
        radius,
        pitch,
    }
}

fn head((length, half): (i32, (i32, i32))) -> Head {
    Head { length, half }
}
