//! What one item is physically like, rolled once when the server makes it and saved
//! on the item from then on. Changing the rolls never changes items already made.
//! See `docs/context/shared/item.md`.
//!
//! Each kind rolls its choices in order, each seeing those before it. A choice can
//! lean on an earlier one by changing weights, or limit it with a weight of 0.

use serde::{Deserialize, Serialize};

use crate::armour::arms::ArmsForm;
use crate::armour::hands::HandsForm;
use crate::armour::head::HeadForm;
use crate::armour::shoulders::ShouldersForm;
use crate::armour::torso::TorsoForm;
use crate::blade::{self, BladeForm};
use crate::creature::Size;
use crate::id::Id;
use crate::item::{Item, ItemKind, Made, Quality};
use crate::material::Material;
use crate::rng::Rng;

mod armour;
mod limits;
mod measure;
mod words;

pub use limits::Limits;
use words::{counted, gem_material, plural};

/// What an item is like, for kinds that have this.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemForm {
    Gem(GemForm),
    Ring(Box<RingForm>),
    Blade(Box<BladeForm>),
    /// Gauntlets and other hand pieces.
    Hands(HandsForm),
    /// A pair of pauldrons or other shoulder pieces.
    Shoulders(ShouldersForm),
    /// A cuirass or other torso armour, or an arming vest.
    Torso(TorsoForm),
    /// A helmet, coif or arming cap.
    Head(HeadForm),
    /// Vambraces, sleeves or other arm pieces, for one arm or both.
    Arms(ArmsForm),
}

impl ItemForm {
    /// What something like this is called, without its size.
    pub fn name(&self) -> String {
        match self {
            Self::Gem(gem) => format!("{} {}", gem.cut.name(), gem.material.name()),
            Self::Ring(ring) => ring.name(),
            Self::Blade(blade) => blade.name().to_owned(),
            Self::Hands(hands) => hands.name().to_owned(),
            Self::Shoulders(shoulders) => shoulders.name().to_owned(),
            Self::Torso(torso) => torso.name().to_owned(),
            Self::Head(head) => head.name(),
            Self::Arms(arms) => arms.name(),
        }
    }

    /// The gems set in it.
    pub fn gems(&self) -> impl Iterator<Item = &Item> {
        let settings = match self {
            Self::Ring(ring) => ring.settings.as_slice(),
            Self::Gem(_)
            | Self::Blade(_)
            | Self::Hands(_)
            | Self::Shoulders(_)
            | Self::Torso(_)
            | Self::Head(_)
            | Self::Arms(_) => &[],
        };
        settings.iter().filter_map(|setting| setting.gem.as_ref())
    }
}

/// How a gem has been cut, or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cut {
    Cushion,
    Emerald,
    Oval,
    Pear,
    Princess,
    Round,
    Trillion,
    Uncut,
}

impl Cut {
    pub const ALL: [Self; 8] = [
        Self::Cushion,
        Self::Emerald,
        Self::Oval,
        Self::Pear,
        Self::Princess,
        Self::Round,
        Self::Trillion,
        Self::Uncut,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Cushion => "Cushion",
            Self::Emerald => "Emerald",
            Self::Oval => "Oval",
            Self::Pear => "Pear",
            Self::Princess => "Princess",
            Self::Round => "Round",
            Self::Trillion => "Trillion",
            Self::Uncut => "Uncut",
        }
    }
}

/// One gem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GemForm {
    pub material: Material,
    pub cut: Cut,
    /// The lumps of an uncut stone. 0 for a cut one, whose looks don't depend on it.
    pub shape_seed: u16,
}

/// The shape of a ring's band, across.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Profile {
    Domed,
    Flat,
    Knife,
    Round,
    Square,
}

/// How wide a ring's band is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BandWidth {
    Thin,
    Medium,
    Wide,
}

/// What has been done to the surface of a ring's band.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pattern {
    Engraved,
    Hammered,
    Plain,
    Twisted,
}

/// How a setting holds its gem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mount {
    Bezel,
    Claws,
}

/// A place on a ring for a gem, which stays when the gem is taken out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    /// How far round the band from the top, in 256ths of a turn.
    pub angle: u8,
    pub size: Size,
    pub mount: Mount,
    /// A whole item, with its own id, or nothing.
    pub gem: Option<Item>,
}

impl Setting {
    /// Whether this could be set here: a cut gem no more than one size too big.
    pub fn takes(&self, item: &Item) -> bool {
        let cut = match &item.form {
            Some(ItemForm::Gem(gem)) => gem.cut != Cut::Uncut,
            _ => false,
        };
        cut && item.made.size <= self.size.stepped(1)
    }
}

/// One ring.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RingForm {
    pub metal: Material,
    pub profile: Profile,
    pub width: BandWidth,
    pub pattern: Pattern,
    pub settings: Vec<Setting>,
}

impl RingForm {
    /// Its metal and what is in its settings, as it is now.
    fn name(&self) -> String {
        let mut stones: Vec<(Material, usize)> = Vec::new();
        let mut empty = 0;
        for setting in &self.settings {
            match setting.gem.as_ref().and_then(gem_material) {
                Some(stone) => match stones.iter_mut().find(|(other, _)| *other == stone) {
                    Some((_, count)) => *count += 1,
                    None => stones.push((stone, 1)),
                },
                None => empty += 1,
            }
        }
        let mut parts: Vec<String> = stones
            .into_iter()
            .map(|(stone, count)| counted(count, stone.name(), plural(stone)))
            .collect();
        if empty > 0 {
            parts.push(counted(empty, "Empty Setting", "Empty Settings".to_owned()));
        }
        let ring = format!("{} Ring", self.metal.name());
        match parts.split_last() {
            None => ring,
            Some((only, [])) => format!("{ring} with {only}"),
            Some((last, others)) => format!("{ring} with {} and {last}", others.join(", ")),
        }
    }
}

/// Gives a new item its form, for kinds that have one, going by its quality. `size`
/// is what it must be, such as a mob's own; without one, it is rolled. Only the server
/// makes items, so anything else it needs, such as gems, gets its id from `new_id`.
pub fn give_form(item: &mut Item, size: Option<Size>, new_id: &mut impl FnMut() -> Id) {
    let mut rng = item.id.seed().derive("item form").rng();
    let quality = item.quality.unwrap_or_default();
    let armour = armour::armour_form(item.kind, &mut rng, quality);
    let (form, rolled_size) = match item.kind {
        _ if let Some(form) = armour => (form, size.unwrap_or(Size::Medium)),
        ItemKind::Gem => {
            let (gem, size) = roll_gem(&mut rng, quality, true);
            (ItemForm::Gem(gem), size)
        }
        ItemKind::Ring => {
            let size = size.unwrap_or_else(|| roll_ring_size(&mut rng));
            let made_at = item.made_at;
            let ring = roll_ring(&mut rng, quality, size, &mut |gem, gem_size, quality| {
                let mut stone = Item::new(new_id(), ItemKind::Gem);
                stone.made = Made::for_body(ItemKind::Gem, gem_size, None, false);
                stone.quality = Some(quality);
                stone.made_at = made_at;
                stone.form = Some(ItemForm::Gem(gem));
                stone
            });
            (ItemForm::Ring(Box::new(ring)), size)
        }
        kind => {
            let Some(family) = kind.blade_family() else {
                return;
            };
            let size = size.unwrap_or(Size::Medium);
            let blade = blade::roll_blade_at(&mut rng, family, quality, size);
            (ItemForm::Blade(Box::new(blade)), size)
        }
    };
    let size = size.unwrap_or(rolled_size);
    item.made = Made::for_body(item.kind, size, item.made.fingers, item.made.magical);
    item.form = Some(form);
}

/// Gives a new blade its form within limits, which may make it another family's kind.
/// `false` if nothing can be made within them, so the item shouldn't exist.
pub fn give_blade_within(item: &mut Item, limits: &Limits) -> bool {
    let Some(family) = item.kind.blade_family() else {
        return false;
    };
    let mut rng = item.id.seed().derive("item form").rng();
    let quality = item.quality.unwrap_or_default();
    let rolled = blade::roll_blade_within(&mut rng, family, quality, limits);
    rolled.map(|rolled| set_blade(item, rolled)).is_some()
}

/// Makes a new item into a specific weapon at any size, keeping its name: only for when
/// that is asked for, as a commission from a shop or an ability might. `false` if there
/// is no such weapon, or it can't keep to the limits.
pub fn give_commission(item: &mut Item, name: &str, size: Size, limits: &Limits) -> bool {
    let mut rng = item.id.seed().derive("item form").rng();
    let quality = item.quality.unwrap_or_default();
    let rolled = blade::commission_blade(&mut rng, name, quality, size, limits);
    rolled.map(|rolled| set_blade(item, rolled)).is_some()
}

fn set_blade(item: &mut Item, rolled: blade::Rolled) {
    item.kind = rolled.form.family.kind();
    item.made = Made::for_body(item.kind, rolled.size, item.made.fingers, item.made.magical);
    item.form = Some(ItemForm::Blade(Box::new(rolled.form)));
}

/// Quality, then material (better favours finer), then cut (by material), then size
/// (finer stones come smaller).
fn roll_gem(rng: &mut Rng, quality: Quality, may_be_uncut: bool) -> (GemForm, Size) {
    let material = rng.pick_weighted(&Material::GEMS.map(|gem| (gem, fine_weight(gem, quality))));
    let cut = roll_cut(rng, material, may_be_uncut);
    let size = rng.pick_weighted(&Size::ALL.map(|size| (size, size_weight(size, material))));
    let shape_seed = if cut == Cut::Uncut {
        rng.range_u64(1, 1 << 16) as u16
    } else {
        0
    };
    let gem = GemForm {
        material,
        cut,
        shape_seed,
    };
    (gem, size)
}

fn roll_cut(rng: &mut Rng, material: Material, may_be_uncut: bool) -> Cut {
    let weight = |cut| match (cut, may_be_uncut) {
        (Cut::Uncut, false) => 0,
        _ => cut_weight(cut, material),
    };
    rng.pick_weighted(&Cut::ALL.map(|cut| (cut, weight(cut))))
}

/// Finer materials are rarer, less so the better the quality.
fn fine_weight(material: Material, quality: Quality) -> u64 {
    let tier = material.tier();
    let base = match tier {
        1 => 40,
        2 => 25,
        3 => 12,
        _ => 4,
    };
    base * (1 + quality as u64 * tier * tier)
}

/// How often each cut is used, and the ones a stone is known for more often.
fn cut_weight(cut: Cut, material: Material) -> u64 {
    let base = match cut {
        Cut::Round => 30,
        Cut::Oval => 20,
        Cut::Cushion => 15,
        Cut::Uncut => 12,
        Cut::Princess | Cut::Pear => 10,
        Cut::Emerald | Cut::Trillion => 5,
    };
    let known_for = matches!(
        (material, cut),
        (Material::Emerald, Cut::Emerald) | (Material::Diamond, Cut::Round | Cut::Princess)
    );
    if known_for { base * 6 } else { base }
}

/// Mostly around medium. Whole-number division leaves the finest with no giants.
fn size_weight(size: Size, material: Material) -> u64 {
    let tier = material.tier();
    match size {
        Size::Tiny => 10 + 5 * tier,
        Size::Small => 30,
        Size::Medium => 40,
        Size::Large => 16 / tier,
        Size::Huge => 6 / tier,
        Size::Giant => 2 / tier,
    }
}

/// The share out of 100 of settings left empty when a ring is made.
const EMPTY_PERCENT: u64 = 5;
/// The share out of 100 of gems after the first that match it.
const SAME_GEM_PERCENT: u64 = 90;
/// The share out of 100 of gems a size too big for their setting: people do it.
const OVERSIZED_PERCENT: u64 = 3;
/// The share out of 100 of gems a size smaller than their setting.
const UNDERSIZED_PERCENT: u64 = 20;
/// How far apart settings go round a band, in 256ths of a turn.
const SETTING_SPACING: u8 = 18;

/// A ring made for anyone: mostly for those of ordinary size.
fn roll_ring_size(rng: &mut Rng) -> Size {
    rng.pick_weighted(&[
        (Size::Tiny, 5),
        (Size::Small, 15),
        (Size::Medium, 60),
        (Size::Large, 15),
        (Size::Huge, 4),
        (Size::Giant, 1),
    ])
}

/// The metals rings are made of, in the order they were first rolled from, so rolls
/// stay as they were when more metals came for armour.
const RING_METALS: [Material; 4] = [
    Material::Copper,
    Material::Gold,
    Material::Iron,
    Material::Silver,
];

/// Quality, then metal (better favours finer), then its band, then how many settings
/// (finer and better have more), then the gems. `new_gem` makes each gem an item.
fn roll_ring(
    rng: &mut Rng,
    quality: Quality,
    size: Size,
    new_gem: &mut impl FnMut(GemForm, Size, Quality) -> Item,
) -> RingForm {
    let metals: Vec<_> = RING_METALS
        .into_iter()
        .map(|metal| (metal, fine_weight(metal, quality)))
        .collect();
    let metal = rng.pick_weighted(&metals);
    let profile = rng.pick_weighted(&[
        (Profile::Domed, 30),
        (Profile::Flat, 30),
        (Profile::Round, 25),
        (Profile::Square, 10),
        (Profile::Knife, 5),
    ]);
    let width = rng.pick_weighted(&[
        (BandWidth::Thin, 30),
        (BandWidth::Medium, 50),
        (BandWidth::Wide, 20),
    ]);
    let pattern = rng.pick_weighted(&[
        (Pattern::Plain, 60),
        (Pattern::Hammered, 15),
        (Pattern::Engraved, 15),
        (Pattern::Twisted, 10),
    ]);
    let count = roll_setting_count(rng, metal, quality);
    let smaller: i8 = rng.pick_weighted(&[(0, 50), (1, 35), (2, 15)]);
    let setting_size = size.stepped(-smaller);
    let mount = rng.pick_weighted(&[(Mount::Claws, 60), (Mount::Bezel, 40)]);
    let mut first: Option<GemForm> = None;
    let mut settings = Vec::new();
    for index in 0..count {
        let mut gem = None;
        if rng.range_u64(0, 100) >= EMPTY_PERCENT {
            let form = match first {
                Some(same) if rng.range_u64(0, 100) < SAME_GEM_PERCENT => same,
                _ => roll_gem(rng, quality, false).0,
            };
            first.get_or_insert(form);
            let gem_size = roll_gem_size(rng, setting_size);
            gem = Some(new_gem(form, gem_size, roll_gem_quality(rng, quality)));
        }
        settings.push(Setting {
            angle: setting_angle(index, count),
            size: setting_size,
            mount,
            gem,
        });
    }
    RingForm {
        metal,
        profile,
        width,
        pattern,
        settings,
    }
}

/// Most rings of poor metal have none. A finer metal and a better make have more.
fn roll_setting_count(rng: &mut Rng, metal: Material, quality: Quality) -> u8 {
    let fine = metal.tier() + quality as u64;
    rng.pick_weighted(&[
        (0, 60),
        (1, fine * 8),
        (2, fine * 3),
        (3, fine),
        // Only the very finest. Whole-number division leaves the rest without.
        (5, fine * fine / 16),
    ])
}

/// In a row across the top of the band, evenly either side of it.
fn setting_angle(index: u8, count: u8) -> u8 {
    let from_first = index as i32 * SETTING_SPACING as i32;
    let row = (count as i32 - 1) * SETTING_SPACING as i32;
    (from_first - row / 2).rem_euclid(256) as u8
}

/// Usually made for the setting, sometimes smaller, and rarely too big.
fn roll_gem_size(rng: &mut Rng, setting: Size) -> Size {
    let roll = rng.range_u64(0, 100);
    if roll < OVERSIZED_PERCENT {
        setting.stepped(1)
    } else if roll < OVERSIZED_PERCENT + UNDERSIZED_PERCENT {
        setting.stepped(-1)
    } else {
        setting
    }
}

/// Near the ring's, which says how well it was made rather than what is in it. Only
/// rolled artifacts are named, so set gems stop below that.
fn roll_gem_quality(rng: &mut Rng, ring: Quality) -> Quality {
    let step: i32 = rng.pick_weighted(&[(-1, 25), (0, 60), (1, 15)]);
    let best = Quality::Legendary as i32;
    Quality::ALL[(ring as i32 + step).clamp(0, best) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Category;

    fn made(kind: ItemKind, seed: u128, quality: Quality, size: Option<Size>) -> Item {
        let mut item = Item::new(Id::for_test(seed), kind);
        item.quality = Some(quality);
        let mut next = 1 << 64;
        give_form(&mut item, size, &mut || {
            next += 1;
            Id::for_test(next)
        });
        item
    }

    fn gem(seed: u128, quality: Quality) -> (GemForm, Size) {
        let item = made(ItemKind::Gem, seed, quality, None);
        match item.form {
            Some(ItemForm::Gem(gem)) => (gem, item.made.size),
            other => panic!("{other:?}"),
        }
    }

    fn ring_form(item: &Item) -> &RingForm {
        match &item.form {
            Some(ItemForm::Ring(ring)) => ring,
            other => panic!("{other:?}"),
        }
    }

    fn rings(quality: Quality) -> impl Iterator<Item = Item> {
        (0..4000).map(move |seed| made(ItemKind::Ring, seed, quality, None))
    }

    fn share_of(quality: Quality, wanted: impl Fn(&GemForm, Size) -> bool) -> u32 {
        let found = (0..4000).filter(|&seed| {
            let (gem, size) = gem(seed, quality);
            wanted(&gem, size)
        });
        found.count() as u32
    }

    fn stone(material: Material) -> Setting {
        Setting {
            angle: 0,
            size: Size::Medium,
            mount: Mount::Claws,
            gem: Some(Item {
                form: Some(ItemForm::Gem(GemForm {
                    material,
                    cut: Cut::Round,
                    shape_seed: 0,
                })),
                ..Item::new(Id::for_test(1), ItemKind::Gem)
            }),
        }
    }

    fn ring_with(settings: Vec<Setting>) -> ItemForm {
        ItemForm::Ring(Box::new(RingForm {
            metal: Material::Gold,
            profile: Profile::Flat,
            width: BandWidth::Medium,
            pattern: Pattern::Plain,
            settings,
        }))
    }

    #[test]
    fn the_same_seed_rolls_the_same_gem() {
        // Golden: changing the rolls is fine, but should be noticed.
        let (rolled, size) = gem(1, Quality::Common);
        assert_eq!(gem(1, Quality::Common), (rolled, size));
        assert_eq!(
            (rolled.material, rolled.cut, size),
            (Material::Emerald, Cut::Oval, Size::Medium)
        );
    }

    #[test]
    fn the_same_seed_rolls_the_same_ring() {
        let one = made(ItemKind::Ring, 7, Quality::Rare, None);
        assert_eq!(one, made(ItemKind::Ring, 7, Quality::Rare, None));
    }

    #[test]
    fn blades_take_their_weight_and_shape_from_their_form() {
        let sword = made(ItemKind::Sword, 1, Quality::Common, None);
        let Some(ItemForm::Blade(form)) = &sword.form else {
            panic!("{sword:?}");
        };
        assert_eq!(sword.total_weight(), form.grams());
        assert_eq!(sword.shape(), form.shape());
        assert_eq!(sword.name(), form.name());
        let giant = made(ItemKind::Sword, 1, Quality::Common, Some(Size::Giant));
        assert_eq!(giant.name(), "Giant Sword");
    }

    #[test]
    fn a_commission_is_named_with_its_size() {
        let mut item = Item::new(Id::for_test(1), ItemKind::Coin);
        assert!(give_commission(
            &mut item,
            "Katana",
            Size::Tiny,
            &Limits::NONE
        ));
        assert_eq!(item.kind, ItemKind::Sword);
        assert_eq!(item.name(), "Tiny Katana");
    }

    #[test]
    fn a_blade_within_limits_may_change_kind_or_not_be_made() {
        let limits = blade::limits_for_wielder(Size::Tiny, 5000);
        let got = (0..50).map(|seed| {
            let mut item = Item::new(Id::for_test(seed), ItemKind::Sword);
            assert!(give_blade_within(&mut item, &limits));
            item.kind
        });
        assert!(got.into_iter().any(|kind| kind != ItemKind::Sword));
        let mut item = Item::new(Id::for_test(1), ItemKind::Sword);
        let none = Limits {
            most_grams: Some(1),
            ..Limits::NONE
        };
        assert!(!give_blade_within(&mut item, &none));
    }

    #[test]
    fn only_kinds_with_a_form_roll_one() {
        let apple = made(ItemKind::Apple, 1, Quality::Common, None);
        assert_eq!(apple.form, None);
    }

    #[test]
    fn a_size_given_is_kept_and_the_gem_stays_the_same() {
        let told = made(ItemKind::Gem, 1, Quality::Common, Some(Size::Giant));
        assert_eq!(told.made.size, Size::Giant);
        assert_eq!(told.form, Some(ItemForm::Gem(gem(1, Quality::Common).0)));
    }

    #[test]
    fn every_gem_rolled_is_a_gem_and_only_uncut_ones_have_a_shape_seed() {
        for seed in 0..4000 {
            let (gem, _) = gem(seed, Quality::Rare);
            assert_eq!(gem.material.category(), Category::Gem);
            assert_eq!(gem.cut == Cut::Uncut, gem.shape_seed != 0, "{gem:?}");
        }
    }

    #[test]
    fn better_gems_are_more_often_fine() {
        let diamonds = |quality| share_of(quality, |gem, _| gem.material == Material::Diamond);
        assert!(diamonds(Quality::Legendary) > diamonds(Quality::Poor) * 4);
    }

    #[test]
    fn stones_favour_the_cuts_they_are_known_for() {
        let emerald_cut = |wanted: Material| {
            let of_it = (0..40_000).map(|seed| gem(seed, Quality::Rare).0);
            let of_it: Vec<_> = of_it.filter(|gem| gem.material == wanted).collect();
            let cut = of_it.iter().filter(|gem| gem.cut == Cut::Emerald).count();
            cut as f64 / of_it.len() as f64
        };
        assert!(emerald_cut(Material::Emerald) > emerald_cut(Material::Ruby) * 3.0);
    }

    #[test]
    fn the_finest_stones_never_come_giant() {
        let fine_giants = |gem: &GemForm, size| gem.material.tier() >= 3 && size == Size::Giant;
        assert_eq!(share_of(Quality::Legendary, fine_giants), 0);
        let any_giants = |_: &GemForm, size| size == Size::Giant;
        assert!(share_of(Quality::Common, any_giants) > 0);
    }

    #[test]
    fn a_gem_is_named_by_its_cut_and_stone() {
        let form = ItemForm::Gem(GemForm {
            material: Material::Ruby,
            cut: Cut::Uncut,
            shape_seed: 3,
        });
        assert_eq!(form.name(), "Uncut Ruby");
    }

    #[test]
    fn a_ring_is_named_by_its_metal_and_what_is_in_it() {
        assert_eq!(ring_with(vec![]).name(), "Gold Ring");
        assert_eq!(
            ring_with(vec![stone(Material::Ruby)]).name(),
            "Gold Ring with a Ruby"
        );
        assert_eq!(
            ring_with(vec![stone(Material::Emerald)]).name(),
            "Gold Ring with an Emerald"
        );
        assert_eq!(
            ring_with(vec![stone(Material::Ruby), stone(Material::Ruby)]).name(),
            "Gold Ring with Two Rubies"
        );
        let mut empty = stone(Material::Ruby);
        empty.gem = None;
        assert_eq!(
            ring_with(vec![stone(Material::Ruby), stone(Material::Topaz), empty]).name(),
            "Gold Ring with a Ruby, a Topaz and an Empty Setting"
        );
    }

    #[test]
    fn rings_are_metal_and_their_gems_are_items_of_their_own_that_fit() {
        for ring in rings(Quality::Epic) {
            assert_eq!(ring_form(&ring).metal.category(), Category::Metal);
            let mut ids = std::collections::BTreeSet::new();
            for setting in &ring_form(&ring).settings {
                assert!(setting.size <= ring.made.size);
                if let Some(gem) = &setting.gem {
                    assert_eq!(gem.kind, ItemKind::Gem);
                    assert!(setting.takes(gem), "{gem:?} in {:?}", setting.size);
                    assert!(ids.insert(gem.id), "the same id twice in a ring");
                    assert!(gem.quality < Some(Quality::Artifact));
                }
            }
        }
    }

    #[test]
    fn a_setting_takes_only_cut_gems_up_to_one_size_too_big() {
        let setting = stone(Material::Ruby);
        let gem = setting.gem.clone().unwrap();
        let at = |size| Item {
            made: Made::for_body(ItemKind::Gem, size, None, false),
            ..gem.clone()
        };
        assert!(setting.takes(&at(Size::Tiny)));
        assert!(setting.takes(&at(Size::Medium)));
        assert!(setting.takes(&at(Size::Large)));
        assert!(!setting.takes(&at(Size::Huge)));
        let uncut = Item {
            form: Some(ItemForm::Gem(GemForm {
                material: Material::Ruby,
                cut: Cut::Uncut,
                shape_seed: 1,
            })),
            ..gem
        };
        assert!(!setting.takes(&uncut));
        assert!(!setting.takes(&Item::new(Id::for_test(2), ItemKind::Apple)));
    }

    #[test]
    fn several_gems_in_a_ring_mostly_match_the_first() {
        let (mut same, mut others) = (0, 0);
        for ring in rings(Quality::Legendary) {
            let stones: Vec<_> = ring.set_gems().filter_map(gem_material).collect();
            if let Some((first, rest)) = stones.split_first() {
                same += rest.iter().filter(|stone| *stone == first).count();
                others += rest.iter().filter(|stone| *stone != first).count();
            }
        }
        assert!(same > others * 8, "{same} against {others}");
    }

    #[test]
    fn some_settings_are_left_empty_but_few() {
        let settings: Vec<bool> = rings(Quality::Legendary)
            .flat_map(|ring| {
                let settings = ring_form(&ring).settings.clone();
                settings.into_iter().map(|setting| setting.gem.is_some())
            })
            .collect();
        let empty = settings.iter().filter(|filled| !**filled).count();
        assert!(empty > 0);
        assert!(empty * 10 < settings.len(), "{empty} of {}", settings.len());
    }

    #[test]
    fn poor_rings_mostly_have_no_settings_and_fine_ones_mostly_do() {
        let with_settings = |quality| {
            rings(quality)
                .filter(|ring| !ring_form(ring).settings.is_empty())
                .count()
        };
        assert!(with_settings(Quality::Poor) < 1500);
        assert!(with_settings(Quality::Legendary) > 2000);
    }

    #[test]
    fn better_rings_are_more_often_of_finer_metal() {
        let gold = |quality| {
            rings(quality)
                .filter(|ring| ring_form(ring).metal == Material::Gold)
                .count()
        };
        assert!(gold(Quality::Legendary) > gold(Quality::Poor) * 3);
    }

    #[test]
    fn settings_sit_evenly_either_side_of_the_top() {
        assert_eq!(setting_angle(0, 1), 0);
        assert_eq!(setting_angle(1, 3), 0);
        assert_eq!(setting_angle(0, 3), (256 - SETTING_SPACING as i32) as u8);
        assert_eq!(setting_angle(2, 3), SETTING_SPACING);
    }

    #[test]
    fn a_ring_given_a_size_keeps_it() {
        let ring = made(ItemKind::Ring, 3, Quality::Rare, Some(Size::Huge));
        assert_eq!(ring.made.size, Size::Huge);
    }

    #[test]
    fn set_gems_count_towards_a_rings_weight() {
        let found = rings(Quality::Legendary)
            .find(|ring| ring.set_gems().count() > 1)
            .unwrap();
        let gems = found.set_gems().count() as u32;
        assert_eq!(
            found.total_weight(),
            ItemKind::Ring.def().weight + gems * ItemKind::Gem.def().weight
        );
    }
}
