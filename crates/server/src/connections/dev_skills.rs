//! Dev tools for skills: practising some at random, or forgetting them.

use shared::protocol::{DEV_SKILLS_LEARNT, DevSkills, ServerMessage};
use shared::rng::Seed;
use shared::skill::{SkillId, Skills};

use super::Client;

/// The least and most practice `DevSkills::LearnSome` gives a node.
const PRACTICE: (u64, u64) = (100, 2500);

impl Client {
    /// Dev tools: practises some skills or forgets them, and sends the sheet. `now`
    /// (the action clock) makes each try pick differently. Whether the client is
    /// still welcome.
    pub(super) fn dev_skills(&mut self, what: DevSkills, now: u64) -> bool {
        match what {
            DevSkills::LearnSome => {
                let ends: Vec<_> = SkillId::all()
                    .filter(|id| id.children().is_empty())
                    .collect();
                let mut rng = Seed::new(now).derive("dev skills").rng();
                for _ in 0..DEV_SKILLS_LEARNT {
                    let end = ends[rng.range_u64(0, ends.len() as u64) as usize];
                    let amount = rng.range_u64(PRACTICE.0, PRACTICE.1) as u32;
                    self.sheet.skills.practise(end, amount);
                }
            }
            DevSkills::Forget => {
                let kind = self.sheet.own_kind(self.inventory.creature);
                self.sheet.skills = Skills::starting(kind);
            }
        }
        self.unsaved = true;
        self.link.send(&ServerMessage::Sheet(self.sheet.clone()))
    }
}
