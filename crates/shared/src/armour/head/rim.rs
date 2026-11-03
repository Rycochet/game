//! Where a head piece's lower edge runs round the head: low at the front for a closed
//! face, to the jaw where cheeks or a bascinet's sides come down, to the nape behind.
//! Neck pieces hang from it (agreed with the Neck work), and the model is cut by it.

use super::{Face, HeadForm, HeadKind, Skull};
use crate::body::Frame;

/// How far the shell stands off the head: 8 mm of lining and about 2 of plate. Old
/// helmets fitted closely (Coppergate's brow band is 572 mm round).
pub const OFF_HEAD: i32 = 10;

/// The edge's height at the front, the sides and the back, in millimetres from the
/// ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub front: i32,
    pub side: i32,
    pub back: i32,
}

impl Edge {
    /// Its height this many degrees round from the front (0 to 180, either way),
    /// changing evenly between the front, sides and back.
    pub fn height(&self, from_front: i32) -> i32 {
        let blend = |a: i32, b: i32, t: i32| a + (b - a) * t / 60;
        match from_front.clamp(0, 180) {
            d if d <= 30 => self.front,
            d if d <= 90 => blend(self.front, self.side, d - 30),
            d if d <= 150 => blend(self.side, self.back, d - 90),
            _ => self.back,
        }
    }
}

/// Sixteen directions round, from the front towards the left, in thousandths: sine
/// (across) and cosine (forwards), so both sides work them out alike.
const ROUND: [(i32, i32); 16] = [
    (0, 1000),
    (383, 924),
    (707, 707),
    (924, 383),
    (1000, 0),
    (924, -383),
    (707, -707),
    (383, -924),
    (0, -1000),
    (-383, -924),
    (-707, -707),
    (-924, -383),
    (-1000, 0),
    (-924, 383),
    (-707, 707),
    (-383, 924),
];

impl HeadForm {
    /// The heights of its lower edge on this frame.
    pub fn edge(&self, frame: &Frame) -> Edge {
        let head = frame.height - frame.chin;
        let brow = frame.chin + head * 60 / 100;
        let ear = frame.chin + head * 35 / 100;
        let jaw = frame.chin + head / 20;
        let nape = frame.chin + head * 15 / 100;
        let closed = matches!(self.face, Face::Faceplate | Face::Slit);
        let level = |height| Edge {
            front: height,
            side: height,
            back: height,
        };
        match self.kind {
            HeadKind::Coif => return level(frame.shoulder + 20),
            HeadKind::Cap => {
                return Edge {
                    front: brow + 10,
                    side: ear,
                    back: nape,
                };
            }
            HeadKind::BevoredHelm => {
                return Edge {
                    front: frame.neck_base + 20,
                    side: frame.neck_base + 30,
                    back: frame.chin - 10,
                };
            }
            HeadKind::Helmet => {}
        }
        if self.brim.round() {
            return level(brow - 10);
        }
        let ogival = self.skull == Skull::Ogival;
        Edge {
            front: if closed { frame.chin - 15 } else { brow },
            side: if closed || matches!(self.face, Face::Visor(_)) {
                frame.chin - 10
            } else if self.cheeks || ogival || self.face == Face::Opening {
                jaw
            } else {
                ear
            },
            back: if closed {
                frame.chin - 10
            } else if ogival {
                jaw
            } else {
                nape
            },
        }
    }

    /// Sixteen points along the lower edge, from the front round to the left, in
    /// millimetres on this frame: where an aventail or camail hangs from.
    pub fn rim(&self, frame: &Frame) -> Vec<[i32; 3]> {
        let edge = self.edge(frame);
        let (across, deep) = (frame.head.0 + OFF_HEAD, frame.head.1 + OFF_HEAD);
        ROUND
            .iter()
            .enumerate()
            .map(|(at, &(sin, cos))| {
                let from_front = (at as i32 * 45 / 2).min(360 - at as i32 * 45 / 2);
                [
                    across * sin / 1000,
                    edge.height(from_front),
                    deep * cos / 1000,
                ]
            })
            .collect()
    }
}
