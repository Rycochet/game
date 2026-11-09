use super::*;

fn first_leaf() -> SkillId {
    SkillId::all()
        .find(|id| id.children().is_empty() && id.depth() > 1)
        .expect("a leaf under a top level")
}

#[test]
fn a_percentage_is_set_on_that_node_alone() {
    let mut skills = Skills::default();
    let leaf = first_leaf();
    skills.set_percent(leaf, 50);
    assert_eq!(skills.percent(leaf), 50);
    assert_eq!(skills.practice(leaf), 125_000);
    let parent = leaf.parent().expect("a parent");
    assert!(!skills.knows(parent));
    skills.set_percent(leaf, 100);
    assert_eq!(skills.practice(leaf), 1_000_000);
    skills.set_percent(leaf, 0);
    assert!(!skills.knows(leaf));
}

#[test]
fn learning_flows_up_and_forgetting_takes_what_is_under() {
    let mut skills = Skills::default();
    let leaf = first_leaf();
    skills.learn(leaf);
    assert_eq!(skills.percent(leaf), DEV_LEARN_PERCENT);
    let top = leaf.upwards().last().expect("a top level");
    assert!(skills.knows(top));
    skills.forget(top);
    assert!(skills.is_empty());
}

#[test]
fn everything_can_be_set_and_cleared() {
    let mut skills = Skills::default();
    skills.set_all(20);
    assert!(SkillId::all().all(|id| skills.percent(id) == 20));
    skills.clear();
    assert!(skills.is_empty());
    assert_eq!(practice_for(5000), practice_for(1625));
}
