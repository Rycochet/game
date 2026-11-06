//! What covers a character's hair: the lowest edge of everything on its head, an
//! aventail hiding it all, a bevor over the chin.

use shared::armour::head::HeadForm;
use shared::armour::neck::{NeckForm, NeckKind};
use shared::body::Frame;

use crate::item_art::Look;
use crate::item_art::body::Cover;

/// What these worn things cover of the hair, in the drawn body's millimetres: what is
/// worn is built on this frame and moved up by `lift` (a taur's upper body).
pub(super) fn cover_of(worn: &[Look], frame: &Frame, lift: f32) -> Cover {
    let mut heads: Vec<&HeadForm> = Vec::new();
    let mut necks: Vec<&NeckForm> = Vec::new();
    for look in worn {
        match look {
            Look::Head { head, .. } => heads.push(&head.form),
            Look::Neck { neck, .. } => necks.push(&neck.form),
            Look::Set { set, .. } => {
                heads.extend(&set.form.head);
                necks.extend(&set.form.neck);
            }
            _ => {}
        }
    }
    // The lowest edge at each angle of whatever is on the head.
    let rim = heads.iter().fold(None, |lowest: Option<[f32; 16]>, form| {
        let edge = form.rim(frame);
        Some(std::array::from_fn(|at| {
            let y = edge[at][1] as f32 + lift;
            lowest.map_or(y, |lowest| lowest[at].min(y))
        }))
    });
    let worn_kind = |kind: NeckKind| necks.iter().any(|form| form.kind == kind);
    Cover {
        rim,
        hidden: worn_kind(NeckKind::Aventail),
        chin_covered: worn_kind(NeckKind::Bevor),
    }
}
