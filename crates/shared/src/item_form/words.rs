//! Words for a ring's name: its gems counted and in the plural.

use super::ItemForm;
use crate::item::Item;
use crate::material::Material;

pub(super) fn gem_material(item: &Item) -> Option<Material> {
    match &item.form {
        Some(ItemForm::Gem(gem)) => Some(gem.material),
        _ => None,
    }
}

pub(super) fn counted(count: usize, one: &str, many: String) -> String {
    let a = if one.starts_with(['A', 'E']) {
        "an"
    } else {
        "a"
    };
    match count {
        1 => format!("{a} {one}"),
        2 => format!("Two {many}"),
        3 => format!("Three {many}"),
        4 => format!("Four {many}"),
        5 => format!("Five {many}"),
        _ => format!("{count} {many}"),
    }
}

pub(super) fn plural(stone: Material) -> String {
    match stone {
        Material::Ruby => "Rubies".to_owned(),
        Material::Topaz => "Topazes".to_owned(),
        _ => format!("{}s", stone.name()),
    }
}
