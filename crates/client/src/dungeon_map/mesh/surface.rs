//! The kinds of surface, each with its own tile picture.

use shared::dungeon::Cell;

/// The kinds of surface, each drawn with its own tile picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Floor,
    Wall,
    Door,
    Ground,
    Water,
    Lava,
    /// The wall's picture fading out across the floor beside a natural wall.
    WallBlend,
    /// A town's roads, gardens, buildings and hedges, from one picture.
    Town,
}

impl Surface {
    pub const ALL: [Self; 8] = [
        Self::Floor,
        Self::Wall,
        Self::Door,
        Self::Ground,
        Self::Water,
        Self::Lava,
        Self::WallBlend,
        Self::Town,
    ];

    /// The surface the floor of a cell is drawn with.
    pub fn of_flat(cell: Cell) -> Self {
        match cell {
            Cell::Ground
            | Cell::CoveredGround
            | Cell::Field
            | Cell::Fence
            | Cell::FieldGate
            | Cell::HerbBed
            | Cell::Grass
            | Cell::Tree => Self::Ground,
            Cell::Water | Cell::Shallows => Self::Water,
            Cell::Lava => Self::Lava,
            _ => Self::Floor,
        }
    }
}
