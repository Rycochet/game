//! Which tile pictures a site is drawn with: walls and floors by the kind of
//! place (stone buildings, wooden huts, crypts, caves, ice), the rest the
//! same everywhere. All from the AngbandTk tiles (`assets/angbandtk/`).

use shared::poi::PoiKind;

use super::mesh::Surface;

/// Pictures shared by every theme.
const DOOR: &str = "angbandtk/dg_extra132_12_05.png";
/// All of a town's roads, gardens, walls, roofs and doors.
pub const TOWN_ATLAS: &str = "town_atlas.png";
const GROUND: &str = "angbandtk/dg_grounds32_01_00.png";
const WATER: &str = "angbandtk/dg_grounds32_02_00.png";
const LAVA: &str = "angbandtk/dg_grounds32_04_00.png";

/// How a place's walls, doors and windows are made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Straight walls, square doors, framed windows: buildings.
    Built,
    /// Walls curved like a letter C, arched doors, windows that are lines of
    /// columns: dungeons and caves.
    Natural,
}

/// A site's wall and floor pictures and how its walls are made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub wall: &'static str,
    pub floor: &'static str,
    pub shape: Shape,
}

const STONE: Theme = Theme {
    wall: "angbandtk/dg_extra132_11_00.png",
    floor: "angbandtk/dg_grounds32_03_03.png",
    shape: Shape::Built,
};
const WOOD: Theme = Theme {
    wall: "angbandtk/dg_extra132_12_03.png",
    floor: "angbandtk/dg_extra132_12_04.png",
    shape: Shape::Built,
};
const CRYPT: Theme = Theme {
    wall: "angbandtk/dg_extra132_12_00.png",
    floor: "angbandtk/dg_features32_10_00.png",
    shape: Shape::Natural,
};
const CAVE: Theme = Theme {
    wall: "angbandtk/dg_features32_00_06.png",
    floor: "angbandtk/dg_grounds32_04_06.png",
    shape: Shape::Natural,
};
const ICE: Theme = Theme {
    wall: "angbandtk/dg_grounds32_11_06.png",
    floor: "angbandtk/dg_grounds32_09_03.png",
    shape: Shape::Natural,
};

impl Theme {
    /// The theme for a kind of place (stone when there is none: the test
    /// room).
    pub fn of(kind: Option<PoiKind>) -> Self {
        use PoiKind::*;
        let Some(kind) = kind else {
            return STONE;
        };
        match kind {
            Castle | Fort | StoneTower | StoneCottage | Warehouse | ArcheryRange | AdobeHouse
            | AdobeStore | StonePyramid | SandPyramid | Hamlet | Village | Town | City => STONE,
            ThatchedHut | StiltHut | GreenStiltHut | RedTent | BlueTent | GreenTent
            | RedRoofCottage | GreyRoofCottage | SnowyCottage | WoodenTower | Farm
            | HerbalistHut => WOOD,
            Crypt | MossyCrypt | BloodyCrypt | Mausoleum | Grave | Ruins | Barrel | Barrels => {
                CRYPT
            }
            GrassyCave | EarthCave | RockCave | Boulders => CAVE,
            IceCave | IceShelter => ICE,
        }
    }

    /// The picture for a surface.
    pub fn picture(self, surface: Surface) -> &'static str {
        match surface {
            Surface::Floor => self.floor,
            Surface::Wall | Surface::WallBlend => self.wall,
            Surface::Door => DOOR,
            Surface::Ground => GROUND,
            Surface::Water => WATER,
            Surface::Lava => LAVA,
            Surface::Town => TOWN_ATLAS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every picture a theme names is in the assets folder.
    #[test]
    fn pictures_exist() {
        let assets = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/");
        let kinds = PoiKind::ALL.into_iter().map(Some).chain([None]);
        for theme in kinds.map(Theme::of) {
            for surface in Surface::ALL {
                let path = format!("{assets}{}", theme.picture(surface));
                assert!(std::path::Path::new(&path).exists(), "missing {path}");
            }
        }
    }
}
