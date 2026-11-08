//! `SkillId`: one node of the tree, and what can be asked of it.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::rust::Rust;
use super::tree::TREE;
use crate::sheet::Attribute;

/// One node of the skill tree. Its number is its place in the built tree, so it is
/// only sent between a client and server of the same version, never saved: saves
/// keep its `path`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SkillId(u16);

impl SkillId {
    pub(super) fn from_index(index: usize) -> Self {
        Self(u16::try_from(index).expect("fewer than 65536 skills"))
    }

    pub(super) fn index(self) -> usize {
        self.0 as usize
    }

    /// The one at this number, if there is one.
    pub fn from_number(number: u16) -> Option<Self> {
        ((number as usize) < TREE.nodes.len()).then_some(Self(number))
    }

    pub fn number(self) -> u16 {
        self.0
    }

    /// Every node, parents before their children, branch by branch.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..TREE.nodes.len()).map(Self::from_index)
    }

    /// Combat, Magic, Athletics, ..., Lore, in that order.
    pub fn top_levels() -> &'static [Self] {
        &TREE.tops
    }

    /// The node at this path (`athletics/climbing`), if there is one.
    pub fn find(path: &str) -> Option<Self> {
        TREE.by_path.get(path).copied()
    }

    /// As shown: "Longsword".
    pub fn name(self) -> &'static str {
        TREE.nodes[self.index()].name
    }

    /// Its own part of the path: `longsword`.
    pub fn key(self) -> &'static str {
        &TREE.nodes[self.index()].key
    }

    /// From the top level down, joined by `/`: `combat/melee/longsword/longsword`.
    pub fn path(self) -> &'static str {
        &TREE.nodes[self.index()].path
    }

    /// The node it is under; none for a top level.
    pub fn parent(self) -> Option<Self> {
        TREE.nodes[self.index()].parent
    }

    /// The nodes directly under it, in order.
    pub fn children(self) -> &'static [Self] {
        &TREE.nodes[self.index()].children
    }

    /// 1 for a top level, 2 under it, and so on.
    pub fn depth(self) -> u8 {
        TREE.nodes[self.index()].depth
    }

    /// The attributes it leans on: its own, or its parent's if it names none.
    pub fn leans(self) -> &'static [Attribute] {
        TREE.nodes[self.index()].leans
    }

    /// How it rusts: its own, or as the nearest node above it that says.
    pub fn rust(self) -> Rust {
        TREE.nodes[self.index()].rust
    }

    /// Itself, then each node above it up to its top level.
    pub fn upwards(self) -> impl Iterator<Item = Self> {
        std::iter::successors(Some(self), |id| id.parent())
    }
}

impl Serialize for SkillId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.0)
    }
}

/// Only a number in the tree: anything else is refused, never looked up.
impl<'de> Deserialize<'de> for SkillId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let number = u16::deserialize(deserializer)?;
        Self::from_number(number)
            .ok_or_else(|| serde::de::Error::custom(format!("no skill numbered {number}")))
    }
}
