//! The panel's lines for skills, until the Skills page shows them as a tree.

use shared::skill::{SkillId, Skills, VALUE_SCALE};

/// A line for each known skill with no known skill under it, by its path, with what
/// a check would count: "Athletics › Climbing 17.1".
pub(super) fn skill_lines(skills: &Skills) -> Vec<String> {
    skills
        .known()
        .map(|(id, _)| id)
        .filter(|id| !id.children().iter().any(|&child| skills.knows(child)))
        .map(|id| {
            let mut names: Vec<_> = id.upwards().map(SkillId::name).collect();
            names.reverse();
            let total = skills.total(id);
            let tenths = total * 10 / VALUE_SCALE;
            format!("{} {}.{}", names.join(" › "), tenths / 10, tenths % 10)
        })
        .collect()
}
