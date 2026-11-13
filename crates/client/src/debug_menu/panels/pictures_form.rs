//! What Item Pictures shows, its sizes, and what each picture is of.

use bevy::prelude::*;
use shared::blade::Family;
use shared::body::{BEASTS, Frame, Hair, Layout};
use shared::creature::{CreatureKind, Size};
use shared::id::Id;
use shared::item::{Item, ItemKind, Quality};
use shared::item_form::{Cut, GemForm, give_form};
use shared::material::Material;
use shared::sheet::Appearance;

use super::page::PicturePage;
use crate::item_art::{Colours, Look, look_of};
use crate::widgets::TextSize;

/// How big each picture in Item Pictures' grid is shown, and is drawn, for sharpness.
pub(super) const PICTURE: f32 = 50.0;
pub(super) const PICTURE_PIXELS: u32 = 100;
/// How long a weapon is shown in Item Pictures' grid.
pub(super) const BLADE_PICTURE: f32 = 150.0;
/// How wide a pair of gauntlets is shown in Item Pictures' grid.
pub(super) const HANDS_PICTURE: f32 = 100.0;
pub(super) const LARGE_PICTURE: f32 = 256.0;
pub(super) const LARGE_PICTURE_PIXELS: u32 = 512;
pub(super) const PICTURE_LABEL: TextSize = TextSize::Small;
/// How many rings Item Pictures shows: a sample of what the server would roll.
pub(super) const RING_ROWS: u128 = 8;
pub(super) const RING_COLUMNS: u128 = 8;
/// Room for the name of a gem beside its row.
pub(super) const PICTURE_ROW_LABEL: f32 = 84.0;

/// What Item Pictures shows.
#[derive(Resource)]
pub(in crate::debug_menu) struct PicturesForm {
    pub(super) quality: Quality,
    pub(super) chosen: Shown,
}

impl Default for PicturesForm {
    fn default() -> Self {
        Self {
            quality: Quality::default(),
            chosen: Shown::Gem(gem_to_show(Material::Ruby, Cut::Round)),
        }
    }
}

/// One of each stone and cut. Each uncut stone has its own lumps.
pub(super) fn gem_to_show(material: Material, cut: Cut) -> GemForm {
    let shape_seed = if cut == Cut::Uncut {
        material as u16 + 1
    } else {
        0
    };
    GemForm {
        material,
        cut,
        shape_seed,
    }
}

/// What one picture in Item Pictures is of.
#[derive(Clone, Copy)]
pub(super) enum Shown {
    Gem(GemForm),
    /// A ring as the server would roll it from this number, at the quality chosen.
    Ring(u128),
    /// A weapon of this family, likewise.
    Blade(Family, u128),
    /// A kind's bare body, at its usual height beside a person.
    Body(CreatureKind),
    /// One of `BEASTS` drawn before it is a kind, filling its picture.
    Beast(usize),
    /// A character, to show its hair.
    Hair(super::hair::HairShown),
    /// A piece of armour.
    Armour(super::armour::ArmourShown),
    /// A pair of footwear.
    Footwear(super::footwear::FootwearShown),
    /// A piece of underclothing.
    Under(super::under::UnderShown),
    /// A kind in one of its markings.
    Marked(super::markings::MarkedShown),
}

impl Shown {
    pub(super) fn look(self, quality: Quality) -> Option<Look> {
        match self {
            Self::Gem(gem) => Some(Look::Gem {
                gem,
                quality: Some(quality),
            }),
            Self::Ring(seed) => {
                let mut ring = Item::new(Id::for_test(seed), ItemKind::Ring);
                ring.quality = Some(quality);
                let mut gems = seed << 32;
                give_form(&mut ring, Some(Size::Medium), &mut || {
                    gems += 1;
                    Id::for_test(gems)
                });
                look_of(&ring)
            }
            Self::Blade(family, seed) => {
                let mut blade = Item::new(Id::for_test(seed), family.kind());
                blade.quality = Some(quality);
                give_form(&mut blade, Some(Size::Medium), &mut || Id::for_test(0));
                look_of(&blade)
            }
            Self::Body(kind) => Some(Look::Body {
                layout: Box::new(Layout::of(kind)?),
                height: Frame::usual_height(kind),
                colours: Colours::of(kind, &Appearance::default()),
                hair: Appearance::default().hair(kind),
            }),
            Self::Beast(index) => Some(Look::Body {
                layout: Box::new(BEASTS[index].layout()),
                height: 0,
                colours: Colours::PLAIN,
                hair: Hair::NONE,
            }),
            Self::Hair(shown) => shown.look(quality),
            Self::Armour(shown) => shown.look(quality),
            Self::Footwear(shown) => shown.look(quality),
            Self::Under(shown) => shown.look(quality),
            Self::Marked(shown) => shown.look(),
        }
    }
}

/// A picture in Item Pictures' grid: pressing it shows it large.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct GemPicture(pub(super) Shown);

#[derive(Component)]
pub(in crate::debug_menu) struct LargePicture;

#[derive(Component)]
pub(in crate::debug_menu) struct PictureQualities;

/// The dropdown choosing Item Pictures' page.
#[derive(Component)]
pub(in crate::debug_menu) struct PicturePages;

/// Holds the rows of Item Pictures' page, and says which page they are, once filled.
#[derive(Component, Default)]
pub(in crate::debug_menu) struct PicturesGrid(pub(super) Option<PicturePage>);
