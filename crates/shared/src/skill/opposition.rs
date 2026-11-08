//! Opposition: one-way links between nodes, using one wearing the other down
//! (`rust.rs`). A link hits only the node it names, never what is under it, so
//! combat wearing down magic takes from Magic's own share of a spell's check and
//! leaves each spell alone (owner, 2026-11-08). Using anything under a link's
//! source counts as using the source.

use std::sync::LazyLock;

use super::SkillId;

/// Each link as the paths it goes from and to. A link between a node and its own
/// ancestor or descendant is refused by a test: practice flows up, so it could
/// never fire, or would wear down the thing being used.
pub(super) const LINKS: &[(&str, &str)] = &[
    ("combat", "magic"),
    ("combat/melee", "combat/ranged"),
    ("combat/ranged", "combat/melee"),
];

/// The links as nodes, found once.
static FOUND: LazyLock<Vec<(SkillId, SkillId)>> = LazyLock::new(|| {
    LINKS
        .iter()
        .filter_map(|&(from, to)| Some((SkillId::find(from)?, SkillId::find(to)?)))
        .collect()
});

/// The nodes using this one wears down, each once.
pub(super) fn opposed_by(id: SkillId) -> Vec<SkillId> {
    let mut targets = Vec::new();
    for &(from, to) in FOUND.iter() {
        let used = id.upwards().any(|node| node == from);
        let own = id.upwards().any(|node| node == to);
        if used && !own && !targets.contains(&to) {
            targets.push(to);
        }
    }
    targets
}
