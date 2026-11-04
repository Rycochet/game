//! `Look`: what can be seen of an item, which its picture is kept by, and `look_of`.

use bevy::prelude::*;
use shared::creature::{CreatureKind, Size};
use shared::item::{Item, Quality};
use shared::item_form::{BandWidth, GemForm, ItemForm, Mount, Pattern, Profile};
use shared::material::Material;

use super::looks::{
    ArmsLook, BladeLook, FeetLook, HandsLook, HeadLook, LegsLook, NeckLook, SetLook, ShouldersLook,
    TorsoLook, WaistLook,
};

/// Everything a picture shows. Ids stay out of it, so things that look the same share
/// one picture, and things that change their looks get a new one.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Look {
    Gem {
        gem: GemForm,
        quality: Option<Quality>,
    },
    Ring {
        ring: RingLook,
        quality: Option<Quality>,
    },
    Blade {
        blade: BladeLook,
        quality: Option<Quality>,
    },
    /// A pair of gauntlets or other hand pieces.
    Hands {
        hands: HandsLook,
        quality: Option<Quality>,
    },
    /// A pair of pauldrons or other shoulder pieces, as worn.
    Shoulders {
        shoulders: ShouldersLook,
        quality: Option<Quality>,
    },
    /// A cuirass or other torso piece, as worn.
    Torso {
        torso: TorsoLook,
        quality: Option<Quality>,
    },
    /// A helmet, coif or arming cap, as worn.
    Head {
        head: HeadLook,
        quality: Option<Quality>,
    },
    /// Vambraces, sleeves or other arm pieces, as worn.
    Arms {
        arms: ArmsLook,
        quality: Option<Quality>,
    },
    /// Greaves, chausses or other leg pieces, as worn.
    Legs {
        legs: LegsLook,
        quality: Option<Quality>,
    },
    /// A fauld, tassets, a skirt or another waist piece, as worn.
    Waist {
        waist: WaistLook,
        quality: Option<Quality>,
    },
    /// Sabatons, mail shoes or foot guards, as worn.
    Feet {
        feet: FeetLook,
        quality: Option<Quality>,
    },
    /// A collar, gorget, bevor, aventail or mantle, as worn.
    Neck {
        neck: NeckLook,
        quality: Option<Quality>,
    },
    /// Shoes, boots, sandals, clogs or slippers, a pair.
    Footwear {
        footwear: super::FootwearLook,
        quality: Option<Quality>,
    },
    /// An armour set: its pieces as worn together.
    Set {
        set: SetLook,
        quality: Option<Quality>,
    },
    /// A bare body of a kind that stands on two legs, standing: what worn things are
    /// built on, and later a character with what it wears.
    Body { kind: CreatureKind },
}

impl Look {
    /// The cells it is drawn to fill, wide and long: only the proportions matter.
    pub fn cells(&self) -> (u8, u8) {
        match self {
            Self::Gem { .. } | Self::Ring { .. } => (1, 1),
            Self::Blade { blade, .. } => blade.cells,
            Self::Hands { hands, .. } => hands.cells,
            Self::Shoulders { shoulders, .. } => shoulders.cells,
            Self::Torso { torso, .. } => torso.cells,
            Self::Head { head, .. } => head.cells,
            Self::Arms { arms, .. } => arms.cells,
            Self::Legs { legs, .. } => legs.cells,
            Self::Waist { waist, .. } => waist.cells,
            Self::Feet { feet, .. } => feet.cells,
            Self::Neck { neck, .. } => neck.cells,
            Self::Footwear { footwear, .. } => footwear.cells,
            Self::Set { set, .. } => set.cells,
            Self::Body { .. } => super::body::CELLS,
        }
    }

    /// The most cells it reaches along either side.
    pub fn longest(&self) -> u8 {
        let (wide, long) = self.cells();
        wide.max(long)
    }

    /// How big its picture is, given how many pixels its longest side has.
    pub fn picture_size(&self, pixels: u32) -> UVec2 {
        let (wide, long) = self.cells();
        let longest = self.longest() as u32;
        UVec2::new(wide as u32, long as u32) * pixels / longest
    }
}

/// What can be seen of a ring. Sizes are against the ring's own, as how big it is
/// doesn't show: only a gem too big for it does.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RingLook {
    pub(super) metal: Material,
    pub(super) profile: Profile,
    pub(super) width: BandWidth,
    pub(super) pattern: Pattern,
    pub(super) settings: Vec<SettingLook>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct SettingLook {
    pub(super) angle: u8,
    pub(super) mount: Mount,
    /// Sizes bigger than the ring.
    pub(super) bigger: i8,
    pub(super) gem: Option<SetGemLook>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct SetGemLook {
    pub(super) gem: GemForm,
    pub(super) quality: Option<Quality>,
    /// Sizes bigger than its setting.
    pub(super) bigger: i8,
}

/// How many sizes `size` is above `than`, or below if negative.
pub(super) fn sizes_above(size: Size, than: Size) -> i8 {
    size as i8 - than as i8
}

/// How this item looks, if it has a form to go by.
pub fn look_of(item: &Item) -> Option<Look> {
    let quality = item.quality;
    match item.form.as_ref()? {
        ItemForm::Blade(form) => Some(super::looks::blade_look(item, form)),
        ItemForm::Gem(gem) => Some(Look::Gem { gem: *gem, quality }),
        ItemForm::Ring(ring) => {
            let settings = ring.settings.iter().map(|setting| SettingLook {
                angle: setting.angle,
                mount: setting.mount,
                bigger: sizes_above(setting.size, item.made.size),
                gem: setting.gem.as_ref().and_then(|gem| match &gem.form {
                    Some(ItemForm::Gem(form)) => Some(SetGemLook {
                        gem: *form,
                        quality: gem.quality,
                        bigger: sizes_above(gem.made.size, setting.size),
                    }),
                    _ => None,
                }),
            });
            Some(Look::Ring {
                ring: RingLook {
                    metal: ring.metal,
                    profile: ring.profile,
                    width: ring.width,
                    pattern: ring.pattern,
                    settings: settings.collect(),
                },
                quality,
            })
        }
        ItemForm::Footwear(form) => {
            let shape = item.shape();
            Some(Look::Footwear {
                footwear: super::FootwearLook::new(*form, (shape.width, shape.height)),
                quality,
            })
        }
        _ => super::looks::armour_look(item),
    }
}
