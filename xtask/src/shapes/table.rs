//! A four-legged beast's measurements written out as tables, in thousandths
//! of its height at the withers: multiply by our own animal's height (740 mm
//! for a wolf) and divide by 1000 for millimetres.

use std::fmt::Write;

use super::fourfoot::Fourfoot;
use super::head::Head;
use super::leg::LegCut;
use super::upright::{LIMB_CUTS, Upright};

/// The tables for one beast, under its name and where it was measured from.
pub fn fourfoot(name: &str, source: &str, beast: &Fourfoot) -> String {
    // Whole thousandths of the withers' height.
    let share = |value: f64| (value / beast.withers * 1000.0).round() as i64;
    let mut text = String::new();
    // Writing into a `String` can't fail, so the results are left unused.
    let _ = writeln!(text, "{name} ({source})");
    let _ = writeln!(
        text,
        "  In thousandths of the height at the withers (the top of the back over the forefeet)."
    );
    let _ = writeln!(
        text,
        "  Hip joints {} above the ground; shoulder joints {} in front of them.",
        share(beast.hips),
        share(beast.shoulders)
    );
    let _ = writeln!(text);
    let _ = writeln!(
        text,
        "  Trunk, rump to the neck's base: along from the hip joints, belly and back from the ground"
    );
    let _ = writeln!(text, "  ring  along  half width  belly   back");
    for (at, (along, ring)) in beast.trunk.iter().enumerate() {
        let _ = writeln!(
            text,
            "  {:>4}  {:>5}  {:>10}  {:>5}  {:>5}",
            at + 1,
            share(*along),
            share(ring.half_width),
            share(ring.belly),
            share(ring.back)
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(
        text,
        "  Left legs clear of the trunk: half thickness (side to side, front to back), and the middle"
    );
    let _ = writeln!(
        text,
        "  (out from the body's middle line, along from the hip joints)"
    );
    let _ = writeln!(
        text,
        "  height  fore: side  front   out  along  hind: side  front   out  along"
    );
    for (height, fore, hind) in &beast.legs {
        let leg = |leg: &Option<LegCut>| match leg {
            Some(leg) => format!(
                "{:>10}  {:>5}  {:>4}  {:>5}",
                share(leg.half.0),
                share(leg.half.1),
                share(leg.middle.0),
                share(leg.middle.1)
            ),
            None => format!("{:>10}  {:>5}  {:>4}  {:>5}", "-", "-", "-", "-"),
        };
        let _ = writeln!(
            text,
            "  {:>6}  {}  {}",
            share(*height),
            leg(fore),
            leg(hind)
        );
    }
    text
}

/// The tables for a body standing upright, in thousandths of its height.
pub fn upright(name: &str, source: &str, body: &Upright) -> String {
    let share = |value: f64| (value / body.height * 1000.0).round() as i64;
    let mut text = String::new();
    let _ = writeln!(text, "{name} ({source})");
    let _ = writeln!(text, "  In thousandths of the height.");
    let _ = writeln!(
        text,
        "  Crotch {}, hip joints {}, shoulder joints {}, neck's base {} above the ground.",
        share(body.crotch),
        share(body.hips),
        share(body.shoulders),
        share(body.neck)
    );
    let _ = writeln!(text);
    let _ = writeln!(
        text,
        "  Trunk, crotch to the neck's base: height, front and back from the line up through the hip joints"
    );
    let _ = writeln!(text, "  ring  height  half width  front   back");
    for (at, (height, ring)) in body.trunk.iter().enumerate() {
        let _ = writeln!(
            text,
            "  {:>4}  {:>6}  {:>10}  {:>5}  {:>5}",
            at + 1,
            share(*height),
            share(ring.half_width),
            share(ring.belly),
            share(ring.back)
        );
    }
    let _ = writeln!(text);
    let _ = writeln!(
        text,
        "  Left limbs: length, then half thickness (across / front to back) at {} cuts along each bone",
        LIMB_CUTS
    );
    for bone in &body.limbs {
        let cuts: Vec<String> = bone
            .cuts
            .iter()
            .map(|cut| match cut {
                Some((a, b)) => format!("{:>3}/{:<3}", share(*a), share(*b)),
                None => format!("{:>3}/{:<3}", "-", "-"),
            })
            .collect();
        let _ = writeln!(
            text,
            "  {:<9}  {:>4}  {}",
            bone.name,
            share(bone.length),
            cuts.join("  ")
        );
    }
    text
}

/// The table for one head, in thousandths of its length from the back of the
/// skull to the nose.
pub fn head(name: &str, source: &str, head: &Head) -> String {
    let share = |value: f64| (value / head.length * 1000.0).round() as i64;
    let mut text = String::new();
    let _ = writeln!(text, "{name}'s head ({source})");
    let _ = writeln!(
        text,
        "  In thousandths of its length from the back of the skull to the nose ({} mm in the mesh); nose down {} degrees.",
        (head.length * 100.0).round() as i64,
        head.pitch.round() as i64
    );
    let _ = writeln!(
        text,
        "  Rings: along from the `head` joint, belly and back up from the head's line"
    );
    let _ = writeln!(text, "  ring  along  half width  belly   back");
    for (at, (along, ring)) in head.rings.iter().enumerate() {
        let _ = writeln!(
            text,
            "  {:>4}  {:>5}  {:>10}  {:>5}  {:>5}",
            at + 1,
            share(*along),
            share(ring.half_width),
            share(ring.belly),
            share(ring.back)
        );
    }
    text
}
