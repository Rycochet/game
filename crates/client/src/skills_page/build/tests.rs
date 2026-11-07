use shared::skill::{SkillId, Skills};

use super::Drawn;

fn find(path: &str) -> SkillId {
    SkillId::find(path).unwrap()
}

#[test]
fn the_tree_is_drawn_again_only_when_a_skill_is_learned() {
    let mut skills = Skills::default();
    skills.practise(find("athletics/climbing"), 100);
    let mut drawn = Drawn::default();
    assert!(drawn.changed_to(&skills), "drawn the first time");
    assert!(!drawn.changed_to(&skills), "nothing changed");
    skills.practise(find("athletics/climbing"), 50);
    assert!(!drawn.changed_to(&skills), "practice alone keeps the tree");
    skills.practise(find("lore/history"), 10);
    assert!(drawn.changed_to(&skills), "a new skill draws it again");
    assert!(!drawn.changed_to(&skills));
}
