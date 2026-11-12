//! Dev tools for skills (the debug menu's Learn Skills): asking what they are,
//! practising some at random, learning, forgetting and setting them.

use shared::id::Id;
use shared::protocol::{DEV_SKILLS_LEARNT, DevSkills, ServerMessage};
use shared::rng::Seed;
use shared::skill::SkillId;

use super::Client;
use super::for_others::{ForOther, ask};

/// The least and most practice `DevSkills::LearnSome` gives a node: 10% to 50%.
const PRACTICE: (u64, u64) = (1_000, 125_000);

impl Client {
    /// Dev tools: does `change` to the character's skills, or asks for it done
    /// to `who`, another player, once messages are read (`for_others`). `now`
    /// (the action clock) makes each try pick differently. Whether the client
    /// is still welcome.
    pub(super) fn dev_skills(&mut self, who: Option<Id>, change: DevSkills, now: u64) -> bool {
        if let Some(other) = who.filter(|&id| id != self.entity_id()) {
            ask(self, other, ForOther::Skills(change));
            return true;
        }
        let changed = self.change_skills(change, now);
        let sheet = !changed || self.link.send(&ServerMessage::Sheet(self.sheet.as_sent()));
        sheet
            && self.link.send(&ServerMessage::DevSkillsOf {
                who: None,
                skills: self.sheet.skills.clone(),
            })
    }

    /// Does `change` to this character's skills: whether anything may have changed.
    pub(super) fn change_skills(&mut self, change: DevSkills, now: u64) -> bool {
        let skills = &mut self.sheet.skills;
        match change {
            DevSkills::Ask => return false,
            DevSkills::LearnSome => {
                let ends: Vec<_> = SkillId::all()
                    .filter(|id| id.children().is_empty())
                    .collect();
                let mut rng = Seed::new(now).derive("dev skills").rng();
                for _ in 0..DEV_SKILLS_LEARNT {
                    let end = ends[rng.range_u64(0, ends.len() as u64) as usize];
                    let amount = rng.range_u64(PRACTICE.0, PRACTICE.1) as u32;
                    skills.practise(end, amount);
                    skills.used(end, self.activity.played);
                }
            }
            DevSkills::Clear => skills.clear(),
            DevSkills::Learn(skill) => skills.learn(skill),
            DevSkills::Forget(skill) => skills.forget(skill),
            DevSkills::Set { skill, percent } => skills.set_percent(skill, percent),
            DevSkills::SetAll(percent) => skills.set_all(percent),
        }
        self.unsaved = true;
        true
    }
}
