//! `Tree`: the draft built into nodes, once, with filled-in children and lookups.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use super::SkillId;
use super::branches::TOP;
use super::draft::{Draft, Filled};
use crate::blade::weapon_names;
use crate::creature::CreatureKind;
use crate::sheet::Attribute;

/// One node of the built tree.
pub(super) struct Node {
    /// Its part of a path: lower case, no spaces.
    pub key: String,
    pub name: &'static str,
    /// From the root down, joined by `/`: `combat/melee/longsword/longsword`.
    pub path: String,
    pub parent: Option<SkillId>,
    pub children: Vec<SkillId>,
    /// Its own, or else its parent's.
    pub leans: &'static [Attribute],
    /// 1 for a top level.
    pub depth: u8,
}

/// Every node, parents before their children, and how to find one by its path.
pub(super) struct Tree {
    pub nodes: Vec<Node>,
    pub tops: Vec<SkillId>,
    pub by_path: BTreeMap<String, SkillId>,
}

/// The tree, built the first time it is needed.
pub(super) static TREE: LazyLock<Tree> = LazyLock::new(Tree::build);

/// A name as a key: lower case, without spaces or hyphens ("Battle Axe": `battleaxe`).
pub(super) fn key_of(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

impl Tree {
    fn build() -> Self {
        let mut tree = Self {
            nodes: Vec::new(),
            tops: Vec::new(),
            by_path: BTreeMap::new(),
        };
        for top in TOP {
            let id = tree.add_draft(top, None);
            tree.tops.push(id);
        }
        tree
    }

    /// Adds a node and returns its id. Leaves `children` to the caller.
    fn add(
        &mut self,
        key: String,
        name: &'static str,
        leans: &'static [Attribute],
        parent: Option<SkillId>,
    ) -> SkillId {
        let id = SkillId::from_index(self.nodes.len());
        let (path, depth, inherited) = match parent {
            Some(parent) => {
                let up = &self.nodes[parent.index()];
                (format!("{}/{key}", up.path), up.depth + 1, up.leans)
            }
            None => (key.clone(), 1, &[][..]),
        };
        let leans = if leans.is_empty() { inherited } else { leans };
        self.by_path.insert(path.clone(), id);
        self.nodes.push(Node {
            key,
            name,
            path,
            parent,
            children: Vec::new(),
            leans,
            depth,
        });
        if let Some(parent) = parent {
            self.nodes[parent.index()].children.push(id);
        }
        id
    }

    fn add_draft(&mut self, draft: &Draft, parent: Option<SkillId>) -> SkillId {
        let id = self.add(draft.key.to_string(), draft.name, draft.leans, parent);
        for child in draft.children {
            self.add_draft(child, Some(id));
        }
        match draft.filled {
            Filled::Nothing => {}
            Filled::Weapons(family) => {
                for name in weapon_names(family) {
                    self.add(key_of(name), name, &[], Some(id));
                }
            }
            Filled::Kinds => {
                for kind in CreatureKind::ALL {
                    let name = kind.def().name;
                    self.add(key_of(name), name, &[], Some(id));
                }
            }
        }
        id
    }
}
