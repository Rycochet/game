//! The catalogue's rules for putting beats together hold for many seeds and
//! every goal, and every story renders in every language.

use super::*;
use crate::creature::CreatureKind;
use crate::lang::{Lang, Language};
use crate::rng::Seed;

/// A request asking `goal` with every fact that goal needs, varied by seed.
pub(super) fn request(seed: u64, goal: Goal) -> Request {
    let household = Household::new(CreatureKind::Human, Seed::new(seed ^ 0xabc));
    let mut request = Request::new(Seed::new(seed), household, 0, "Ashford", goal);
    let kinds = PlaceKind::ALL;
    let kind = kinds[(seed % kinds.len() as u64) as usize];
    request.place = Some(Place {
        description: PlaceDescription::new(kind, seed.is_multiple_of(3)),
        name: None,
        direction: Direction::ALL[(seed % 8) as usize],
        distance: DistanceBand::ALL[(seed % 4) as usize],
    });
    request.creatures = Some(Creatures {
        kind: CreatureKind::Kobold,
        count: CountBand::ALL[(seed % 3) as usize],
    });
    let items = ItemCategory::ALL;
    request.item = Some(items[(seed % items.len() as u64) as usize]);
    if goal == Goal::Deliver {
        request.destination = Some(Destination {
            name: "Little Fen".into(),
            kind: CreatureKind::Halfling,
            direction: Direction::East,
            distance: DistanceBand::Day,
        });
    }
    if goal == Goal::Slay && seed.is_multiple_of(2) {
        request.boss = Some(Boss::new(CreatureKind::Kobold, Seed::new(seed)));
    }
    request.reward = vec![RewardPart::Coins(seed * 37 % 20_000 + 5)];
    if seed % 4 == 1 {
        request.higher_reward = true;
        request.season = Some(Season::ALL[(seed / 4 % 4) as usize]);
        request.weather = Some(Weather::ALL[(seed / 4 % 5) as usize]);
        request.reward.push(RewardPart::Item(ItemCategory::Ring));
    }
    request
}

fn all_requests() -> impl Iterator<Item = Request> {
    (0..80u64).flat_map(|seed| Goal::ALL.iter().map(move |&goal| request(seed, goal)))
}

fn assert_rules(request: &Request, telling: &Telling) {
    let story = &telling.story;
    let beats: Vec<Beat> = story.beats().collect();
    assert!(!story.paragraphs.is_empty() && story.paragraphs.len() <= 3);
    assert!(story.paragraphs.iter().all(|p| !p.is_empty()));
    let last = story.paragraphs.last().unwrap();
    assert!(last.iter().any(|b| b.slot() == Slot::Ask), "{beats:?}");
    let asks = beats.iter().filter(|b| b.slot() == Slot::Ask).count();
    assert!((1..=request.asks.len()).contains(&asks), "{beats:?}");
    for (n, beat) in beats.iter().enumerate() {
        assert!(!beats[n + 1..].contains(beat), "twice: {beats:?}");
        assert_ne!(beat.slot(), Slot::Ending);
        let reasons = beat.reasons();
        assert!(
            reasons.is_empty() || reasons.contains(&story.reason),
            "{beat:?}"
        );
        assert!(!beat.is_twist() || request.later_step, "{beat:?}");
    }
    assert_eq!(beats.iter().any(|b| b.is_dark()), request.dark, "{beats:?}");
    assert_eq!(beats.contains(&Beat::Reward), !request.reward.is_empty());
    assert_eq!(story.cast.giver().role, Role::Giver);
    assert_eq!(story.cast.giver().person, request.household.member(0));
    assert_eq!(
        telling.ending.beats.contains(&Beat::TruthFound),
        !telling.ending.lies.is_empty()
    );
    for lie in &telling.ending.lies {
        assert_ne!(Some(lie.truth), story.facts.errand.map(Truth::Errand));
    }
    if let Some(errand) = story.facts.errand {
        let place = story.facts.place.as_ref().unwrap().description.kind;
        assert!(errand.allowed_at(place), "{errand:?} at {place:?}");
    }
}

#[test]
fn stories_keep_the_rules() {
    for mut request in all_requests() {
        assert_rules(&request, &tell(&request));
        request.dark = true;
        assert_rules(&request, &tell(&request));
        request.length = Length::Short;
        let telling = tell(&request);
        assert_rules(&request, &telling);
        assert_eq!(telling.story.paragraphs.len(), 1);
    }
}

#[test]
fn every_story_renders_in_every_language() {
    for language in Language::ALL {
        let lang = Lang::new(language);
        for mut request in all_requests() {
            request.dark = request.text_seed.value().is_multiple_of(5);
            request.continues = true;
            let telling = tell(&request);
            let story = &telling.story;
            let text = story
                .paragraphs_text(&lang)
                .unwrap_or_else(|e| panic!("{e} {story:?}"));
            assert!(text.iter().all(|p| !p.is_empty()));
            story.title(&lang).unwrap();
            story.step_text(&lang, 0, 1).unwrap();
            story.return_text(&lang).unwrap();
            story.ending_text(&lang, &telling.ending).unwrap();
        }
    }
}

#[test]
fn the_same_request_tells_the_same_story_and_it_travels() {
    for request in all_requests().take(50) {
        let telling = tell(&request);
        assert_eq!(tell(&request), telling);
        let bytes = postcard::to_allocvec(&telling.story).unwrap();
        assert_eq!(
            postcard::from_bytes::<Story>(&bytes).unwrap(),
            telling.story
        );
    }
}

#[test]
fn the_cast_holds_only_whom_the_beats_need() {
    for request in all_requests() {
        let story = tell(&request).story;
        let beats: Vec<Beat> = story.beats().collect();
        if story.cast.member(Role::Recipient).is_some() {
            assert!(
                beats
                    .iter()
                    .any(|b| matches!(b, Beat::Take | Beat::DebtOwed | Beat::OrderPending))
            );
        }
        if story.cast.pet.is_some() {
            assert!(
                beats
                    .iter()
                    .any(|b| matches!(b, Beat::PetStrayed | Beat::BringPet))
            );
        }
        if story.cast.boss.is_some() {
            assert!(beats.iter().any(|b| matches!(
                b,
                Beat::BossNamed | Beat::BossWasOneOfUs | Beat::SpareOrSlay
            )));
        }
    }
}

#[test]
fn twists_come_in_later_steps_with_the_cast_carried() {
    let mut twists = 0;
    for request in all_requests() {
        let first = tell(&request).story;
        let mut later = request.clone();
        later.text_seed = request.text_seed.derive("step 2");
        later.reason = Some(first.reason);
        later.later_step = true;
        later.carried = first.carry(&[Role::Giver, Role::Relative, Role::Rival], true, true);
        let telling = tell(&later);
        assert_rules(&later, &telling);
        let story = &telling.story;
        assert_eq!(story.reason, first.reason);
        assert_eq!(story.cast.giver().person, first.cast.giver().person);
        if let (Some(before), Some(after)) = (
            first.cast.member(Role::Relative),
            story.cast.member(Role::Relative),
        ) {
            assert_eq!(before.person, after.person);
            assert!(
                before.state.can_become(after.state),
                "{before:?} -> {after:?}"
            );
        }
        twists += story.beats().filter(|b| b.is_twist()).count();
    }
    assert!(twists > 50, "{twists}");
}

#[test]
fn the_dead_stay_dead() {
    assert!(!State::Dead.can_become(State::Alive));
    assert!(!State::Dead.can_become(State::Missing));
    assert!(State::Missing.can_become(State::Dead));
    assert!(State::Gone.can_become(State::Alive));
    let person = Household::new(CreatureKind::Human, Seed::new(1)).member(1);
    let member = Member {
        role: Role::Relative,
        person,
        relation: Some(Relation::Grandmother),
        nature: Trait::Brave,
        state: State::Dead,
    };
    assert_eq!(member.moved_to(State::Alive), None);
    assert!(member.moved_to(State::Dead).is_some());
}

#[test]
fn relations_suit_genders() {
    for request in all_requests() {
        for member in tell(&request).story.cast.members {
            if let Some(relation) = member.relation {
                assert!(relation.suits(member.person.gender), "{member:?}");
            }
        }
    }
}

#[test]
fn every_reason_and_many_beats_turn_up() {
    let mut reasons = std::collections::BTreeSet::new();
    let mut beats = std::collections::BTreeSet::new();
    for mut request in all_requests() {
        request.dark = request.text_seed.value().is_multiple_of(4);
        let story = tell(&request).story;
        reasons.insert(story.reason);
        beats.extend(story.beats());
    }
    assert_eq!(reasons.len(), Reason::ALL.len(), "{reasons:?}");
    let drawn = Beat::ALL
        .iter()
        .filter(|b| !b.is_twist() && b.slot() != Slot::Ending);
    let missing: Vec<_> = drawn.filter(|b| !beats.contains(b)).collect();
    assert!(missing.len() <= 4, "never told: {missing:?}");
}
