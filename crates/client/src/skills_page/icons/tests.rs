use shared::skill::SkillId;

use super::{ICONS, icon_path};

/// Every path in the table is a skill, and every skill has an icon that has been
/// drawn into the assets (`cargo xtask icons`).
#[test]
fn every_skill_has_a_drawn_icon() {
    for (path, _) in ICONS {
        assert!(SkillId::find(path).is_some(), "no skill {path}");
    }
    for id in SkillId::all() {
        let icon = icon_path(id).unwrap_or_else(|| panic!("{} has no icon", id.path()));
        let file = format!("{}/assets/{icon}", env!("CARGO_MANIFEST_DIR"));
        assert!(std::path::Path::new(&file).exists(), "{file}");
    }
}

#[test]
fn a_skill_without_its_own_takes_the_nearest_above() {
    let scimitar = SkillId::find("combat/melee/sword/scimitar").unwrap();
    assert_eq!(icon_path(scimitar).unwrap(), "icons/lorc/broadsword.png");
    let katana = SkillId::find("combat/melee/sword/katana").unwrap();
    assert_eq!(icon_path(katana).unwrap(), "icons/delapouite/katana.png");
}
