//! The Dev Area panel (owner's design): size, layout, who is in it and how many,
//! and a seed of any words, then Generate, which goes straight in.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::site::{DEV_AREA_SIZES, DevArea, DevLayout, DevMobs, MOST_DEV_MOBS};

use crate::dom::Template;
use crate::html;
use crate::widgets::UiId;

/// What the Dev Area panel is set to, besides its seed box. Kept while the
/// game runs, so the panel opens as it was left.
#[derive(Resource, Clone, Debug)]
pub(in crate::debug_menu) struct DevAreaForm {
    pub(super) size: u16,
    pub(super) layout: DevLayout,
    /// `None` for Default, `Some(None)` for None, else the kind.
    pub(super) mobs: Option<Option<CreatureKind>>,
    pub(super) count: u8,
    /// The seed box's words, kept for the next time the panel opens.
    pub(super) seed: String,
    pub(super) seeds_made: i64,
}

impl Default for DevAreaForm {
    fn default() -> Self {
        Self {
            size: DEV_AREA_SIZES[1],
            layout: DevLayout::Empty,
            mobs: None,
            count: 3,
            seed: String::new(),
            seeds_made: 0,
        }
    }
}

impl DevAreaForm {
    /// What to ask the server to make, as the panel says.
    pub(super) fn area(&self) -> DevArea {
        let mobs = match self.mobs {
            None => DevMobs::Default,
            Some(None) => DevMobs::None,
            Some(Some(kind)) => DevMobs::Kind {
                kind,
                count: self.count,
            },
        };
        DevArea {
            size: self.size,
            layout: self.layout,
            mobs,
            seed: DevArea::seed_from_text(&self.seed),
        }
        .checked()
    }

    /// Whether a count goes with the mobs chosen.
    pub(super) fn has_count(&self) -> bool {
        matches!(self.mobs, Some(Some(_)))
    }

    /// The mobs dropdown's choice: Default, None, then every kind.
    pub(super) fn mobs_index(&self) -> usize {
        match self.mobs {
            None => 0,
            Some(None) => 1,
            Some(Some(kind)) => {
                2 + CreatureKind::ALL
                    .iter()
                    .position(|&k| k == kind)
                    .unwrap_or(0)
            }
        }
    }

    pub(super) fn choose_mobs(&mut self, index: usize) {
        self.mobs = match index {
            0 => None,
            1 => Some(None),
            _ => Some(CreatureKind::ALL.get(index - 2).copied()),
        };
    }

    /// One more or fewer, kept to what can be made.
    pub(super) fn step_count(&mut self, step: i8) {
        self.count = self
            .count
            .saturating_add_signed(step)
            .clamp(1, MOST_DEV_MOBS);
    }
}

/// Which of the panel's dropdowns it is.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum DevAreaChoice {
    Size,
    Layout,
    Mobs,
}

#[derive(Component)]
pub(in crate::debug_menu) struct DevAreaCount;

#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct DevAreaCountStep(pub(super) i8);

#[derive(Component)]
pub(in crate::debug_menu) struct DevAreaSeed;

#[derive(Component)]
pub(in crate::debug_menu) struct DevAreaNewSeed;

#[derive(Component)]
pub(in crate::debug_menu) struct DevAreaGenerate;

/// The panel's contents, as markup; `start_dev_area_panel` gives the
/// dropdowns their choices and sets it all to the form kept, with a seed if
/// there is none.
pub(super) static DEV_AREA: Template = html!(
    "debug_menu.dev_area",
    r#"
<form id="debug.dev_area">
  <panel id="panel" moveable>
    <span nowrap valign="middle">
      <label for="size">Size</label>
      <select id="size" width="130"><option>32 × 32</option></select>
      <label for="generator">Generator</label>
      <select id="generator" searchable width="200"><option>Empty</option></select>
    </span>
    <span nowrap valign="middle">
      <label for="mobs">Mobs</label>
      <select id="mobs" searchable width="200"><option>Default</option></select>
      <button id="less">-</button>
      <input id="count" type="text" inputmode="numeric" width="64"/>
      <button id="more">+</button>
    </span>
    <span nowrap valign="middle">
      <label for="seed">Seed</label>
      <input id="seed" type="text" width="240"/>
      <button id="new_seed" title="Random seed"><img src="picture:new_seed"/></button>
    </span>
    <span nowrap align="right">
      <button id="generate" class="main">Generate</button>
    </span>
  </panel>
</form>
"#
);

impl DevAreaChoice {
    pub(super) const ALL: [Self; 3] = [Self::Size, Self::Layout, Self::Mobs];

    /// Its dropdown's name in the markup.
    pub(super) fn ui_name(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::Layout => "generator",
            Self::Mobs => "mobs",
        }
    }

    /// What it offers, in order.
    pub(super) fn options(self) -> Vec<String> {
        match self {
            Self::Size => DEV_AREA_SIZES
                .iter()
                .map(|size| format!("{size} × {size}"))
                .collect(),
            Self::Layout => DevLayout::all()
                .into_iter()
                .map(|layout| layout.name())
                .collect(),
            Self::Mobs => {
                let kinds = CreatureKind::ALL.iter().map(|kind| kind.def().name);
                ["Default", "None"]
                    .into_iter()
                    .chain(kinds)
                    .map(str::to_owned)
                    .collect()
            }
        }
    }
}

/// What a built part of the panel is known by, from its id.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId) {
    let name = id.parts().1;
    if let Some(choice) = DevAreaChoice::ALL
        .into_iter()
        .find(|it| it.ui_name() == name)
    {
        bound.insert(choice);
        return;
    }
    match name {
        "less" => bound.insert(DevAreaCountStep(-1)),
        "more" => bound.insert(DevAreaCountStep(1)),
        "count" => bound.insert(DevAreaCount),
        "seed" => bound.insert(DevAreaSeed),
        "new_seed" => bound.insert(DevAreaNewSeed),
        "generate" => bound.insert(DevAreaGenerate),
        _ => bound,
    };
}
