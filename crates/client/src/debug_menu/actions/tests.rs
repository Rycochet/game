use super::*;

#[test]
fn entries_are_found_by_name_in_any_case() {
    assert_eq!(DebugAction::named("God Mode"), Some(DebugAction::GodMode));
    assert_eq!(
        DebugAction::named(" teleport to start "),
        Some(DebugAction::TeleportToStart)
    );
    assert_eq!(DebugAction::named("Godmode"), None);
    assert_eq!(DebugAction::names().count(), DebugAction::ALL.len());
}

#[test]
fn the_user_documentation_lists_every_entry() {
    let page = include_str!("../../../../../docs/debug-menu.md");
    for action in DebugAction::ALL {
        let name = action.name();
        assert!(
            page.contains(&format!("| {name} |")),
            "{name} is missing from docs/debug-menu.md"
        );
    }
}

fn facts(view: GameView) -> Facts {
    Facts {
        view,
        has_stairs: false,
        has_fields: false,
        has_herbs: false,
        walking: false,
        detecting: false,
        god_mode: false,
        truesight: false,
        killing: false,
        teleporting: false,
        changing_cells: false,
        showing_info: true,
        in_town: false,
        world_time_open: false,
        suns_and_moons_open: true,
        dev_tools: true,
    }
}

#[test]
fn entries_with_submenus_end_in_dots_and_only_those() {
    for action in DebugAction::ALL {
        assert_eq!(action.label().ends_with("..."), action.has_submenu());
        assert!(!action.name().ends_with("..."));
    }
    assert_eq!(DebugAction::GeneratePoi.label(), "Generate POI...");
}

#[test]
fn the_kinds_list_says_their_names_with_spaces() {
    assert_eq!(kind_name(PoiKind::StoneTower), "Stone Tower");
    assert_eq!(kind_name(PoiKind::Castle), "Castle");
}

#[test]
fn the_list_is_alphabetical() {
    let names: Vec<_> = DebugAction::ALL.iter().map(|a| a.name()).collect();
    let mut sorted = names.clone();
    sorted.sort_by_key(|name| name.to_lowercase());
    assert_eq!(names, sorted);
}

#[test]
fn the_filter_ignores_case_and_matches_anywhere() {
    assert!(DebugAction::TeleportMode.matches("PORT"));
    assert!(DebugAction::TeleportMode.matches(""));
    assert!(!DebugAction::TeleportMode.matches("stairs"));
}

#[test]
fn site_options_need_a_site_and_what_they_change() {
    let world = Facts {
        has_stairs: true,
        ..facts(GameView::WorldMap)
    };
    assert!(!DebugAction::StairsStyle.is_available(&world));
    assert!(DebugAction::AutoWalk.is_available(&world));

    let site = facts(GameView::Dungeon);
    assert!(!DebugAction::StairsStyle.is_available(&site));
    assert!(!DebugAction::FieldsState.is_available(&site));
    assert!(!DebugAction::HerbsState.is_available(&site));
    assert!(!DebugAction::AutoWalk.is_available(&site));
    let stairs = Facts {
        has_stairs: true,
        ..site
    };
    assert!(DebugAction::StairsStyle.is_available(&stairs));
}

#[test]
fn reveal_needs_a_site_and_neither_map_option_works_in_a_town() {
    assert!(!DebugAction::RevealMap.is_available(&facts(GameView::WorldMap)));
    assert!(DebugAction::ForgetMap.is_available(&facts(GameView::WorldMap)));
    assert!(DebugAction::RevealMap.is_available(&facts(GameView::Dungeon)));
    let town = Facts {
        in_town: true,
        ..facts(GameView::Dungeon)
    };
    assert!(!DebugAction::RevealMap.is_available(&town));
    assert!(!DebugAction::ForgetMap.is_available(&town));
    // A town's generator can change like any other.
    assert!(DebugAction::ForgetMapOnServer.is_available(&town));
}

#[test]
fn what_the_server_must_allow_is_disabled_when_it_does_not() {
    let refused = Facts {
        dev_tools: false,
        ..facts(GameView::Dungeon)
    };
    assert!(!DebugAction::TeleportMode.is_available(&refused));
    assert!(!DebugAction::TeleportToStart.is_available(&refused));
    assert!(!DebugAction::ForgetMapOnServer.is_available(&refused));
    assert!(!DebugAction::RevealMap.is_available(&refused));
    assert!(DebugAction::ResetView.is_available(&refused));
    assert!(DebugAction::ShowDebugInfo.is_available(&refused));
}

#[test]
fn only_toggles_have_a_state() {
    let site = facts(GameView::Dungeon);
    assert_eq!(DebugAction::ShowDebugInfo.is_on(&site), Some(true));
    assert_eq!(DebugAction::TeleportMode.is_on(&site), Some(false));
    assert_eq!(DebugAction::ResetView.is_on(&site), None);
    // Panels that open and close show which.
    assert_eq!(DebugAction::SunsAndMoons.is_on(&site), Some(true));
    assert_eq!(DebugAction::WorldTime.is_on(&site), Some(false));
    assert_eq!(DebugAction::FieldsState.is_on(&site), None);
}
