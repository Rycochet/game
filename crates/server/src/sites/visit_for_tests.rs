//! Helpers on `Visit` for tests.

use shared::spot::Spot;

use super::Visit;

impl Visit {
    /// Puts the player somewhere else, for tests.
    pub(crate) fn move_for_test(&mut self, at: (i64, i64)) {
        self.position = Spot::middle_of(at);
    }

    /// Whether a cell of the level can be walked on, for tests.
    pub(crate) fn walkable_for_test(&self, (x, y): (i64, i64)) -> bool {
        self.level.level.is_walkable(x, y)
    }

    /// The level's first stairs, for tests.
    pub(crate) fn first_stairs(&self) -> (i64, i64) {
        let stairs = self.level.stairs[0];
        (stairs.x, stairs.y)
    }

    /// The stairs at a cell of the level, for tests.
    pub(crate) fn stairs_at(&self, (x, y): (i64, i64)) -> Option<shared::site::Stairs> {
        (self.level.stairs.iter())
            .find(|stairs| (stairs.x, stairs.y) == (x, y))
            .copied()
    }

    /// A walkable cell the player can't see, for tests.
    pub(crate) fn walkable_out_of_sight(&mut self) -> (i64, i64) {
        const SEARCH: i64 = 200;
        (-SEARCH..SEARCH)
            .flat_map(|y| (-SEARCH..SEARCH).map(move |x| (x, y)))
            .find(|&(x, y)| self.level.level.is_walkable(x, y) && !self.can_see(x, y))
            .expect("a site has more than one room")
    }

    /// A cell with room for this many steps east of it, for tests.
    pub(crate) fn room_to_walk_east(&self, steps: i64) -> (i64, i64) {
        const SEARCH: i64 = 200;
        (-SEARCH..SEARCH)
            .flat_map(|y| (-SEARCH..SEARCH).map(move |x| (x, y)))
            .find(|&(x, y)| (0..=steps).all(|dx| self.level.level.is_walkable(x + dx, y)))
            .expect("a site has a room")
    }
}
