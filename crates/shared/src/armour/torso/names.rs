//! What a piece of torso armour is called: a specific name when it fits one, otherwise
//! its construction's generic name.

use super::{Breast, Fluting, Quilting, Rings, TorsoBody, TorsoForm, Weave};
use crate::armour::Style;
use crate::material::Material;

impl TorsoForm {
    /// The generic name for its construction.
    pub fn generic_name(&self) -> &'static str {
        match self.body {
            TorsoBody::Plate { .. } if self.back => "Cuirass",
            TorsoBody::Plate { .. } => "Breastplate",
            TorsoBody::Leather { .. } => "Leather Cuirass",
            TorsoBody::Mail { .. } => "Mail Shirt",
            TorsoBody::RingMail { .. } => "Ring Mail",
            TorsoBody::Padded { .. } if self.vest => "Arming Vest",
            TorsoBody::Padded { .. } => "Padded Jack",
        }
    }

    /// A name from history or stories, when it fits one.
    pub fn specific_name(&self) -> Option<&'static str> {
        let back = self.back;
        let pick = |cuirass, breastplate| Some(if back { cuirass } else { breastplate });
        match self.body {
            TorsoBody::Plate {
                breast, fluting, ..
            } => match breast {
                Breast::Discs => pick("Triple-disc Cuirass", "Kardiophylax"),
                Breast::Muscle => Some("Muscle Cuirass"),
                Breast::Bell => Some("Bell Cuirass"),
                Breast::Tanko => Some("Tankō"),
                Breast::Box => Some("Kastenbrust"),
                Breast::Gothic => Some("Gothic Cuirass"),
                Breast::Globose => pick("Milanese Cuirass", "Milanese Breastplate"),
                Breast::Peascod => pick("Peascod Cuirass", "Peascod Breastplate"),
                Breast::Munition => pick("Munition Cuirass", "Munition Breastplate"),
                Breast::Leaf => Some("Elven Cuirass"),
                Breast::Panel => Some("Dwarven Cuirass"),
                Breast::Round if fluting == Fluting::All => Some("Maximilian Cuirass"),
                Breast::Round => None,
            },
            TorsoBody::Leather { breast, studded } => {
                if studded {
                    Some("Studded Leather")
                } else if self.material == Material::BuffLeather {
                    Some("Buff Jerkin")
                } else if breast == Breast::Muscle {
                    Some("Leather Muscle Cuirass")
                } else {
                    None
                }
            }
            TorsoBody::Mail {
                weave,
                rings,
                sleeves,
                ..
            } => {
                if self.material == Material::Mithril {
                    Some("Mithril Shirt")
                } else if weave == Weave::Japanese {
                    Some("Kusari Katabira")
                } else if self.style == Style::Classical {
                    Some("Lorica Hamata")
                } else if self.material == Material::Iron && rings == Rings::Coarse && sleeves {
                    Some("Byrnie")
                } else {
                    None
                }
            }
            TorsoBody::RingMail { .. } => None,
            TorsoBody::Padded { quilting, sleeves } => match (self.material, quilting) {
                _ if self.vest => None,
                (Material::Coir, _) => Some("Coir Armour"),
                (Material::Linen, Quilting::Laminated) => Some("Linothorax"),
                (Material::Cotton, Quilting::Laminated) => Some("Myeonje Baegap"),
                (Material::Cotton, Quilting::Squares) if !sleeves => Some("Ichcahuipilli"),
                _ => None,
            },
        }
    }

    pub fn name(&self) -> &'static str {
        self.specific_name().unwrap_or(self.generic_name())
    }
}
