//! What the debug menu offers, and when each is possible. No Bevy systems here,
//! so it can be tested alone.

use crate::common::GameView;

/// One entry in the debug menu. Sending it does what it says.
#[derive(bevy::prelude::Message, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DebugAction {
    AutoWalk,
    BackToStart,
    ChangeCellMode,
    CycleTemperature,
    FieldsState,
    ForgetMap,
    HerbsState,
    PoiShowcase,
    ResetView,
    RevealMap,
    ShowDebugInfo,
    ShowWidgets,
    StairsStyle,
    SunsAndMoons,
    TeleportMode,
    WorldTime,
}

/// The state of the game that the menu depends on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Facts {
    pub view: GameView,
    pub has_stairs: bool,
    pub has_fields: bool,
    pub has_herbs: bool,
    pub walking: bool,
    pub cycling: bool,
    pub teleporting: bool,
    pub changing_cells: bool,
    pub showing_info: bool,
    pub in_town: bool,
    pub world_time_open: bool,
    pub suns_and_moons_open: bool,
    /// Whether the server accepts developer tools messages.
    pub dev_tools: bool,
}

impl DebugAction {
    /// Every action, in the order the menu lists them: alphabetical by name.
    pub(super) const ALL: [Self; 16] = [
        Self::AutoWalk,
        Self::BackToStart,
        Self::ChangeCellMode,
        Self::CycleTemperature,
        Self::FieldsState,
        Self::ForgetMap,
        Self::HerbsState,
        Self::PoiShowcase,
        Self::ResetView,
        Self::RevealMap,
        Self::ShowDebugInfo,
        Self::ShowWidgets,
        Self::StairsStyle,
        Self::SunsAndMoons,
        Self::TeleportMode,
        Self::WorldTime,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::AutoWalk => "Auto Walk",
            Self::BackToStart => "Back to Start",
            Self::ChangeCellMode => "Change Cell Mode",
            Self::CycleTemperature => "Cycle Temperature",
            Self::FieldsState => "Fields State",
            Self::ForgetMap => "Forget Map",
            Self::HerbsState => "Herbs State",
            Self::PoiShowcase => "Poi Showcase",
            Self::ResetView => "Reset View",
            Self::RevealMap => "Reveal Map",
            Self::ShowDebugInfo => "Show Debug Info",
            Self::ShowWidgets => "Show Widgets",
            Self::StairsStyle => "Stairs Style",
            Self::SunsAndMoons => "Suns and Moons",
            Self::TeleportMode => "Teleport Mode",
            Self::WorldTime => "World Time",
        }
    }

    /// Whether it can be used now.
    pub(super) fn is_available(self, facts: &Facts) -> bool {
        if self.needs_server_dev_tools() && !facts.dev_tools {
            return false;
        }
        match self {
            // In a site it would stop at a wall.
            Self::AutoWalk => facts.view == GameView::WorldMap,
            Self::ChangeCellMode => facts.view == GameView::Dungeon,
            // A town is known from the start, and would only come straight back.
            Self::ForgetMap => !facts.in_town,
            Self::RevealMap => facts.view == GameView::Dungeon && !facts.in_town,
            Self::FieldsState => facts.view == GameView::Dungeon && facts.has_fields,
            Self::HerbsState => facts.view == GameView::Dungeon && facts.has_herbs,
            Self::StairsStyle => facts.view == GameView::Dungeon && facts.has_stairs,
            Self::PoiShowcase => shared::showcase::ENABLED,
            _ => true,
        }
    }

    /// Whether it does nothing unless the server accepts dev tools.
    fn needs_server_dev_tools(self) -> bool {
        matches!(
            self,
            Self::BackToStart
                | Self::ChangeCellMode
                | Self::CycleTemperature
                | Self::ForgetMap
                | Self::PoiShowcase
                | Self::RevealMap
                | Self::TeleportMode
                | Self::WorldTime
        )
    }

    /// For one that toggles, whether it is on.
    pub(super) fn is_on(self, facts: &Facts) -> Option<bool> {
        match self {
            Self::AutoWalk => Some(facts.walking),
            Self::ChangeCellMode => Some(facts.changing_cells),
            Self::CycleTemperature => Some(facts.cycling),
            Self::ShowDebugInfo => Some(facts.showing_info),
            Self::SunsAndMoons => Some(facts.suns_and_moons_open),
            Self::WorldTime => Some(facts.world_time_open),
            Self::TeleportMode => Some(facts.teleporting),
            _ => None,
        }
    }

    /// Whether its name matches what has been typed: anywhere in it, whatever
    /// the case.
    pub(super) fn matches(self, filter: &str) -> bool {
        self.name().to_lowercase().contains(&filter.to_lowercase())
    }
}

/// How many entries fit in a column, for the arrow keys and the layout.
pub(super) fn next_highlight(
    highlight: Option<usize>,
    count: usize,
    rows: usize,
    step: Step,
) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let last = count - 1;
    Some(match (highlight, step) {
        (None, Step::Up) => last,
        (None, _) => 0,
        (Some(at), Step::Up) => at.saturating_sub(1),
        (Some(at), Step::Down) => (at + 1).min(last),
        (Some(at), Step::Left) => at.checked_sub(rows).unwrap_or(at),
        (Some(at), Step::Right) => {
            if at + rows <= last {
                at + rows
            } else {
                at
            }
        }
    })
}

/// A press of an arrow key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Step {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(view: GameView) -> Facts {
        Facts {
            view,
            has_stairs: false,
            has_fields: false,
            has_herbs: false,
            walking: false,
            cycling: false,
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
    }

    #[test]
    fn what_the_server_must_allow_is_disabled_when_it_does_not() {
        let refused = Facts {
            dev_tools: false,
            ..facts(GameView::Dungeon)
        };
        assert!(!DebugAction::TeleportMode.is_available(&refused));
        assert!(!DebugAction::BackToStart.is_available(&refused));
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

    #[test]
    fn arrows_move_through_the_columns_and_stop_at_the_ends() {
        let go = |from, step| next_highlight(from, 10, 4, step);
        assert_eq!(go(None, Step::Down), Some(0));
        assert_eq!(go(None, Step::Up), Some(9));
        assert_eq!(go(Some(3), Step::Down), Some(4));
        assert_eq!(go(Some(9), Step::Down), Some(9));
        assert_eq!(go(Some(0), Step::Up), Some(0));
        assert_eq!(go(Some(1), Step::Right), Some(5));
        assert_eq!(go(Some(5), Step::Left), Some(1));
        assert_eq!(go(Some(1), Step::Left), Some(1));
        // No such row in the next column: stays.
        assert_eq!(go(Some(7), Step::Right), Some(7));
        assert_eq!(next_highlight(None, 0, 4, Step::Down), None);
    }
}
