//! The player's steps the server hasn't confirmed yet, and where they put the
//! player once it has said where it has them.

use std::collections::VecDeque;

use bevy::prelude::*;
use shared::movement::Step;
use shared::protocol::ClientMessage;

/// The steps sent and not yet confirmed, on the current map.
#[derive(Resource, Default)]
pub(super) struct Prediction {
    /// How many maps the server has put the player on (`ClientMessage::Step`).
    map: u32,
    /// The number of the last step sent.
    last: u32,
    /// How far each step sent on this map that the server hasn't confirmed
    /// moved the player (a jump moves two cells), by number.
    pending: VecDeque<(u32, (i64, i64))>,
}

impl Prediction {
    /// The message for a step just taken, which moved the player by `moved`,
    /// remembered until it is confirmed.
    pub fn step(&mut self, step: Step, moved: (i64, i64)) -> ClientMessage {
        self.last += 1;
        self.pending.push_back((self.last, moved));
        ClientMessage::Step {
            map: self.map,
            number: self.last,
            step,
        }
    }

    /// The server has put the player on another map: steps on the last one
    /// will never be confirmed.
    pub fn changed_map(&mut self) {
        self.map += 1;
        self.pending.clear();
    }

    /// The server has the player at `at` on `map`, after step `number`:
    /// where the player should be now, with the steps taken since. `None` if
    /// it is about a map the player has left.
    pub fn confirmed(&mut self, map: u32, number: u32, at: (i64, i64)) -> Option<(i64, i64)> {
        if map != self.map {
            return None;
        }
        while self.pending.front().is_some_and(|&(n, _)| n <= number) {
            self.pending.pop_front();
        }
        Some(
            self.pending
                .iter()
                .fold(at, |at, (_, moved)| (at.0 + moved.0, at.1 + moved.1)),
        )
    }
}

/// Where to put the player, at `shown`, if the server's word puts them at
/// `predicted` instead: `None` if they are already there. On the world map
/// (`width`: it wraps east-west) the same place on another copy of the
/// planet counts as there; a move stays on the player's copy.
pub(super) fn correction(
    shown: (i64, i64),
    predicted: (i64, i64),
    width: Option<i64>,
) -> Option<(i64, i64)> {
    let (x, y) = predicted;
    let x = match width {
        Some(width) => x + (shown.0 - x + width / 2).div_euclid(width) * width,
        None => x,
    };
    ((x, y) != shown).then_some((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn east() -> Step {
        Step::new(1, 0).unwrap()
    }

    #[test]
    fn confirmed_steps_are_forgotten_and_later_ones_added() {
        let mut prediction = Prediction::default();
        for _ in 0..3 {
            prediction.step(east(), (1, 0));
        }
        // The server has dealt with the first two: the third is on top.
        assert_eq!(prediction.confirmed(0, 2, (10, 0)), Some((11, 0)));
        assert_eq!(prediction.confirmed(0, 3, (11, 0)), Some((11, 0)));
    }

    #[test]
    fn a_jump_counts_as_the_two_cells_it_moved() {
        let mut prediction = Prediction::default();
        prediction.step(east(), (1, 0));
        prediction.step(east(), (2, 0));
        assert_eq!(prediction.confirmed(0, 1, (1, 0)), Some((3, 0)));
    }

    #[test]
    fn a_refused_step_moves_the_player_back() {
        let mut prediction = Prediction::default();
        prediction.step(east(), (1, 0));
        prediction.step(east(), (1, 0));
        // The first was refused: the server still has the player at 0.
        assert_eq!(prediction.confirmed(0, 1, (0, 0)), Some((1, 0)));
        assert_eq!(correction((2, 0), (1, 0), None), Some((1, 0)));
    }

    #[test]
    fn positions_from_a_left_map_are_ignored() {
        let mut prediction = Prediction::default();
        prediction.step(east(), (1, 0));
        prediction.changed_map();
        assert_eq!(prediction.confirmed(0, 1, (5, 5)), None);
        assert_eq!(prediction.confirmed(1, 1, (5, 5)), Some((5, 5)));
        let ClientMessage::Step { map, number, .. } = prediction.step(east(), (1, 0)) else {
            unreachable!()
        };
        assert_eq!((map, number), (1, 2));
    }

    #[test]
    fn the_world_map_wraps() {
        // The same place a planet round: nothing to correct.
        assert_eq!(correction((1005, 3), (5, 3), Some(1000)), None);
        // Elsewhere: corrected on the player's copy of the planet.
        assert_eq!(correction((1005, 3), (4, 3), Some(1000)), Some((1004, 3)));
        assert_eq!(correction((5, 3), (995, 3), Some(1000)), Some((-5, 3)));
    }
}
