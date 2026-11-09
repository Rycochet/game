use shared::protocol::{ClientMessage, DevSkills, ServerMessage};
use shared::skill::{SkillId, Skills};

use super::tests::TestClient;

/// The skills the server last told this client of its own.
fn told(answers: &[ServerMessage]) -> Skills {
    answers
        .iter()
        .rev()
        .find_map(|message| match message {
            ServerMessage::DevSkillsOf { who: None, skills } => Some(skills.clone()),
            _ => None,
        })
        .expect("told the skills")
}

fn change(test: &mut TestClient, change: DevSkills) -> Skills {
    told(&test.send(&[ClientMessage::DevSkills { who: None, change }]))
}

#[test]
fn dev_tools_set_learn_forget_and_clear_skills() {
    let mut test = TestClient::new();
    let leaf = SkillId::all()
        .find(|id| id.children().is_empty() && id.depth() > 1)
        .expect("a leaf");
    let skills = change(
        &mut test,
        DevSkills::Set {
            skill: leaf,
            percent: 40,
        },
    );
    assert_eq!(skills.percent(leaf), 40);
    let skills = change(&mut test, DevSkills::Forget(leaf));
    assert!(!skills.knows(leaf));
    let skills = change(&mut test, DevSkills::Learn(leaf));
    assert!(skills.knows(leaf.parent().expect("a parent")));
    let skills = change(&mut test, DevSkills::SetAll(7));
    assert!(SkillId::all().all(|id| skills.percent(id) == 7));
    assert!(change(&mut test, DevSkills::Clear).is_empty());
    assert!(change(&mut test, DevSkills::Ask).is_empty());
}
