//! What a pair of footwear is called: a name from the research when its signature
//! fits one, otherwise a plain one from its kind, how it is made and how high it is.

use super::{Build, Decoration, Fastening, FootwearForm, FootwearPiece, Heel, Height, Toe, Top};
use crate::material::{Category, Material};

impl FootwearForm {
    /// A name from history, when it fits one.
    pub fn specific_name(&self) -> Option<&'static str> {
        match self.piece {
            FootwearPiece::Shoes => self.shoe_name(),
            FootwearPiece::Boots => self.boot_name(),
            FootwearPiece::Sandals => self.sandal_name(),
            FootwearPiece::Clogs => self.clog_name(),
            FootwearPiece::Slippers => self.slipper_name(),
        }
    }

    fn shoe_name(&self) -> Option<&'static str> {
        let (toe, fastening) = (self.toe, self.fastening);
        Some(match self.build {
            Build::Rawhide if toe == Toe::Upturned => "Opanaks",
            Build::Rawhide if fastening == Fastening::Loops => "Carbatinas",
            Build::Rawhide => "Pampooties",
            Build::Woven if self.upper == Material::Cotton => "Espadrilles",
            Build::Woven => "Bast Shoes",
            Build::Soft if fastening == Fastening::Toggles => "Turnshoes",
            _ if toe == Toe::Upturned => match self.decoration {
                Decoration::Pompom => "Tsarouchia",
                Decoration::Embroidery => "Juttis",
                _ => return None,
            },
            Build::Soft if toe == Toe::Round => "Moccasins",
            Build::Sewn => return self.sewn_shoe_name(),
            _ => return None,
        })
    }

    fn sewn_shoe_name(&self) -> Option<&'static str> {
        let (toe, fastening, heel) = (self.toe, self.fastening, self.heel);
        let cloth = self.upper.category() == Category::Textile;
        Some(match fastening {
            _ if toe == Toe::LongPoint => "Poulaines",
            _ if toe == Toe::Broad => "Duckbill Shoes",
            _ if cloth && heel == Heel::Flat => "Cloth Shoes",
            Fastening::Loops => "Ghillies",
            _ if fastening.laced() && self.decoration == Decoration::Broguing => "Brogues",
            _ if fastening.laced() && self.contrast.is_some() => "Saddle Shoes",
            Fastening::OpenLaces if toe == Toe::Square => "Latchet Shoes",
            Fastening::ClosedLaces => "Oxfords",
            Fastening::OpenLaces => "Derbies",
            Fastening::Buckle if self.decoration == Decoration::Jewels || heel == Heel::Louis => {
                "Buckled Shoes"
            }
            Fastening::Buckle if toe == Toe::Round => "Mary Janes",
            Fastening::Buckle => "Monk Shoes",
            Fastening::PullOn if self.decoration == Decoration::Tassels || heel == Heel::Flat => {
                "Loafers"
            }
            Fastening::PullOn => "Court Shoes",
            _ => return None,
        })
    }

    fn boot_name(&self) -> Option<&'static str> {
        let (fastening, heel, top) = (self.fastening, self.heel, self.top);
        Some(match self.build {
            _ if self.height == Height::Thigh => "Cavalier Boots",
            Build::Felted => "Valenki",
            Build::Soft if self.upper == Material::Fur || top == Top::Fur => "Mukluks",
            Build::Soft if self.toe == Toe::Upturned => "Hwa",
            Build::Soft if fastening == Fastening::Toggles => "Turnshoes",
            Build::Sewn if self.toe == Toe::Upturned => "Gutals",
            Build::Sewn if self.toe == Toe::LongPoint => return None,
            Build::Sewn => match (self.height, fastening) {
                (Height::Ankle, Fastening::ElasticSides) => "Chelsea Boots",
                (Height::Ankle, Fastening::Buttons) => "Button Boots",
                (Height::Ankle, Fastening::Buckle) => "Jodhpur Boots",
                (Height::Ankle, _) if fastening.laced() => match () {
                    _ if self.upper == Material::Suede => "Chukka Boots",
                    _ if self.under != super::Under::Nothing => "Hobnail Boots",
                    _ if heel == Heel::Flat => "Brogans",
                    _ => return None,
                },
                (Height::Calf, Fastening::Thongs) => "Calcei",
                (Height::Calf | Height::Knee, Fastening::OpenLaces) => "Buskins",
                (Height::Calf, Fastening::Buckle) => "Engineer Boots",
                (Height::Calf, Fastening::PullOn)
                    if heel == Heel::Riding
                        && (top == Top::Scalloped || self.decoration == Decoration::Stitching) =>
                {
                    "Cowboy Boots"
                }
                (Height::Knee, _) => match top {
                    Top::Bucket => "Jackboots",
                    Top::Notch => "Hessian Boots",
                    Top::Cuff => "Top Boots",
                    _ if heel == Heel::Louis => "Russian Boots",
                    Top::Plain if fastening == Fastening::PullOn && heel != Heel::Riding => {
                        "Riding Boots"
                    }
                    _ => return None,
                },
                _ => return None,
            },
            _ => return None,
        })
    }

    fn sandal_name(&self) -> Option<&'static str> {
        let fastening = self.fastening;
        Some(match self.build {
            Build::Strapped if self.height > Height::Low => "Caligae",
            Build::Strapped if fastening == Fastening::Loops => "Crepidae",
            Build::Strapped if self.decoration == Decoration::Plaited => match fastening {
                Fastening::ToeThong => "Kolhapuris",
                _ => "Huaraches",
            },
            Build::Woven if fastening == Fastening::Thongs => "Waraji",
            Build::Woven if self.upper == Material::Straw => "Zori",
            Build::Woven => "Reed Sandals",
            _ => return None,
        })
    }

    fn clog_name(&self) -> Option<&'static str> {
        let (fastening, heel) = (self.fastening, self.heel);
        Some(match fastening {
            Fastening::ToeThong if self.upper == Material::Wood => "Padukas",
            Fastening::ToeThong if heel == Heel::Teeth => "Geta",
            Fastening::Strap if heel == Heel::Teeth => "Nalins",
            Fastening::Strap if self.decoration == Decoration::Carved => "Bakya",
            _ if self.open() => return None,
            _ if self.upper == Material::Wood => "Sabots",
            _ => return None,
        })
    }

    fn slipper_name(&self) -> Option<&'static str> {
        Some(match self.toe {
            Toe::Pointed if self.backless && self.heel == Heel::Flat => "Babouches",
            _ if self.backless => "Mules",
            Toe::Round if self.build == Build::Soft && self.upper == Material::Leather => "Socci",
            _ => return None,
        })
    }

    /// The plain name for its kind, how it is made and how high it is.
    pub fn generic_name(&self) -> &'static str {
        use Height as H;
        match (self.piece, self.build, self.height) {
            (FootwearPiece::Shoes, Build::Soft, _) => "Soft Shoes",
            (FootwearPiece::Shoes, Build::Rawhide, _) => "Rawhide Shoes",
            (FootwearPiece::Shoes, Build::Woven, _) => "Woven Shoes",
            (FootwearPiece::Shoes, _, _) => "Shoes",
            (FootwearPiece::Boots, Build::Felted, _) => "Felt Boots",
            (FootwearPiece::Boots, Build::Wooden, _) => "Clog Boots",
            (FootwearPiece::Boots, Build::Soft, H::Ankle) => "Soft Ankle Boots",
            (FootwearPiece::Boots, Build::Soft, H::Calf) => "Soft Calf Boots",
            (FootwearPiece::Boots, Build::Soft, _) => "Soft Knee Boots",
            (FootwearPiece::Boots, _, H::Ankle) => "Ankle Boots",
            (FootwearPiece::Boots, _, H::Calf) => "Calf Boots",
            (FootwearPiece::Boots, _, H::Thigh) => "Thigh Boots",
            (FootwearPiece::Boots, _, _) => "Knee Boots",
            (FootwearPiece::Sandals, Build::Woven, _) => "Woven Sandals",
            (FootwearPiece::Sandals, _, H::Low) => "Sandals",
            (FootwearPiece::Sandals, _, _) => "Sandal Boots",
            (FootwearPiece::Clogs, _, _) if self.open() => "Wooden Sandals",
            (FootwearPiece::Clogs, _, _) => "Clogs",
            (FootwearPiece::Slippers, _, _) => "Slippers",
        }
    }

    /// Its name: the specific one, or the plain one with an unusual upper first.
    pub fn name(&self) -> String {
        if let Some(name) = self.specific_name() {
            return name.to_owned();
        }
        let generic = self.generic_name();
        match self.upper {
            Material::Silk
            | Material::Wool
            | Material::Linen
            | Material::Cotton
            | Material::Fur
            | Material::Felt
            | Material::Suede
            | Material::BuffLeather => format!("{} {generic}", self.upper.name()),
            _ => generic.to_owned(),
        }
    }
}
