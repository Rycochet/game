use super::*;

#[test]
fn it_says_the_chosen_skills_path_values_and_attributes() {
    let mut skills = Skills::default();
    let climbing = SkillId::find("athletics/climbing").unwrap();
    skills.practise(climbing, 1000);
    assert_eq!(
        describe(&skills, Some(climbing)),
        "Athletics › Climbing: 10.0 (17.9 in checks), from 1000 practice. \
         Leans on Strength and Agility."
    );
    assert!(describe(&skills, None).starts_with("Point at or choose"));
    assert!(describe(&Skills::default(), None).starts_with("Nothing learned yet"));
}
