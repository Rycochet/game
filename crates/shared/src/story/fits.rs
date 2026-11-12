//! Whether a beat can be told in a story being made: its roles can be filled
//! and its checkable facts hold in the world (the catalogue's "needs before
//! beats"). What choosing it changes is in `apply.rs`.

use super::cast::Role;
use super::draft::Draft;
use super::facts::{Goal, ItemCategory, Relation, State, TimeAgo};
use super::{Beat, PlaceKind};

impl Draft<'_> {
    pub(super) fn has_goal(&self, goal: Goal) -> bool {
        self.request.asks.iter().any(|ask| ask.goal == goal)
    }

    pub(super) fn place_kind(&self) -> Option<PlaceKind> {
        self.request.place.as_ref().map(|p| p.description.kind)
    }

    fn keepsake(&self) -> bool {
        self.request.item.is_some_and(ItemCategory::is_keepsake)
    }

    /// The state the relative goes to by going somewhere and not returning.
    pub(super) fn went_state(&self) -> State {
        let relative = self.member(Role::Relative);
        if self.time_ago == TimeAgo::LongAgo || relative.state == State::Dead {
            State::Dead
        } else {
            State::Missing
        }
    }

    fn relative_can_become(&self, state: State) -> bool {
        self.member(Role::Relative).state.can_become(state)
    }

    pub(super) fn relative_used(&self) -> bool {
        self.used.roles.contains(&Role::Relative)
    }

    fn relative_is(&self, state: State) -> bool {
        self.relative_used() && self.member(Role::Relative).state == state
    }

    fn pet_lost(&self) -> bool {
        self.used.pet && self.pet.state == State::Missing
    }

    /// Whether the beat may be told here, not counting reasons, darkness or
    /// twists (`plot` checks those).
    pub(super) fn fits(&self, beat: Beat) -> bool {
        use Beat::*;
        let request = self.request;
        let place = self.place_kind();
        let at_place = place.is_some();
        let creatures = request.creatures.is_some();
        let later = request.later_step;
        match beat {
            RelativeWent => self.errand.is_some() && self.relative_can_become(self.went_state()),
            HeirloomLost => at_place && self.keepsake() && self.has_goal(Goal::Find),
            DrivenOff => {
                creatures
                    && place.is_some_and(|p| {
                        use PlaceKind::*;
                        matches!(p, Farm | Dwelling | Settlement | Store)
                    })
            }
            PetStrayed => at_place && self.pet.state.can_become(State::Missing),
            RumourHeard | GiverWasThere => at_place,
            NeighbourWrong => true,
            Accused => self.used.roles.contains(&Role::Rival),
            DebtOwed => request.destination.is_some(),
            OrderPending => {
                self.has_goal(Goal::Gather)
                    || request.item.is_some_and(|item| {
                        use ItemCategory::*;
                        matches!(item, Goods | Herb | Tool | Remedy | Book)
                    })
            }
            ThreatSeen => creatures,
            DiedThere => self.errand.is_some() && self.relative_can_become(State::Dead),
            DyingWish | Murdered => self.relative_can_become(State::Dead),
            PlaceHistory => place.is_some_and(|p| p != PlaceKind::Wilds),
            BossNamed => self.boss.is_some(),
            SickAtHome => {
                (self.has_goal(Goal::Gather)
                    || matches!(
                        request.item,
                        Some(ItemCategory::Remedy | ItemCategory::Herb)
                    ))
                    && self.relative_can_become(State::Sick)
            }
            Eloped => {
                self.relative_can_become(State::Gone)
                    && !(self.member(Role::Relative).relation).is_some_and(Relation::is_elder)
            }
            StudyWanted => at_place || request.item.is_some(),
            FirstTryFailed => at_place && self.member(Role::Giver).state.can_become(State::Hurt),
            WrongPlaceSearched => at_place,
            CreaturesMovedIn => {
                creatures
                    && [
                        RelativeWent,
                        HeirloomLost,
                        DiedThere,
                        DrivenOff,
                        PlaceHistory,
                    ]
                    .iter()
                    .any(|b| self.has(*b))
            }
            GiverCannot | Livelihood => true,
            TimePressure | ForOccasion => true,
            KeepItQuiet => self.used.roles.contains(&Role::Rival),
            NobodyBelieves => self.has(RumourHeard) || self.has(ThreatSeen),
            RivalSeeksIt => at_place || request.item.is_some(),
            FateUnknown => self.relative_is(State::Missing),
            CannotPay => true,
            Guilt => self.pet_lost(),
            WeatherBars => request.higher_reward && at_place && request.weather.is_some(),
            NotGuilty => later && request.carried.member(Role::Rival).is_some(),
            LeftOnPurpose => {
                later
                    && (request.carried.member(Role::Relative))
                        .is_some_and(|m| m.state == State::Missing)
            }
            BossWasOneOfUs => later && self.boss.is_some(),
            WhoToHelp => later,
            KeepOrReturn => later && request.item.is_some(),
            SpareOrSlay => later && self.has_goal(Goal::Slay) && self.boss.is_some(),
            Memory => {
                self.relative_is(State::Dead) || self.relative_is(State::Gone) || self.keepsake()
            }
            Safety | Warning => creatures,
            Honour => self.used.roles.contains(&Role::Rival),
            ChildMisses => true,
            Promise => self.has(DyingWish),
            Reward => !request.reward.is_empty(),
            SeasonTurns => request.higher_reward && request.season.is_some(),
            Revenge => true,
            // Asks and the ending are chosen by `plot`, not drawn.
            _ => false,
        }
    }
}
