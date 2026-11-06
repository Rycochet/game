//! `Draft`: how the tree is written in `branches/`, one node and what is under it.

use crate::blade::Family;
use crate::sheet::Attribute;

/// A node as written: its key in paths, its name, the attributes it leans on (none to
/// lean on its parent's), and what is under it.
pub(super) struct Draft {
    pub key: &'static str,
    pub name: &'static str,
    pub leans: &'static [Attribute],
    pub children: &'static [Draft],
    pub filled: Filled,
}

/// Children filled in from other tables, after any written ones.
#[derive(Clone, Copy)]
pub(super) enum Filled {
    Nothing,
    /// Every name a weapon of this family may go by (`blade::weapon_names`).
    Weapons(Family),
    /// Every kind of creature, for fighting with its own body.
    Kinds,
}

/// A node with children written out.
pub(super) const fn branch(
    key: &'static str,
    name: &'static str,
    leans: &'static [Attribute],
    children: &'static [Draft],
) -> Draft {
    Draft {
        key,
        name,
        leans,
        children,
        filled: Filled::Nothing,
    }
}

/// A node with nothing under it.
pub(super) const fn leaf(
    key: &'static str,
    name: &'static str,
    leans: &'static [Attribute],
) -> Draft {
    branch(key, name, leans, &[])
}

/// A node whose children come from another table.
pub(super) const fn filled(
    key: &'static str,
    name: &'static str,
    leans: &'static [Attribute],
    filled: Filled,
) -> Draft {
    Draft {
        key,
        name,
        leans,
        children: &[],
        filled,
    }
}
