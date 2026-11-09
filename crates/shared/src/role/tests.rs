use super::*;

#[test]
fn ids_are_short_and_plain() {
    assert!(RoleId::new("game-master_2").is_valid());
    assert!(!RoleId::new("").is_valid());
    assert!(!RoleId::new("Admin").is_valid());
    assert!(!RoleId::new("a role").is_valid());
    assert!(!RoleId::new(&"a".repeat(25)).is_valid());
    assert!(BUILT_IN.iter().all(|id| RoleId::new(id).is_valid()));
}

#[test]
fn a_title_is_shown_in_place_of_the_name() {
    let mut badge = RoleBadge {
        id: RoleId::new(ADMIN),
        name: "Admin".into(),
        title: None,
    };
    assert_eq!(badge.shown(), "Admin");
    badge.title = Some("Archmage".into());
    assert_eq!(badge.shown(), "Archmage");
}

#[test]
fn staff_are_marked_and_plain_players_arent() {
    for id in [OWNER, ADMIN, DEVELOPER, MODERATOR, "game-master"] {
        assert!(is_marked(&RoleId::new(id)), "{id}");
    }
    for id in [PLAYER, GUEST, MODERATED, BANNED] {
        assert!(!is_marked(&RoleId::new(id)), "{id}");
    }
}
