//! Give Item's panel, as markup, laid out to read like what it makes ("Give 3
//! Large Size Identical / Quality Rare Set Blessed Magical Longsword").

use bevy::prelude::*;
use shared::creature::Size;
use shared::item::Quality;

use super::{
    FIELDS, GiveButton, GiveChoice, GiveCount, GiveCountStep, GiveNoSpace, GiveOption, GivePreview,
    GiveRandomSeed, GiveSeed, GiveToWords, SetChoice, give_kinds, ui_name,
};
use crate::dom::Template;
use crate::html;
use crate::widgets::UiId;

/// The panel's contents. The dropdowns' choices are given by
/// `start_give_panel` (`GiveChoice::options`) and, for the item's form, by
/// `fill_give_options`, which also shows that line only for kinds that take
/// any. The item to be given is drawn in the canvas.
pub(super) static GIVE_ITEM: Template = html!(
    "debug_menu.give_item",
    r#"
<form id="debug.give_item">
  <panel id="panel" moveable>
    <span nowrap valign="middle">
      Give
      <button id="less">-</button>
      <input id="count" type="text" inputmode="numeric" value="1" width="64"/>
      <button id="more">+</button>
      <select id="size" width="130"><option>Size</option></select>
      Size
      <select id="identical" width="120"><option>Identical</option></select>
    </span>
    <span nowrap valign="middle">
      Quality
      <select id="quality" width="130"><option>Common</option></select>
      <select id="set" width="100"><option>Set</option></select>
      <select id="favour" width="130"><option>Unaligned</option></select>
      <select id="magic" width="120"><option>Mundane</option></select>
      <select id="kind" searchable width="220"><option>Kind</option></select>
    </span>
    <span id="form_line" valign="middle" hidden>
      Form
      <select id="style" width="170" hidden><option>Any style</option></select>
      <select id="construction" width="170" hidden><option>Any construction</option></select>
      <select id="material" width="170" hidden><option>Any material</option></select>
      <select id="finish" width="170" hidden><option>Any finish</option></select>
      <select id="dye" width="170" hidden><option>Any colour</option></select>
      <select id="settings" width="170" hidden><option>Any settings</option></select>
      <select id="gem_material" width="170" hidden><option>Any gem</option></select>
      <select id="gem_cut" width="170" hidden><option>Any cut</option></select>
      <select id="limits" width="170" hidden><option>Any length</option></select>
    </span>
    <span nowrap valign="middle">
      <label for="seed">Seed</label>
      <input id="seed" type="text" inputmode="numeric" width="240"/>
      <button id="new_seed" title="Random seed"><img src="picture:new_seed"/></button>
    </span>
    <canvas id="preview"/>
    <span nowrap align="right" valign="middle">
      <p id="no_space" class="warning" hidden>No space</p>
      <span id="to">to you</span>
      <button id="give" class="main">Give</button>
    </span>
  </panel>
</form>
"#
);

impl GiveChoice {
    pub(super) const ALL: [Self; 7] = [
        Self::Size,
        Self::Identical,
        Self::Quality,
        Self::Set,
        Self::Favour,
        Self::Magic,
        Self::Kind,
    ];

    /// Its dropdown's name in the markup.
    pub(super) fn ui_name(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::Identical => "identical",
            Self::Quality => "quality",
            Self::Set => "set",
            Self::Favour => "favour",
            Self::Magic => "magic",
            Self::Kind => "kind",
        }
    }

    /// What it offers, in order, and which a newly opened panel has chosen
    /// (as `GiveForm::default`).
    pub(super) fn options(self) -> (Vec<String>, usize) {
        let named = |names: &[&str]| names.iter().map(|name| (*name).to_owned()).collect();
        match self {
            Self::Size => {
                let sizes = Size::ALL.iter().map(|size| size.name().to_owned());
                (
                    std::iter::once("<target>".to_owned())
                        .chain(sizes)
                        .collect(),
                    0,
                )
            }
            Self::Identical => (named(&["Different", "Identical"]), 1),
            Self::Quality => {
                let common = Quality::ALL.iter().position(|&q| q == Quality::default());
                let all = Quality::ALL.iter().map(|quality| format!("{quality:?}"));
                (all.collect(), common.unwrap_or(0))
            }
            Self::Set => (named(&SetChoice::ALL.map(SetChoice::name)), 0),
            Self::Favour => (named(&["Unaligned", "Blessed", "Cursed"]), 0),
            Self::Magic => (named(&["Mundane", "Magical"]), 0),
            Self::Kind => {
                let kinds = give_kinds()
                    .into_iter()
                    .map(|kind| kind.def().name.to_owned());
                (kinds.collect(), 0)
            }
        }
    }
}

/// What a built part of the panel is known by, from its id.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId) {
    let name = id.parts().1;
    if let Some(choice) = GiveChoice::ALL.into_iter().find(|it| it.ui_name() == name) {
        bound.insert(choice);
        return;
    }
    if let Some(field) = FIELDS.into_iter().find(|&field| ui_name(field) == name) {
        let values = Vec::new();
        bound.insert(GiveOption { field, values });
        return;
    }
    match name {
        "less" => bound.insert(GiveCountStep(-1)),
        "more" => bound.insert(GiveCountStep(1)),
        "count" => bound.insert(GiveCount),
        "seed" => bound.insert(GiveSeed),
        "new_seed" => bound.insert(GiveRandomSeed),
        "preview" => bound.insert(GivePreview::default()),
        "no_space" => bound.insert(GiveNoSpace),
        "to" => bound.insert(GiveToWords),
        "give" => bound.insert(GiveButton),
        _ => bound,
    };
}
