//! Blending two bodies' limbs. Each side's limbs are paired: first with a limb of the
//! same role, then with what it becomes (an arm with a foreleg or a wing-arm, a leg
//! with a hind leg), then whatever is left in slot order. A limb with no partner grows
//! from, or shrinks to, a point where it hangs, and is left out at size nothing.

use super::super::Side;
use super::super::layout::{Bone, Girdle, Layout, Limb, MAX_BONES, MAX_LIMBS, Role, Spine};
use super::{HALF, WHOLE, mix};

/// Which of `b`'s slots each of `a`'s is paired with, and the other way round.
type Pairs = ([Option<usize>; MAX_LIMBS], [Option<usize>; MAX_LIMBS]);

pub(super) fn blend(a: &Layout, b: &Layout, t: i32) -> [Option<Limb>; MAX_LIMBS] {
    let (of_a, of_b) = pairs(a, b);
    let mut both: Vec<(Option<usize>, Option<usize>)> = Vec::with_capacity(MAX_LIMBS * 2);
    // In the slots of the body it is nearer, the other's unpaired limbs after them.
    if t < HALF {
        both.extend(filled(a).map(|i| (Some(i), of_a[i])));
        both.extend(
            filled(b)
                .filter(|&j| of_b[j].is_none())
                .map(|j| (None, Some(j))),
        );
    } else {
        both.extend(filled(b).map(|j| (of_b[j], Some(j))));
        both.extend(
            filled(a)
                .filter(|&i| of_a[i].is_none())
                .map(|i| (Some(i), None)),
        );
    }
    let mut limbs = [None; MAX_LIMBS];
    let blended = both.into_iter().filter_map(|(i, j)| {
        let la = i.and_then(|i| a.limbs[i]);
        let lb = j.and_then(|j| b.limbs[j]);
        mix_limb(la, lb, (&a.spine, &b.spine), t)
    });
    for (slot, limb) in limbs.iter_mut().zip(blended) {
        *slot = Some(limb);
    }
    limbs
}

/// The slots that hold a limb.
fn filled(layout: &Layout) -> impl Iterator<Item = usize> + '_ {
    (0..MAX_LIMBS).filter(|&i| layout.limbs[i].is_some())
}

/// Whether a limb of one role becomes one of the other when a body changes.
fn becomes(x: Role, y: Role) -> bool {
    use Role::*;
    let one_way = |x, y| {
        matches!(
            (x, y),
            (Arm | WingArm, Foreleg) | (Arm, WingArm) | (Leg, Hindleg)
        )
    };
    one_way(x, y) || one_way(y, x)
}

fn pairs(a: &Layout, b: &Layout) -> Pairs {
    let (mut of_a, mut of_b) = ([None; MAX_LIMBS], [None; MAX_LIMBS]);
    let role = |layout: &Layout, i: usize| layout.limbs[i].map(|limb| limb.role);
    let tests: [&dyn Fn(Role, Role) -> bool; 3] = [&|x, y| x == y, &becomes, &|_, _| true];
    for side in Side::BOTH {
        let on_side = |layout: &Layout, i: usize| layout.limbs[i].is_some_and(|l| l.side == side);
        for test in tests {
            for i in filled(a).filter(|&i| on_side(a, i)) {
                if of_a[i].is_some() {
                    continue;
                }
                let partner = filled(b).find(|&j| {
                    on_side(b, j)
                        && of_b[j].is_none()
                        && role(a, i).zip(role(b, j)).is_some_and(|(x, y)| test(x, y))
                });
                if let Some(j) = partner {
                    of_a[i] = Some(j);
                    of_b[j] = Some(i);
                }
            }
        }
    }
    (of_a, of_b)
}

/// One limb blended into its partner, or growing or shrinking alone; `None` once it
/// has no bone left with any length.
fn mix_limb(a: Option<Limb>, b: Option<Limb>, spines: (&Spine, &Spine), t: i32) -> Option<Limb> {
    let (a, b) = match (a, b) {
        (Some(a), Some(b)) => (a, b),
        (Some(a), None) => (a, point(&a)),
        (None, Some(b)) => (point(&b), b),
        (None, None) => return None,
    };
    let mut bones = [None; MAX_BONES];
    for (k, slot) in bones.iter_mut().enumerate() {
        *slot = match (a.bones[k], b.bones[k]) {
            (Some(x), Some(y)) => Some(mix_bone(&x, &y, t)),
            (Some(x), None) => Some(mix_bone(&x, &nothing(&x), t)),
            (None, Some(y)) => Some(mix_bone(&nothing(&y), &y, t)),
            (None, None) => None,
        }
        .filter(|bone| bone.span > 0);
    }
    // Nothing is left of a limb whose first bone has no length.
    bones[0]?;
    Some(Limb {
        role: if t < HALF { a.role } else { b.role },
        girdle: mix_girdle(a.girdle, b.girdle, spines, t),
        side: a.side,
        apart: mix(a.apart, b.apart, t),
        bones,
        foot: mix(a.foot, b.foot, t),
    })
}

/// A limb shrunk to where it hangs from.
fn point(limb: &Limb) -> Limb {
    Limb {
        bones: limb.bones.map(|bone| bone.map(|bone| nothing(&bone))),
        foot: 0,
        ..*limb
    }
}

/// A bone the same way, with no length or thickness.
fn nothing(bone: &Bone) -> Bone {
    Bone {
        span: 0,
        radius: 0,
        end_radius: 0,
        ..*bone
    }
}

fn mix_bone(a: &Bone, b: &Bone, t: i32) -> Bone {
    Bone {
        span: mix(a.span, b.span, t),
        way: mix_way(a.way, b.way, t),
        radius: mix(a.radius, b.radius, t),
        end_radius: mix(a.end_radius, b.end_radius, t),
    }
}

/// A way between two, as long as theirs blended, so a bone turning through a right
/// angle doesn't shorten on the way.
fn mix_way(a: [i32; 3], b: [i32; 3], t: i32) -> [i32; 3] {
    let way = [0, 1, 2].map(|i| mix(a[i], b[i], t));
    let length = length(way);
    if length == 0 {
        return way;
    }
    let wanted = mix(self::length(a), self::length(b), t);
    way.map(|x| x * wanted / length)
}

fn length(way: [i32; 3]) -> i32 {
    let squares: u64 = way.iter().map(|&x| (x as i64 * x as i64) as u64).sum();
    whole_root(squares) as i32
}

/// The square root, rounded down, in whole numbers so both sides agree.
pub(super) fn whole_root(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Where a limb hangs: the same girdle stays; otherwise it moves along the spine from
/// one place to the other, each end exactly its own.
fn mix_girdle(a: Girdle, b: Girdle, spines: (&Spine, &Spine), t: i32) -> Girdle {
    let along = |girdle: Girdle, spine: &Spine| match girdle {
        Girdle::Hips => 0,
        Girdle::Shoulders => spine.shoulders,
        Girdle::Along(along) => along,
    };
    match (a, b) {
        _ if a == b => a,
        _ if t == 0 => a,
        _ if t == WHOLE => b,
        _ => Girdle::Along(mix(along(a, spines.0), along(b, spines.1), t)),
    }
}
