use super::*;

#[test]
fn it_says_the_chosen_skills_path_values_and_attributes() {
    let mut skills = Skills::default();
    let climbing = SkillId::find("athletics/climbing").unwrap();
    skills.practise(climbing, 100);
    assert_eq!(
        describe(&skills, Some(climbing)),
        "Athletics › Climbing: 17.0 in checks (10.0 of its own), from 100 practice. \
         Leans on Strength and Agility."
    );
    assert!(describe(&skills, None).starts_with("Choose a skill"));
    assert!(describe(&Skills::default(), None).starts_with("Nothing learned yet"));
}
