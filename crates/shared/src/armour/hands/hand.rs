//! `Hand`: a hand's measurements, from the body frame.

use crate::body::{Frame, Joint, Side};

/// One hand on a body frame, in millimetres, worked out from the frame's hand length
/// as artists proportion it: what hand pieces are built on. It hangs from the wrist
/// with the palm towards the thigh, the back of the hand outwards and the thumb forwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hand {
    pub side: Side,
    /// The middle of the wrist.
    pub wrist: [i32; 3],
    /// The tip of the longest finger.
    pub tip: [i32; 3],
    /// From the wrist to the tip.
    pub length: i32,
    /// From the wrist to the knuckles.
    pub palm: i32,
    /// Across the knuckles, front to back.
    pub breadth: i32,
    /// From palm to back.
    pub thickness: i32,
    /// The thumb's length, and how far below the wrist it starts.
    pub thumb: i32,
    pub thumb_from: i32,
    /// Fingers other than the thumb.
    pub fingers: u8,
    /// The forearm's radius at the wrist, and how long the forearm is.
    pub wrist_radius: i32,
    pub forearm_length: i32,
}

impl Hand {
    /// This side's hand on this frame, with this many digits, thumb included (half the
    /// fingers its owner has; at least a thumb and one finger).
    pub fn of(frame: &Frame, side: Side, digits: u8) -> Self {
        let length = frame.hand_length;
        Self {
            side,
            wrist: frame.joint(Joint::Wrist(side)),
            tip: frame.joint(Joint::Fingertips(side)),
            length,
            palm: length * 55 / 100,
            breadth: length * 45 / 100,
            thickness: length * 16 / 100,
            thumb: length * 33 / 100,
            thumb_from: length * 15 / 100,
            fingers: digits.max(2) - 1,
            // As the bare body draws its wrist.
            wrist_radius: frame.forearm * 7 / 10,
            forearm_length: frame.wrist_length,
        }
    }

    /// The digits on each hand of a body with this many fingers in all.
    pub fn digits(fingers: Option<u8>) -> u8 {
        fingers.map_or(5, |fingers| fingers / 2)
    }
}
