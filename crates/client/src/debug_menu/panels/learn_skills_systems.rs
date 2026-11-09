//! What Learn Skills does: asking the server for whose skills it shows, sending
//! each change, and showing what it was told.

use bevy::prelude::*;
use shared::id::Id;
use shared::protocol::{ClientMessage, DevSkills, ServerMessage};
use shared::sheet::Feeling;

use super::super::actions::DebugAction;
use super::OpenPanels;
use super::learn_skills::{LearnButton, LearnSkillsForm, SetAllBox, SkillBox, SkillCheck};
use crate::common::{Other, ServerDevTools, Target};
use crate::connection::{FromServer, ToServer};
use crate::widgets::{ButtonPressed, ButtonState, Checked, TextBox, TextFocus};

/// Adds the panel's systems to the debug menu's plugin.
pub(in crate::debug_menu) fn add_learn_skills_panel(app: &mut App) {
    app.init_resource::<LearnSkillsForm>().add_systems(
        Update,
        (
            ask_skills,
            take_skills,
            press_learn_buttons,
            edit_skill_boxes,
            show_skills,
        )
            .chain(),
    );
}

/// The targeted player, if one is in sight (mobs have no skills yet; other
/// players feel nothing).
fn targeted_player(target: &Target, others: &Query<&Other>) -> Option<Id> {
    let id = target.0?;
    let other = others.iter().find(|other| other.id == id)?;
    (other.feeling == Feeling::None).then_some(id)
}

/// Whose skills the panel is for now: `None` the character's; `Some(None)`
/// for the target with no player targeted.
fn whose(form: &LearnSkillsForm, target: &Target, others: &Query<&Other>) -> Option<Option<Id>> {
    if form.for_target {
        targeted_player(target, others).map(Some)
    } else {
        Some(None)
    }
}

/// Asks for the skills shown whenever the panel opens or whose they are changes.
fn ask_skills(
    open: Res<OpenPanels>,
    target: Res<Target>,
    others: Query<&Other>,
    mut form: ResMut<LearnSkillsForm>,
    mut to_server: MessageWriter<ToServer>,
) {
    if !open.is_open(DebugAction::LearnSkills) {
        if form.asked.is_some() {
            form.asked = None;
            form.skills = None;
        }
        return;
    }
    let who = whose(&form, &target, &others);
    if form.asked == who && form.asked.is_some() {
        return;
    }
    form.asked = who;
    form.skills = None;
    if let Some(who) = who {
        let change = DevSkills::Ask;
        to_server.write(ToServer(ClientMessage::DevSkills { who, change }));
    }
}

/// What the server says the skills are, for whoever the panel shows.
fn take_skills(mut from_server: MessageReader<FromServer>, mut form: ResMut<LearnSkillsForm>) {
    for FromServer(message) in from_server.read() {
        if let ServerMessage::DevSkillsOf { who, skills } = message
            && form.asked == Some(*who)
        {
            form.skills = Some(skills.clone());
        }
    }
}

/// Whose skills to show, and the changes the buttons and checkboxes make.
fn press_learn_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&LearnButton>,
    checks: Query<&SkillCheck>,
    set_all: Query<&TextBox, With<SetAllBox>>,
    mut form: ResMut<LearnSkillsForm>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let change = if let Ok(&check) = checks.get(press.0) {
            let known = form.skills.as_ref().is_some_and(|s| s.knows(check.0));
            if known {
                DevSkills::Forget(check.0)
            } else {
                DevSkills::Learn(check.0)
            }
        } else {
            match buttons.get(press.0) {
                Ok(LearnButton::You) => {
                    form.for_target = false;
                    continue;
                }
                Ok(LearnButton::Target) => {
                    form.for_target = true;
                    continue;
                }
                Ok(LearnButton::Random) => DevSkills::LearnSome,
                Ok(LearnButton::Clear) => DevSkills::Clear,
                Ok(LearnButton::SetAll) => {
                    let percent = set_all
                        .single()
                        .ok()
                        .and_then(|b| b.text.trim().parse().ok());
                    let Some(percent) = percent else {
                        continue;
                    };
                    DevSkills::SetAll(percent)
                }
                Err(_) => continue,
            }
        };
        if let Some(who) = form.asked {
            to_server.write(ToServer(ClientMessage::DevSkills { who, change }));
        }
    }
}

/// A skill's box, left with a number in it, sets that skill to it: only a known
/// skill's, as the box of one not known only shows it isn't (owner).
fn edit_skill_boxes(
    focus: Res<TextFocus>,
    boxes: Query<(Entity, &TextBox, &SkillBox)>,
    form: Res<LearnSkillsForm>,
    mut had_focus: Local<Option<Entity>>,
    mut to_server: MessageWriter<ToServer>,
) {
    if *had_focus != focus.0
        && let Some((_, text_box, &SkillBox(skill))) =
            had_focus.and_then(|left| boxes.get(left).ok())
        && let (Ok(percent), Some(who)) = (text_box.text.trim().parse(), form.asked)
        && form.skills.as_ref().is_some_and(|s| s.knows(skill))
    {
        let change = DevSkills::Set { skill, percent };
        to_server.write(ToServer(ClientMessage::DevSkills { who, change }));
    }
    *had_focus = focus.0;
}

/// Ticks what is known and writes each value in its box (but the one being
/// typed in), greying it all while there is nothing to show or change.
#[allow(clippy::too_many_arguments)]
fn show_skills(
    form: Res<LearnSkillsForm>,
    dev_tools: Res<ServerDevTools>,
    focus: Res<TextFocus>,
    target: Res<Target>,
    others: Query<&Other>,
    mut checks: Query<(&SkillCheck, &mut Checked, &mut ButtonState)>,
    mut boxes: Query<(Entity, &SkillBox, &mut TextBox)>,
    mut buttons: Query<(&LearnButton, &mut ButtonState), Without<SkillCheck>>,
) {
    let can = dev_tools.0 && form.skills.is_some();
    for (button, mut state) in &mut buttons {
        let (usable, chosen) = match button {
            LearnButton::You => (true, !form.for_target),
            LearnButton::Target => (targeted_player(&target, &others).is_some(), form.for_target),
            _ => (can, false),
        };
        state.set_if_neq(match (usable, chosen) {
            (true, true) => ButtonState::Selected,
            (true, false) => ButtonState::Normal,
            (false, true) => ButtonState::DisabledSelected,
            (false, false) => ButtonState::Disabled,
        });
    }
    if !form.is_changed() && !dev_tools.is_changed() {
        return;
    }
    let skills = form.skills.as_ref();
    for (&SkillCheck(skill), mut checked, mut state) in &mut checks {
        let known = skills.is_some_and(|s| s.knows(skill));
        checked.set_if_neq(Checked(known));
        state.set_if_neq(if can {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        });
    }
    for (entity, &SkillBox(skill), mut text_box) in &mut boxes {
        if focus.0 == Some(entity) {
            continue;
        }
        let shown = skills
            .filter(|s| s.knows(skill))
            .map_or(String::new(), |s| s.percent(skill).to_string());
        if text_box.text != shown {
            text_box.text = shown;
        }
    }
}
