//! What a head piece is shaped like: each property a helmet rolls.

use serde::{Deserialize, Serialize};

/// How the bowl over the skull is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Bowl {
    /// Raised from one piece.
    Raised,
    /// Two halves riveted along a ridge over the top: late Roman, Vendel.
    Ridged,
    /// Plates in a frame of bands: spangenhelm.
    Segmented,
    /// Many narrow plates riveted round: kabuto, steppe helmets.
    Plates,
    /// Small plates laced together.
    Laced,
    /// Hardened leather moulded to shape.
    Moulded,
    /// Rattan woven over a frame.
    Woven,
    /// Cloth or mail, no bowl: caps and coifs.
    Soft,
}

/// The shape of the skull.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skull {
    Round,
    /// Shallow, flatter on top.
    Low,
    /// Rising to a point.
    Conical,
    /// Pointed, the point towards the back: bascinet, sugarloaf.
    Ogival,
    /// Upright sides and a flat top: the early great helm.
    Flat,
    /// Bulging, then drawn up to a spike: turban helmets.
    Onion,
    /// The top curling forward: Phrygian.
    Forward,
}

/// How a moving visor is shaped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visor {
    /// Rounded round the face: armet, great bascinet.
    Rounded,
    /// Drawn forward to a snout: hounskull.
    Pointed,
    /// Small, hinged at the brow.
    Klapp,
    /// Ridged across, with a sight slit: close helmet.
    Bellows,
    /// A face of its own: a skull, a beast.
    Grotesque,
}

/// What covers the face.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Face {
    Open,
    /// A bar down the nose.
    Nasal,
    /// Guards round the eyes.
    Spectacles,
    /// Bars down from a peak: lobster pot.
    Bars,
    /// A T- or Y-shaped opening, the cheeks part of the bowl: barbute, Corinthian.
    Opening,
    /// Fixed, with sight slits and breaths: great helm.
    Faceplate,
    /// One sight slit under a raised front: frog-mouth helm, some sallets.
    Slit,
    /// Moving.
    Visor(Visor),
    /// A face, fixed or tied on: Roman cavalry, mempō.
    Mask,
}

impl Face {
    /// Whether it covers the face, so the helmet is a helm.
    pub fn closed(self) -> bool {
        matches!(
            self,
            Self::Faceplate | Self::Slit | Self::Visor(_) | Self::Mask
        )
    }
}

/// What guards the back of the neck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Back {
    None,
    /// A small lip turned out.
    Flare,
    /// A broad plate standing out: Roman galea.
    Guard,
    /// The bowl sweeping back in one piece: sallet.
    Tail,
    /// Overlapping strips: lobster tail, shikoro.
    Lames,
    /// Hanging lamellar or quilted panels to the shoulders.
    Flaps,
}

/// What stands out from the bowl's edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Brim {
    None,
    /// At the front only.
    Peak,
    /// All round, narrow.
    Narrow,
    /// All round, wide and sloping: kettle hat.
    Wide,
    /// Curving up to points fore and aft: morion.
    Boat,
}

impl Brim {
    /// All the way round, so a war hat.
    pub fn round(self) -> bool {
        matches!(self, Self::Narrow | Self::Wide | Self::Boat)
    }
}

/// What is on top.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Top {
    None,
    Knob,
    Spike,
    /// A little point, curled back: cabasset.
    Stalk,
    /// A low ridge front to back.
    Comb,
    /// A tall one: morion, burgonet.
    HighComb,
    /// Horsehair on a holder, front to back.
    Crest,
    /// A feather in a tube.
    Plume,
}

/// Something added for show: rare, and mostly from stories.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ornament {
    None,
    Horns,
    Wings,
    /// A badge or two prongs at the brow: Japanese maedate, kuwagata.
    FrontCrest,
    /// A toothed band round the brow.
    Crown,
    Antlers,
    /// A dragon's head and wings over the top.
    Dragon,
}

impl Ornament {
    /// The word put before a name, for those that change it.
    pub fn prefix(self) -> Option<&'static str> {
        match self {
            Self::Horns => Some("Horned"),
            Self::Wings => Some("Winged"),
            Self::Crown => Some("Crowned"),
            Self::Antlers => Some("Antlered"),
            Self::Dragon => Some("Dragon"),
            Self::None | Self::FrontCrest => None,
        }
    }
}
