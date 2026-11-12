//! Every wording of every beat renders with only the values its beat is sure
//! to have (`SURE`), for every gender and creature count; and `SURE` holds:
//! stories told from many varied requests have those values whenever they
//! have the beat. Wordings may use the giver and home anywhere.

use super::plot_tests::request;
use super::*;
use crate::gender::Gender;
use crate::lang::{Args, Lang, Language, Selector};
use crate::rng::Seed;

/// What a beat is sure to have, besides the giver and home: roles and facts
/// by the names in `values`.
fn sure(beat: Beat) -> &'static [&'static str] {
    use Beat::*;
    match beat {
        RelativeWent | DiedThere => &["relative", "place", "errand", "time-ago"],
        HeirloomLost => &["relative", "item", "place", "time-ago"],
        DrivenOff => &["creatures", "place", "time-ago"],
        PetStrayed => &["pet", "place", "time-ago"],
        RumourHeard | Silence => &["witness", "place"],
        NeighbourWrong | KeepItQuiet | RivalSeeksIt | Accused | NotGuilty | WhoToHelp | Honour
        | Revenge => &["rival"],
        DebtOwed => &["recipient", "destination"],
        OrderPending | KeepOrReturn | Collect => &["item"],
        ThreatSeen => &["creatures", "time-ago"],
        DyingWish | Eloped | FateUnknown | LeftOnPurpose | Promise => &["relative"],
        GiverWasThere => &["place", "time-ago"],
        PlaceHistory | FirstTryFailed | WrongPlaceSearched | Clear | Look | Found => &["place"],
        BossNamed | BossWasOneOfUs | SpareOrSlay => &["boss"],
        SickAtHome => &["relative", "item"],
        Murdered => &["relative", "rival"],
        CreaturesMovedIn => &["creatures", "place"],
        TimePressure | ForOccasion => &["occasion"],
        Guilt => &["pet"],
        WeatherBars => &["weather", "place"],
        Retrieve => &["item", "place"],
        LearnFate | Bring => &["relative", "place"],
        Take => &["item", "recipient", "destination"],
        BringPet => &["pet", "place"],
        LayToRest => &["relative", "item", "place"],
        StealFrom => &["rival", "item", "place"],
        Frame => &["rival", "item", "destination"],
        Safety | Warning => &["creatures"],
        Reward => &["reward"],
        SeasonTurns => &["season"],
        TruthFound => &["relative", "place", "truth"],
        StudyWanted | GiverCannot | NobodyBelieves | CannotPay | Memory | Livelihood
        | ChildMisses | Thanks | StakeMet | Lead => &[],
    }
}

const GENDERS: [Gender; 3] = [Gender::Male, Gender::Female, Gender::Neutral];

fn person(args: Args, role: &str, gender: Gender) -> Args {
    args.name(format!("{role}-name"), "Mara Tull")
        .name(format!("{role}-given"), "Mara")
        .choice(format!("{role}-gender"), &gender)
        .text(format!("{role}-nature"), "kind")
        .text(format!("{role}-state"), "alive")
        .text(format!("{role}-relation"), "friend")
}

/// Values for one need: a role in `gender`, or a fact (creatures `count`).
fn values(args: Args, need: &str, gender: Gender, count: i64) -> Args {
    match need {
        "relative" | "rival" | "recipient" => person(args, need, gender),
        "witness" => (args.name("witness-name", "a shepherd")).choice("witness-gender", &gender),
        "pet" => args
            .name("pet-name", "Bramble")
            .text("pet-kind", "dog")
            .text("pet-nature", "curious")
            .choice("pet-gender", &gender),
        "boss" => (args.name("boss-name", "Grisk the Grey")).choice("boss-gender", &gender),
        "place" => args
            .text("place", "ruined tower")
            .text("place-named", "no")
            .text("place-direction", "north of town")
            .text("place-distance", "days away"),
        "creatures" => args
            .text("creatures", if count == 1 { "orc" } else { "orcs" })
            .count("creature-count", count)
            .text("creature-number", if count == 1 { "one" } else { "many" }),
        "item" => args.text("item", "ring").text("items", "rings"),
        "destination" => args
            .name("destination", "Little Fen")
            .text("destination-direction", "east of town")
            .text("destination-distance", "a short walk away"),
        "truth" => (args.name("liar-name", "Mara Tull")).text("truth", "to trade"),
        "liar" => args.text("liar-is-giver", if count == 1 { "yes" } else { "no" }),
        other => args.text(other, "something"),
    }
}

#[test]
fn every_wording_renders_with_only_what_its_beat_is_sure_to_have() {
    let lang = Lang::new(Language::English);
    for &beat in Beat::ALL {
        let id = format!("{}-{}", beat.fact(), beat.key());
        let wordings = lang.attributes(&id);
        assert!(!wordings.is_empty(), "{id} has no wordings");
        let mut needs = sure(beat).to_vec();
        if beat == Beat::TruthFound {
            needs.push("liar");
        }
        for gender in GENDERS {
            for count in [1, 3] {
                let mut args = person(Args::new(), "giver", gender).name("home", "Ashford");
                for need in &needs {
                    args = values(args, need, gender, count);
                }
                for wording in &wordings {
                    let id = format!("{id}.{wording}");
                    let text = lang.render(&id, &args).unwrap_or_else(|e| panic!("{e}"));
                    assert!(!text.contains(['{', '}', '$']), "{id}: {text}");
                }
            }
        }
    }
}

/// Requests with only what their goals need, varied by seed: no creatures,
/// item, boss, reward or place unless kept, two asks, pets, dark, short.
fn varied(seed: u64) -> Request {
    let goal = Goal::ALL[(seed % Goal::ALL.len() as u64) as usize];
    let mut request = request(seed / 7, goal);
    request.text_seed = Seed::new(seed).derive("varied");
    let bit = |n: u32| (seed >> (3 + n)) & 1 == 1;
    let needs_item = matches!(goal, Goal::Find | Goal::Deliver | Goal::Gather);
    let needs_place = !matches!(goal, Goal::Deliver | Goal::Gather);
    if goal == Goal::Slay && bit(0) {
        request.boss = Some(Boss::new(
            crate::creature::CreatureKind::Orc,
            Seed::new(seed),
        ));
        request.creatures = None;
    } else if goal != Goal::Slay && bit(0) {
        request.creatures = None;
    }
    if !needs_item && bit(1) {
        request.item = None;
    }
    if !needs_place && bit(2) {
        request.place = None;
    }
    if bit(3) {
        request.reward.clear();
    }
    if goal == Goal::Escort && bit(4) {
        request.pet = Some(Pet::new(Seed::new(seed)));
    }
    if goal == Goal::Gather && bit(5) {
        request.asks.push(Ask {
            goal: Goal::Deliver,
            count: None,
        });
        request.destination = request.destination.or(Some(Destination {
            name: "Little Fen".into(),
            kind: crate::creature::CreatureKind::Halfling,
            direction: Direction::East,
            distance: DistanceBand::Day,
        }));
    }
    request.dark = bit(6) && bit(7);
    request.continues = bit(8);
    if bit(9) && bit(10) {
        request.length = Length::Short;
    }
    request
}

fn check(lang: &Lang, telling: &Telling) {
    let story = &telling.story;
    let args = story.args(lang).unwrap();
    let beats = story.beats().chain(telling.ending.beats.iter().copied());
    for beat in beats {
        for need in sure(beat) {
            let keys: &[&str] = match *need {
                "relative" | "rival" | "recipient" => &["-name", "-given", "-gender", "-relation"],
                "witness" | "pet" | "boss" => &["-name", "-gender"],
                "creatures" => &["", "-count"],
                "item" => &["", "s"],
                "truth" => &[],
                _ => &[""],
            };
            for key in keys {
                let key = match (*need, *key) {
                    ("creatures", "-count") => "creature-count".to_string(),
                    _ => format!("{need}{key}"),
                };
                assert!(args.has(&key), "{beat:?} without {key}: {story:?}");
            }
        }
        // Every wording, not just the one picked.
        let id = format!("{}-{}", beat.fact(), beat.key());
        for wording in lang.attributes(&id) {
            if beat == Beat::TruthFound {
                continue; // needs the ending's values: ending_text renders it
            }
            let text = lang.render(&format!("{id}.{wording}"), &args);
            text.unwrap_or_else(|e| panic!("{e}: {story:?}"));
        }
    }
    story.title(lang).unwrap();
    for index in 0..story.facts.asks.len() {
        story.step_text(lang, index, 1).unwrap();
    }
    story.ending_text(lang, &telling.ending).unwrap();
}

#[test]
fn beats_have_what_they_are_sure_to_have_in_varied_quests() {
    let lang = Lang::new(Language::English);
    let mut told = std::collections::BTreeSet::new();
    for seed in 0..4000 {
        let first = varied(seed);
        let telling = tell(&first);
        check(&lang, &telling);
        told.extend(telling.story.beats());
        let mut later = first.clone();
        later.text_seed = first.text_seed.derive("step 2");
        later.reason = Some(telling.story.reason);
        later.later_step = true;
        later.carried =
            (telling.story).carry(&[Role::Giver, Role::Relative, Role::Rival], true, true);
        let telling = tell(&later);
        check(&lang, &telling);
        told.extend(telling.story.beats());
    }
    let never: Vec<_> = (Beat::ALL.iter())
        .filter(|b| b.slot() != Slot::Ending && !told.contains(b))
        .collect();
    assert!(never.len() <= 2, "never told: {never:?}");
}
