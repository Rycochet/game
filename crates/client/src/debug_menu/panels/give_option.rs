//! Give Item's options for an item's form (`RollRequest`): the values each can take,
//! their words, and reading and setting them in a request.

use shared::armour::{Construction, Dye, Finish, Style};
use shared::item_form::{Choices, Cut, Field, Limits, RollRequest};
use shared::material::Material;

/// Every option, in the order the panel shows them.
pub(super) const FIELDS: [Field; 9] = [
    Field::Style,
    Field::Construction,
    Field::Material,
    Field::Finish,
    Field::Dye,
    Field::Settings,
    Field::GemMaterial,
    Field::GemCut,
    Field::Limits,
];

/// The longest a blade can be asked to be at least, in cells.
const LONGEST: u8 = 8;

/// A value one of the options can take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::debug_menu) enum OptionValue {
    Style(Style),
    Construction(Construction),
    Material(Material),
    Finish(Finish),
    Dye(Dye),
    Settings(u8),
    Gem(Material),
    Cut(Cut),
    /// A blade at least this many cells long.
    Long(u8),
}

/// What the dropdown says when nothing is asked for.
/// A field's dropdown's name in Give Item's form.
pub(super) fn ui_name(field: Field) -> &'static str {
    match field {
        Field::Style => "style",
        Field::Construction => "construction",
        Field::Material => "material",
        Field::Finish => "finish",
        Field::Dye => "dye",
        Field::Settings => "settings",
        Field::GemMaterial => "gem_material",
        Field::GemCut => "gem_cut",
        Field::Limits => "limits",
    }
}

pub(super) fn any(field: Field) -> &'static str {
    match field {
        Field::Style => "Any style",
        Field::Construction => "Any construction",
        Field::Material => "Any material",
        Field::Finish => "Any finish",
        Field::Dye => "Any colour",
        Field::Settings => "Any settings",
        Field::GemMaterial => "Any gem",
        Field::GemCut => "Any cut",
        Field::Limits => "Any length",
    }
}

/// The values this option can take, as `choices` found them.
pub(super) fn values(field: Field, choices: &Choices) -> Vec<OptionValue> {
    use OptionValue as V;
    match field {
        Field::Style => choices.styles.iter().map(|&v| V::Style(v)).collect(),
        Field::Construction => choices
            .constructions
            .iter()
            .map(|&v| V::Construction(v))
            .collect(),
        Field::Material => choices.materials.iter().map(|&v| V::Material(v)).collect(),
        Field::Finish => choices.finishes.iter().map(|&v| V::Finish(v)).collect(),
        Field::Dye => choices.dyes.iter().map(|&v| V::Dye(v)).collect(),
        Field::Settings => choices.settings.iter().map(|&v| V::Settings(v)).collect(),
        Field::GemMaterial => choices.gem_materials.iter().map(|&v| V::Gem(v)).collect(),
        Field::GemCut => choices.gem_cuts.iter().map(|&v| V::Cut(v)).collect(),
        Field::Limits if choices.limits => (1..=LONGEST).map(V::Long).collect(),
        Field::Limits => Vec::new(),
    }
}

/// How a value reads in its dropdown.
pub(super) fn words(value: OptionValue) -> String {
    match value {
        OptionValue::Style(style) => style.name().to_owned(),
        OptionValue::Construction(construction) => format!("{construction:?}"),
        OptionValue::Material(material) | OptionValue::Gem(material) => material.name().to_owned(),
        OptionValue::Finish(Finish::Painted(dye)) => format!("Painted {}", dye.name()),
        OptionValue::Finish(Finish::Lacquered(dye)) => format!("Lacquered {}", dye.name()),
        OptionValue::Finish(finish) => format!("{finish:?}"),
        OptionValue::Dye(dye) => dye.name().to_owned(),
        OptionValue::Settings(1) => "1 setting".to_owned(),
        OptionValue::Settings(count) => format!("{count} settings"),
        OptionValue::Cut(cut) => cut.name().to_owned(),
        OptionValue::Long(cells) => format!("{cells}+ cells long"),
    }
}

/// What a request asks of this option.
pub(super) fn asked(request: &RollRequest, field: Field) -> Option<OptionValue> {
    use OptionValue as V;
    match field {
        Field::Style => request.style.map(V::Style),
        Field::Construction => request.construction.map(V::Construction),
        Field::Material => request.material.map(V::Material),
        Field::Finish => request.finish.map(V::Finish),
        Field::Dye => request.dye.map(V::Dye),
        Field::Settings => request.settings.map(V::Settings),
        Field::GemMaterial => request.gem_material.map(V::Gem),
        Field::GemCut => request.gem_cut.map(V::Cut),
        Field::Limits => request
            .limits
            .and_then(|limits| limits.at_least_long.map(V::Long)),
    }
}

/// The request asking for this value of its option.
pub(super) fn asking(request: RollRequest, value: OptionValue) -> RollRequest {
    match value {
        OptionValue::Style(style) => RollRequest {
            style: Some(style),
            ..request
        },
        OptionValue::Construction(c) => RollRequest {
            construction: Some(c),
            ..request
        },
        OptionValue::Material(m) => RollRequest {
            material: Some(m),
            ..request
        },
        OptionValue::Finish(finish) => RollRequest {
            finish: Some(finish),
            ..request
        },
        OptionValue::Dye(dye) => RollRequest {
            dye: Some(dye),
            ..request
        },
        OptionValue::Settings(count) => RollRequest {
            settings: Some(count),
            ..request
        },
        OptionValue::Gem(m) => RollRequest {
            gem_material: Some(m),
            ..request
        },
        OptionValue::Cut(cut) => RollRequest {
            gem_cut: Some(cut),
            ..request
        },
        OptionValue::Long(cells) => {
            let limits = Limits {
                at_least_long: Some(cells),
                ..Limits::NONE
            };
            RollRequest {
                limits: Some(limits),
                ..request
            }
        }
    }
}
