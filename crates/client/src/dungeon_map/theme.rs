//! Which tile pictures a site is drawn with: walls and floors by the kind of
//! place (stone buildings, wooden huts, crypts, caves, ice), the rest the
//! same everywhere. All from the AngbandTk tiles (`assets/angbandtk/`).

use shared::feature::FeatureKind;

use super::mesh::Surface;

/// Pictures shared by every theme.
const DOOR: &str = "angbandtk/dg_extra132_12_05.png";
const GROUND: &str = "angbandtk/dg_grounds32_01_00.png";
const WATER: &str = "angbandtk/dg_grounds32_02_00.png";
const LAVA: &str = "angbandtk/dg_grounds32_04_00.png";

/// A site's wall and floor pictures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub wall: &'static str,
    pub floor: &'static str,
}

const STONE: Theme = Theme {
    wall: "angbandtk/dg_extra132_11_00.png",
    floor: "angbandtk/dg_grounds32_03_03.png",
};
const WOOD: Theme = Theme {
    wall: "angbandtk/dg_extra132_12_03.png",
    floor: "angbandtk/dg_extra132_12_04.png",
};
const CRYPT: Theme = Theme {
    wall: "angbandtk/dg_extra132_12_00.png",
    floor: "angbandtk/dg_features32_10_00.png",
};
const CAVE: Theme = Theme {
    wall: "angbandtk/dg_features32_00_06.png",
    floor: "angbandtk/dg_grounds32_04_06.png",
};
const ICE: Theme = Theme {
    wall: "angbandtk/dg_grounds32_11_06.png",
    floor: "angbandtk/dg_grounds32_09_03.png",
};

impl Theme {
    /// The theme for a kind of place (stone when there is none: the test
    /// room).
    pub fn of(kind: Option<FeatureKind>) -> Self {
        use FeatureKind::*;
        let Some(kind) = kind else {
            return STONE;
        };
        match kind {
            Castle | Fort | StoneTower | StoneCottage | Warehouse | ArcheryRange | AdobeHouse
            | AdobeStore | StonePyramid | SandPyramid => STONE,
            ThatchedHut | StiltHut | GreenStiltHut | RedTent | BlueTent | GreenTent
            | RedRoofCottage | GreyRoofCottage | SnowyCottage | WoodenTower => WOOD,
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
            Surface::Wall => self.wall,
            Surface::Door => DOOR,
            Surface::Ground => GROUND,
            Surface::Water => WATER,
            Surface::Lava => LAVA,
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
        let kinds = FeatureKind::ALL.into_iter().map(Some).chain([None]);
        for theme in kinds.map(Theme::of) {
            for surface in Surface::ALL {
                let path = format!("{assets}{}", theme.picture(surface));
                assert!(std::path::Path::new(&path).exists(), "missing {path}");
            }
        }
    }
}
