use super::*;
use crate::blade::Family;
use crate::creature::CreatureKind;
use crate::sheet::Sheet;

#[test]
fn the_tree_is_strict_and_every_path_is_its_own() {
    let all: Vec<_> = SkillId::all().collect();
    assert!(all.len() > 100, "{} nodes", all.len());
    let mut paths = std::collections::BTreeSet::new();
    for id in &all {
        assert!(paths.insert(id.path()), "{} twice", id.path());
        assert_eq!(SkillId::find(id.path()), Some(*id));
        assert!(!id.key().is_empty() && !id.key().contains(['/', ' ']));
        assert!(!id.leans().is_empty(), "{} leans on nothing", id.path());
        match id.parent() {
            Some(parent) => {
                assert!(parent < *id, "parents come first");
                assert_eq!(id.depth(), parent.depth() + 1);
                assert!(parent.children().contains(id));
                assert_eq!(id.path(), format!("{}/{}", parent.path(), id.key()));
            }
            None => assert!(SkillId::top_levels().contains(id)),
        }
        assert!(id.depth() <= 4, "{} is too deep", id.path());
    }
}

#[test]
fn the_top_levels_are_the_nine_branches() {
    let names: Vec<_> = SkillId::top_levels().iter().map(|id| id.name()).collect();
    assert_eq!(
        names,
        [
            "Combat",
            "Magic",
            "Athletics",
            "Stealth",
            "Survival",
            "Gathering",
            "Crafting",
            "Social",
            "Lore"
        ]
    );
}

#[test]
fn melee_is_weapon_type_then_weapon_and_unarmed_is_by_kind() {
    for path in [
        "combat/melee/longsword/longsword",
        "combat/melee/shortsword/khopesh",
        "combat/melee/axe/battleaxe",
        "combat/melee/unarmed/human",
        "combat/melee/unarmed/scorpionman",
    ] {
        assert!(SkillId::find(path).is_some(), "{path}");
    }
    let unarmed = SkillId::find("combat/melee/unarmed").unwrap();
    assert_eq!(unarmed.children().len(), CreatureKind::ALL.len());
    for family in Family::ALL {
        assert!(!crate::blade::weapon_names(family).is_empty(), "{family:?}");
    }
}

#[test]
fn practice_flows_up_halving() {
    let sword = SkillId::find("combat/melee/longsword/longsword").unwrap();
    let mut skills = Skills::default();
    skills.practise(sword, 1000);
    let practice: Vec<_> = sword.upwards().map(|id| skills.practice(id)).collect();
    assert_eq!(practice, [1000, 500, 250, 125]);
    assert_eq!(skills.value(sword), 3162);
    // 31.62 + 22.36 + 15.81 + 11.18, as in the plan's worked example.
    assert_eq!(skills.total(sword), 3162 + 2236 + 1581 + 1118);
    let other = SkillId::find("combat/melee/longsword/claymore").unwrap();
    assert!(!skills.knows(other));
    assert_eq!(skills.total(other), 2236 + 1581 + 1118);
}

#[test]
fn every_kinds_starting_skills_are_in_the_tree() {
    for kind in CreatureKind::ALL {
        for (path, _) in kind.def().skills {
            assert!(SkillId::find(path).is_some(), "{kind:?}: {path}");
        }
    }
    let wolf = Skills::starting(CreatureKind::Wolf);
    let running = SkillId::find("athletics/running").unwrap();
    assert_eq!(wolf.practice(running), level_practice(4));
}

#[test]
fn saves_keep_paths_and_messages_numbers() {
    let mut skills = Skills::default();
    skills.practise(SkillId::find("magic/arcane/fire").unwrap(), 40);
    let written = ron::to_string(&skills).unwrap();
    assert!(written.contains("(\"magic/arcane/fire\",40)"), "{written}");
    assert_eq!(ron::from_str::<Skills>(&written).unwrap(), skills);
    let bytes = postcard::to_allocvec(&skills).unwrap();
    assert_eq!(postcard::from_bytes::<Skills>(&bytes).unwrap(), skills);
}

#[test]
fn an_old_save_is_read_by_the_old_names() {
    let old: Skills = ron::from_str("{Climbing: 1, Stealth: 3, Gone: 2}").unwrap();
    let climbing = SkillId::find("athletics/climbing").unwrap();
    let sneaking = SkillId::find("stealth/sneaking").unwrap();
    assert_eq!(old.practice(climbing), level_practice(1));
    assert_eq!(old.practice(sneaking), level_practice(3));
    assert_eq!(
        old.known().count(),
        4,
        "climbing, sneaking and their top levels"
    );
    let dropped: Skills = ron::from_str("[(\"combat/gone\", 5)]").unwrap();
    assert!(dropped.is_empty());
}

#[test]
fn a_sheet_starts_with_its_kinds_skills() {
    let sheet = Sheet::new(CreatureKind::Halfling);
    assert_eq!(sheet.skills, Skills::starting(CreatureKind::Halfling));
    assert!(!sheet.skills.is_empty());
}
