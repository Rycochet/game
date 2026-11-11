//! The Learn Skills panel (owner's design, 2026-11-09): every skill of the tree
//! as a checkbox (learn or forget it) with its value in a box beside it, for
//! the character or another player targeted, and buttons for six skills at
//! random, forgetting them all, or setting them all to one value. It takes the
//! place of Learn Some Skills and Forget Skills.

use bevy::prelude::*;
use shared::id::Id;
use shared::skill::{SkillId, Skills};

use crate::dom::{Dom, Template};
use crate::html;
use crate::widgets::UiId;

/// The panel's form, for the debug API.
pub(super) const FORM: &str = "debug.learn_skills";
/// How far each level of the tree is set in, in pixels.
const INDENT: u32 = 16;

/// Whose skills the panel shows and changes, and what it was last told of them.
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct LearnSkillsForm {
    /// The targeted player's, rather than the character's.
    pub(super) for_target: bool,
    /// Whose skills were last asked for (`None`: the character's), while open.
    pub(super) asked: Option<Option<Id>>,
    /// What the server said they are.
    pub(super) skills: Option<Skills>,
}

/// The panel's buttons, other than the skills'.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(in crate::debug_menu) enum LearnButton {
    You,
    Target,
    Random,
    Clear,
    SetAll,
}

/// A skill's checkbox: learns or forgets it.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct SkillCheck(pub(super) SkillId);

/// A skill's value, as a whole percentage, set when the box is left.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct SkillBox(pub(super) SkillId);

/// The value Set All gives every skill.
#[derive(Component)]
pub(in crate::debug_menu) struct SetAllBox;

/// The panel's contents, as markup. Each skill's row (its value's box, then
/// its checkbox) is a copy of the template, numbered by the skill's place in
/// `SkillId::all`.
pub(super) static LEARN_SKILLS: Template = html!(
    "debug_menu.learn_skills",
    r#"
<form id="debug.learn_skills">
  <span nowrap valign="middle">
    For
    <button id="you">You</button>
    <button id="target">Target</button>
  </span>
  <span nowrap valign="middle">
    <button id="random">Random</button>
    <button id="clear">Clear Skills</button>
    <input id="all" type="text" inputmode="numeric" width="56"/>
    %
    <button id="set_all">Set All</button>
  </span>
  <scroll id="scroll" max-height="60%">
    <div id="skills" class="tight"/>
  </scroll>
  <template id="skill">
    <span id="row" nowrap valign="middle">
      <input id="value" type="text" inputmode="numeric" width="56"/>
      <small><input id="skill" type="checkbox" label="Skill" readonly disabled width="240"/></small>
    </span>
  </template>
</form>
"#
);

/// What a built part of the panel is known by, from its id; the list is
/// filled with a row for each skill, set in by its depth in the tree.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId, dom: &mut Dom) {
    let skill = id.index().and_then(|at| SkillId::all().nth(at));
    match (id.parts().1, skill) {
        ("you", _) => bound.insert(LearnButton::You),
        ("target", _) => bound.insert(LearnButton::Target),
        ("random", _) => bound.insert(LearnButton::Random),
        ("clear", _) => bound.insert(LearnButton::Clear),
        ("set_all", _) => bound.insert(LearnButton::SetAll),
        ("all", _) => bound.insert(SetAllBox),
        ("value", Some(skill)) => bound.insert(SkillBox(skill)),
        ("skill", Some(skill)) => bound.insert(SkillCheck(skill)),
        ("skills", _) => {
            for (at, skill) in SkillId::all().enumerate() {
                dom.copy_template("debug.learn_skills/skill", "debug.learn_skills/skills", at);
                dom.set_text(&format!("{FORM}/skill.{at}"), skill.name());
                let indent = INDENT * u32::from(skill.depth().saturating_sub(1));
                dom.set(
                    &format!("{FORM}/row.{at}"),
                    "margin-left",
                    &indent.to_string(),
                );
            }
            bound
        }
        _ => bound,
    };
}
