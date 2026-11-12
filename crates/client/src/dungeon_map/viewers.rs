//! What each viewer sees, worked out again when anyone looks anew: the
//! player in their cone, from their cell's middle (as the server works it
//! out), and those sharing sight all round.

use shared::dungeon::{Level, Sight};
use shared::facing::{Cone, Field, Heading};
use shared::spot::Spot;

use super::mesh::View;
use super::sight_lines::add_cells;

/// Each viewer's view, the player's first, and all they see together.
pub(super) fn views(
    level: &Level,
    player: (i64, i64),
    facing: Option<(Heading, Field)>,
    sharing: impl Iterator<Item = (i64, i64)>,
) -> (Vec<View>, Sight) {
    let player_cone =
        facing.map(|(heading, field)| Cone::new(Spot::middle_of(player), heading, field));
    let views: Vec<View> = std::iter::once((player, player_cone))
        .chain(sharing.map(|cell| (cell, None)))
        .map(|((x, y), cone)| {
            let mut sight = Sight::new(level);
            match &cone {
                Some(cone) => sight.add_glimpses_in(level, x, y, None, cone),
                None => sight.add_glimpses(level, x, y, None),
            }
            View {
                cell: (x, y),
                sight,
            }
        })
        .collect();
    let mut sight = Sight::new(level);
    for view in &views {
        add_cells(level, &view.sight, &mut sight);
    }
    (views, sight)
}
