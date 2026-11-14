use super::test_world::{TestWorld, buildings};
use super::*;
use crate::lang::{Lang, Language};
use crate::site::{CAVE_VERSION, Site};
use crate::story::PlaceKind;

/// Every quest dungeon among the test world's towns' quests and a week of
/// dailies.
fn dungeons() -> Vec<(crate::poi::Poi, Quest)> {
    let world = TestWorld::new();
    let mut found = Vec::new();
    for town in world.towns(24) {
        let looks = buildings(&town);
        let mut quests = town_quests(&world.view(), &town, &looks);
        for building in daily_givers(&town, &looks) {
            quests.extend((0..7).filter_map(|day| {
                daily_quest(&world.view(), &town, &looks, building, 20_000 + day)
            }));
        }
        found.extend(
            (quests.into_iter())
                .filter(|quest| quest.dungeon.is_some())
                .map(|quest| (town, quest)),
        );
    }
    found
}

#[test]
fn some_quests_are_cellars_and_they_can_be_done() {
    let lang = Lang::new(Language::English);
    let cellars: Vec<_> = (dungeons().into_iter())
        .filter(|(_, q)| q.dungeon.unwrap().way == DungeonWay::Cellar)
        .collect();
    assert!(cellars.len() >= 5, "{}", cellars.len());
    assert!(cellars.iter().any(|(_, q)| q.kind == QuestKind::Daily));
    assert!(cellars.iter().any(|(_, q)| q.kind == QuestKind::Normal));
    for (town, quest) in &cellars {
        let dungeon = quest.dungeon.unwrap();
        assert_eq!(dungeon.way, DungeonWay::Cellar);
        assert_eq!(dungeon.poi.kind, DUNGEON_KIND);
        assert_eq!(dungeon.poi.chunk, town.chunk);
        assert_ne!(dungeon.poi.seed, town.seed);
        if quest.kind == QuestKind::Daily {
            assert_eq!(dungeon.levels, 1);
        }
        // Every step is in it.
        for step in &quest.steps {
            assert!(
                matches!(step.target, Target::Dungeon { chunk, .. } if chunk == town.chunk),
                "{quest:?}"
            );
        }
        let explores = quest
            .steps
            .iter()
            .any(|s| s.goal == crate::story::Goal::Explore);
        assert_eq!(explores, dungeon.levels > 1);
        assert!(peopled_enough(quest), "{quest:?}");
        // Told as a cellar in town.
        let place = quest.story.facts.place.as_ref().unwrap();
        assert_eq!(place.description.kind, PlaceKind::Cellar);
        quest.story.paragraphs_text(&lang).unwrap();
        quest.story.step_text(&lang, 0, 1).unwrap();
    }
}

/// Its site has its levels, and enough of its kind live there for the count.
fn peopled_enough(quest: &Quest) -> bool {
    let dungeon = quest.dungeon.unwrap();
    let site = Site::with_generator(
        dungeon.poi.kind,
        dungeon.poi.seed,
        CAVE_VERSION,
        dungeon.generator(),
    )
    .unwrap();
    assert_eq!(site.levels(), dungeon.levels);
    let mut living = 0;
    for index in 0..site.levels() {
        let level = site.level(index);
        let stairs: Vec<_> = level.stairs.iter().map(|s| (s.x, s.y)).collect();
        let spawns = dungeon.spawns(index, &level.level, &stairs);
        assert!(spawns.iter().all(|s| s.kind == dungeon.creatures));
        assert_eq!(spawns, dungeon.spawns(index, &level.level, &stairs));
        living += spawns.len() as u32;
    }
    living >= quest.steps[0].count.unwrap()
}

#[test]
fn towns_and_cities_have_hidden_ways_down_nearby() {
    let world = TestWorld::new();
    let lang = Lang::new(Language::English);
    let hidden: Vec<_> = (dungeons().into_iter())
        .filter(|(_, q)| q.dungeon.unwrap().way == DungeonWay::Hidden)
        .collect();
    assert!(!hidden.is_empty());
    for (town, quest) in &hidden {
        use crate::poi::PoiKind;
        assert!(
            matches!(town.kind, PoiKind::Town | PoiKind::City),
            "{town:?}"
        );
        assert_eq!(quest.kind, QuestKind::Normal);
        let dungeon = quest.dungeon.unwrap();
        let poi = dungeon.poi;
        assert_eq!(poi.kind, HIDDEN_KIND);
        // In a chunk of its own near the town, on open land.
        assert_ne!(poi.chunk, town.chunk);
        assert_eq!(world.pois.in_chunk(poi.chunk), None);
        assert_eq!(world.pois.chunk_at(poi.x, poi.y), poi.chunk);
        assert!(world.pois.is_open_land(poi.x, poi.y));
        assert!((poi.chunk.x - town.chunk.x).abs() <= 3 && (poi.chunk.y - town.chunk.y).abs() <= 3);
        assert!(dungeon.levels >= 2);
        for step in &quest.steps {
            let chunk = match step.target {
                Target::Dungeon {
                    chunk,
                    way: DungeonWay::Hidden,
                    ..
                } => chunk,
                other => panic!("{other:?}"),
            };
            assert_eq!(chunk, poi.chunk);
        }
        assert!(peopled_enough(quest), "{quest:?}");
        let place = quest.story.facts.place.as_ref().unwrap();
        assert_eq!(place.description.kind, PlaceKind::of_poi(HIDDEN_KIND));
        quest.story.paragraphs_text(&lang).unwrap();
    }
}

#[test]
fn bigger_towns_have_bigger_cellars() {
    let cellars = dungeons();
    let most = |kind| {
        (cellars.iter())
            .filter(|(town, q)| town.kind == kind && q.kind == QuestKind::Normal)
            .map(|(_, q)| q.dungeon.unwrap().levels)
            .max()
    };
    use crate::poi::PoiKind;
    for small in [PoiKind::Hamlet, PoiKind::Village] {
        assert!(most(small).is_none_or(|levels| levels == 1));
    }
}
