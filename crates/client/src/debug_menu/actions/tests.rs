use super::*;

#[test]
fn entries_are_found_by_name_in_any_case() {
    assert_eq!(DebugAction::named("God Mode"), Some(DebugAction::GodMode));
    assert_eq!(
        DebugAction::named(" teleport to world "),
        Some(DebugAction::TeleportToWorld)
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
        detecting: false,
        god_mode: false,
        truesight: false,
        killing: false,
        inspecting: false,
        deleting: false,
        freezing: false,
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
fn entries_opening_something_end_in_dots_and_only_those() {
    for action in DebugAction::ALL {
        assert_eq!(action.label().ends_with("..."), action.opens_something());
        assert!(!action.name().ends_with("..."));
    }
    assert_eq!(DebugAction::GeneratePoi.label(), "Generate POI...");
    assert_eq!(DebugAction::TeleportToWorld.label(), "Teleport to World...");
    assert_eq!(DebugAction::GodMode.label(), "God Mode");
}

#[test]
fn the_kinds_list_says_their_names_with_spaces() {
    assert_eq!(kind_name(PoiKind::StoneTower), "Stone Tower");
    assert_eq!(kind_name(PoiKind::Castle), "Castle");
}

#[test]
fn the_list_keeps_each_group_together_in_order() {
    let groups: Vec<Group> = DebugAction::ALL.iter().map(|a| a.group()).collect();
    let mut sorted = groups.clone();
    sorted.sort();
    assert_eq!(groups, sorted);
    let mut seen = std::collections::HashSet::new();
    assert!(DebugAction::ALL.iter().all(|&action| seen.insert(action)));
}

#[test]
fn the_filter_ignores_case_and_matches_anywhere() {
    assert!(DebugAction::TeleportMode.matches("PORT"));
    assert!(DebugAction::TeleportMode.matches(""));
    assert!(!DebugAction::TeleportMode.matches("stairs"));
}

#[test]
fn level_tools_need_a_site() {
    assert!(!DebugAction::OpenAllDoors.is_available(&facts(GameView::WorldMap)));
    assert!(DebugAction::OpenAllDoors.is_available(&facts(GameView::Dungeon)));
    assert!(!DebugAction::ChangeCellMode.is_available(&facts(GameView::WorldMap)));
    // Drawing States stays open with its rows greyed instead.
    assert!(DebugAction::DrawingStates.is_available(&facts(GameView::WorldMap)));
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
    assert!(DebugAction::RemakeSite.is_available(&town));
}

#[test]
fn what_the_server_must_allow_is_disabled_when_it_does_not() {
    let refused = Facts {
        dev_tools: false,
        ..facts(GameView::Dungeon)
    };
    assert!(!DebugAction::TeleportMode.is_available(&refused));
    assert!(!DebugAction::TeleportToWorld.is_available(&refused));
    assert!(!DebugAction::RemakeSite.is_available(&refused));
    assert!(!DebugAction::RevealMap.is_available(&refused));
    assert!(DebugAction::ItemPictures.is_available(&refused));
    assert!(DebugAction::ShowDebugInfo.is_available(&refused));
}

#[test]
fn only_toggles_have_a_state() {
    let site = facts(GameView::Dungeon);
    assert_eq!(DebugAction::ShowDebugInfo.is_on(&site), Some(true));
    assert_eq!(DebugAction::TeleportMode.is_on(&site), Some(false));
    assert_eq!(DebugAction::Vitals.is_on(&site), None);
    // Panels that open and close show which.
    assert_eq!(DebugAction::SunsAndMoons.is_on(&site), Some(true));
    assert_eq!(DebugAction::WorldTime.is_on(&site), Some(false));
    assert_eq!(DebugAction::DrawingStates.is_on(&site), None);
}
