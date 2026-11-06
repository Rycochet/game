//! What one cell of a level is, and what that allows.

use serde::{Deserialize, Serialize};

/// One grid cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    /// Open: can be walked on, seen across and lit through.
    Floor,
    /// Solid: blocks movement, sight and light.
    Wall,
    /// A doorway. Doors are open for now (opening and closing them comes
    /// with mobs), so it behaves like floor.
    Door,
    /// A hidden door: a wall until someone finds it (finding comes later).
    /// Generators only put them where nothing depends on them.
    SecretDoor,
    /// Outside ground round a building or cave mouth: walkable, and the way
    /// back to the world map.
    Ground,
    /// Seen and lit across, but not walkable: it can only be jumped over
    /// (`movement`; swimming later).
    Water,
    /// As water, hotter.
    Lava,
    /// Blocks movement but not sight or light. Joins the walls either side
    /// of it, like a door.
    Window,
    /// Road surfaces in a town, poorest first: walkable outside ground, the
    /// land's own beyond them.
    DirtTrack,
    GravelTrack,
    WoodenBoards,
    Cobblestone,
    Flagstones,
    DressedStone,
    /// Walkable outside ground, planted: a town's garden.
    Garden,
    /// The door of a town's solid building: part of its wall, with nothing
    /// behind it to go into. Used from the cell it faces.
    BuildingDoor,
    /// A window in a town's solid building, for looks: part of its wall.
    BlindWindow,
    /// A gap in the floor, or open air beyond a ruined floor's edge: seen and
    /// lit across, and not out in the weather. Not walkable: as with liquid,
    /// one cell of it can only be jumped over (`movement`).
    Hole,
    /// Walkable outside ground, sown: a farm's field. What grows there is only
    /// looks, and the client's to choose.
    Field,
    /// Round a field: seen and lit across, and not walkable: as with liquid,
    /// it can only be jumped over (`movement`).
    Fence,
    /// The way through a fence: walkable.
    FieldGate,
    /// Walkable outside ground, tended: a strip of a herbalist's patch. What
    /// grows there is only looks, and the client's to choose.
    HerbBed,
    /// Walkable ground like `Ground`, but with something built over it: the
    /// weather and the light of outside don't reach it. Under a stilt hut, say.
    CoveredGround,
    /// The floor a World Gate stands on, in the Hall of Worlds
    /// (`shared::hall`): walkable floor, with the gate's own model on it.
    WorldGate,
    /// Walkable outside ground, grassed: round an oasis's pools.
    Grass,
    /// A tree on grass: not walkable, but seen and lit past.
    Tree,
    /// Water shallow enough to wade: walkable outside, drawn as water at half
    /// depth. Within `wilderness::SHALLOWS_REACH` of land in the wilderness.
    Shallows,
}

impl Cell {
    /// Whether it can be walked on.
    pub fn is_walkable(self) -> bool {
        matches!(
            self,
            Self::Floor
                | Self::Door
                | Self::Ground
                | Self::Garden
                | Self::Field
                | Self::FieldGate
                | Self::HerbBed
                | Self::CoveredGround
                | Self::WorldGate
                | Self::Grass
                | Self::Shallows
        ) || self.is_road()
    }

    /// Whether it is a road's surface.
    pub fn is_road(self) -> bool {
        matches!(
            self,
            Self::DirtTrack
                | Self::GravelTrack
                | Self::WoodenBoards
                | Self::Cobblestone
                | Self::Flagstones
                | Self::DressedStone
        )
    }

    /// Whether it is out in the open: weather falls on it.
    pub fn is_outside(self) -> bool {
        matches!(
            self,
            Self::Ground
                | Self::Garden
                | Self::Field
                | Self::Fence
                | Self::FieldGate
                | Self::HerbBed
                | Self::Grass
                | Self::Tree
                | Self::Shallows
        ) || self.is_road()
    }

    /// Whether it is a liquid: not walkable, but one cell of it can be jumped.
    pub fn is_liquid(self) -> bool {
        matches!(self, Self::Water | Self::Lava)
    }

    /// Whether one cell of it can be jumped over: liquid, a hole or a fence.
    pub fn is_jumpable(self) -> bool {
        self.is_liquid() || matches!(self, Self::Hole | Self::Fence)
    }

    /// Whether sight and light pass through it.
    pub fn is_open(self) -> bool {
        self.is_walkable()
            || matches!(
                self,
                Self::Water | Self::Lava | Self::Window | Self::Hole | Self::Fence | Self::Tree
            )
    }

    /// Whether it is a wall (a hidden door looks and acts like one, and so do
    /// the doors and windows of a town's solid buildings).
    pub fn is_wall(self) -> bool {
        matches!(
            self,
            Self::Wall | Self::SecretDoor | Self::BuildingDoor | Self::BlindWindow
        )
    }

    /// Whether it is a hole in a wall, which joins the walls either side.
    pub fn is_opening(self) -> bool {
        matches!(self, Self::Door | Self::Window)
    }
}
