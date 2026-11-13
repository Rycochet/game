//! `cargo xtask test`'s check that the figures copied into the shared body
//! (`shared::body::MEASURED`) are still what the source files measure, as the
//! icons check does for the icons: if a source file or the tool changes, the
//! figures are copied again.

use std::path::Path;
use std::process::ExitCode;

use shared::body::{
    HEAD_RINGS, MEASURED, MEASURED_HEADS, Measured, MeasuredHead, MeasuredRing, RINGS,
};

use super::fourfoot::Fourfoot;
use super::head::Head;
use super::{FOURFEET, HEADS, measure, measure_head};

/// Fails if any committed figure differs from what is measured now.
pub fn check(root: &Path) -> ExitCode {
    println!("==> Creature shapes");
    let mut wrong = Vec::new();
    for (name, target) in FOURFEET {
        let Some((_, committed)) = MEASURED.iter().find(|(named, _)| *named == name) else {
            wrong.push(format!("{name}: not in shared::body::MEASURED"));
            continue;
        };
        match measure(root, target, RINGS) {
            Ok(beast) => {
                let measured = figures(target, &beast);
                if measured != *committed {
                    wrong.push(format!(
                        "{name}: shared::body has {committed:?}, the tool measures {measured:?}"
                    ));
                }
            }
            Err(error) => wrong.push(error),
        }
    }
    for (name, target) in HEADS {
        let Some((_, committed)) = MEASURED_HEADS.iter().find(|(named, _)| *named == name) else {
            wrong.push(format!(
                "{name}'s head: not in shared::body::MEASURED_HEADS"
            ));
            continue;
        };
        match measure_head(root, target) {
            Ok(head) => {
                let measured = head_figures(target, &head);
                if measured != *committed {
                    wrong.push(format!(
                        "{name}'s head: shared::body has {committed:?}, the tool measures {measured:?}"
                    ));
                }
            }
            Err(error) => wrong.push(error),
        }
    }
    if wrong.is_empty() {
        println!("==> The measured figures match their source files");
        return ExitCode::SUCCESS;
    }
    for line in &wrong {
        eprintln!("{line}");
    }
    eprintln!(
        "==> FAILED: copy `cargo xtask shapes` into crates/shared/src/body/measured.rs (heads: measured_heads.rs)"
    );
    ExitCode::FAILURE
}

/// A beast's figures as the shared body keeps them, in whole thousandths of
/// the withers' height, as the tables print them.
fn figures(source: &'static str, beast: &Fourfoot) -> Measured {
    let share = |value: f64| (value / beast.withers * 1000.0).round() as i32;
    let mut rings = [MeasuredRing {
        along: 0,
        half: 0,
        belly: 0,
        back: 0,
    }; RINGS];
    for (slot, (along, ring)) in rings.iter_mut().zip(&beast.trunk) {
        *slot = MeasuredRing {
            along: share(*along),
            half: share(ring.half_width),
            belly: share(ring.belly),
            back: share(ring.back),
        };
    }
    Measured {
        source,
        hips: share(beast.hips),
        shoulders: share(beast.shoulders),
        rings,
    }
}

/// A head's figures as the shared body keeps them, in whole thousandths of its
/// length, as the table prints them.
fn head_figures(source: &'static str, head: &Head) -> MeasuredHead {
    let share = |value: f64| (value / head.length * 1000.0).round() as i32;
    let mut rings = [MeasuredRing {
        along: 0,
        half: 0,
        belly: 0,
        back: 0,
    }; HEAD_RINGS];
    for (slot, (along, ring)) in rings.iter_mut().zip(&head.rings) {
        *slot = MeasuredRing {
            along: share(*along),
            half: share(ring.half_width),
            belly: share(ring.belly),
            back: share(ring.back),
        };
    }
    MeasuredHead {
        source,
        pitch: head.pitch.round() as i32,
        rings,
    }
}
