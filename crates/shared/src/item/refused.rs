//! `MoveRefused`: why a move or an addition was refused.

/// Why something can't go where it was asked to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveRefused {
    /// The item isn't there to move.
    NoSuchItem,
    /// The grid isn't there, or what was named has none.
    NoSuchGrid,
    /// It would be inside itself.
    IntoItself,
    /// It is too big for the container's outside.
    TooBig,
    /// It would go too many grids deep.
    TooDeep,
    /// It would stick out of the grid, or lie on something else.
    NoRoom,
    /// It is too heavy for a container it would end up in.
    TooHeavy,
    /// It is worn, under something on a higher layer.
    Covered,
}
