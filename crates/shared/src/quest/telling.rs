//! A plan into a story: the `shared::story` request with the quest's facts
//! as the giver's town sees them, carried on from the chain's earlier quest.

use super::QuestWorld;
use super::plan::Plan;
use crate::creature::CreatureKind;
use crate::poi::Poi;
use crate::rng::Seed;
use crate::site::Generator;
use crate::story::{
    CountBand, Creatures, Destination, Direction, DistanceBand, Goal, Household, Length, Place,
    PlaceDescription, PlaceKind, Request, RewardPart, Role, Story, Telling, tell,
};

/// Everything about the telling but the plan.
pub(super) struct Tale<'a> {
    pub household: Household,
    pub member: u32,
    /// The story's own seed, sent to clients.
    pub text_seed: Seed,
    pub reward: Vec<RewardPart>,
    pub higher: bool,
    /// The chain's quest before this one, whose cast and reason it keeps.
    pub earlier: Option<&'a Story>,
    /// Only for characters with a bad name (alignment).
    pub dark: bool,
    /// The chain goes on after it.
    pub continues: bool,
    pub daily: bool,
}

/// The story of `plan`, given by `town`'s household.
pub(super) fn tell_plan(world: &QuestWorld, town: &Poi, plan: &Plan, tale: Tale) -> Telling {
    let home = (world.name_of)(town).unwrap_or_default();
    let goal = plan.steps.first().map_or(Goal::Visit, |step| step.goal);
    let mut request = Request::new(tale.text_seed, tale.household, tale.member, home, goal);
    request.asks = plan.asks();
    let width = world.pois.width();
    let from = (town.x, town.y);
    request.place = plan.place.map(|poi| {
        let wilds = poi.kind.is_town();
        let (kind, ruined) = if wilds {
            (PlaceKind::Wilds, false)
        } else {
            // Not a town, so no roads' sides.
            let ruin = Generator::for_site(poi.kind, poi.seed, poi.desert, [false; 4]).ruin();
            (PlaceKind::of_poi(poi.kind), ruin > 0)
        };
        Place {
            description: PlaceDescription::new(kind, ruined),
            name: (!wilds).then(|| (world.name_of)(&poi)).flatten(),
            direction: Direction::between(from, (poi.x, poi.y), width),
            distance: DistanceBand::between(from, (poi.x, poi.y), width),
        }
    });
    if plan
        .dungeon
        .is_some_and(|d| d.way == super::DungeonWay::Cellar)
    {
        // Under the giver's own house.
        request.place = Some(Place {
            description: PlaceDescription::new(PlaceKind::Cellar, false),
            name: None,
            direction: Direction::between(from, from, width),
            distance: DistanceBand::Near,
        });
    }
    request.creatures = plan.creatures.map(|(kind, count)| Creatures {
        kind,
        count: CountBand::of(count),
    });
    request.boss = plan.boss.clone();
    request.item = plan.item;
    request.remains = plan.remains;
    request.pet = plan.pet.clone();
    request.destination = plan.destination.map(|poi| Destination {
        name: (world.name_of)(&poi).unwrap_or_default(),
        kind: CreatureKind::Human,
        direction: Direction::between(from, (poi.x, poi.y), width),
        distance: DistanceBand::between(from, (poi.x, poi.y), width),
    });
    request.reward = tale.reward;
    request.higher_reward = tale.higher;
    request.dark = tale.dark;
    request.continues = tale.continues;
    if tale.daily {
        request.length = Length::Short;
    }
    if let Some(earlier) = tale.earlier {
        request.reason = Some(earlier.reason);
        request.later_step = true;
        let roles = [Role::Giver, Role::Relative, Role::Rival];
        request.carried = earlier.carry(&roles, plan.pet.is_none(), true);
    }
    tell(&request)
}
