# shared: story

Quests' stories: who is in them (households, pets, named bosses), where they went (unnamed places, direction and distance from a town), and the stories themselves: a reason and beats chosen from the text seed for a quest's facts, in up to three paragraphs, sent as a `Story` the client words in its language. Names are sounds and never translated; everything else is a value (`lang::Selector`). Plan: `story-text.md`; catalogue: `research-story-plots.md`.

## Files

- `crates/shared/src/story.rs`: `mod` lines, re-exports; `is_people` (every kind listed: animals have no families), `offset` (the near way round a planet, north −y).
- Names and places (PR 2): `story/household.rs` (`Household`, `Person`), `family_names.rs`, `pet.rs` (`PetKind`, `Pet`), `boss.rs`, `epithet.rs`, `place.rs` (`PlaceKind`, `Condition`, `PlaceDescription`), `direction.rs`, `distance.rs`.
- `story/fact.rs`: the `fact!` macro: an enum of values, `ALL`, and its `Selector`. `story/facts.rs` and `facts/`: `people.rs` (`Relation`, `Trait`, `State`, `Passerby`), `events.rs` (`Errand`, `TimeAgo`, `Occasion`, `Stake`), `things.rs` (`ItemCategory`, `CountBand`, `Season`, `Weather`, `RewardPart`), `reason.rs` (`Goal`, `Reason` and their weights). `creature_words.rs`: `CreatureKind` as a selector.
- `story/beat.rs`: `Beat` (the catalogue's H, C, D, T, Q, A, S beats and the ending's), `Slot`, which reasons each suits.
- `story/cast.rs`: `Role`, `Member`, `CastPet`, `Cast`, `Carried`. `story/request.rs`: `Request`, `Ask`, `Place`, `Creatures`, `Destination`, `Length`. `story/told.rs`: `Story`, `Facts`, `Telling`, `Ending`, `Lie`, `Truth`.
- Telling: `story/plot.rs` (`tell`, `choose_reason`, drawing beats, lies, the ending), `draft.rs` (a story being made, `finish` keeps what was used), `roll.rs` (candidates for each role and colour fact), `fits.rs` (a beat's needs), `apply.rs` (what a beat uses and the states it sets).
- Wording: `story/words.rs` (`Story::args`, the reward and creatures in words), `render.rs` (paragraphs, title, steps, ending; picking a wording).
- English: `lang/en/story.ftl` (names, places, where), `lang/en/story-plots.ftl` (facts, beats with five wordings each, titles, steps, and helpers wordings call: `creatures-some`, a pronoun message per role and form such as `relative-he`). Tests: `wording_tests.rs` (what each beat is sure to have, `sure`: every wording renders with only that in every gender mix and creature count, and varied quests confirm it), `tests.rs`, `where_tests.rs`, `words_tests.rs`, `plot_tests.rs` (the rules over many seeds, every story renders), `sample_tests.rs` (prints samples with `--nocapture`; a golden of one story's beats).

## Public API

- Names: `Household::new(kind, seed).member(index) -> Person` (`full_name(&lang)`), `Pet::new(seed)`, `Pet::of_kind`, `PetKind`, `Boss::new(kind, seed)` (`full_name`), `Epithet`, `PlaceKind::of_poi`, `PlaceDescription::new(kind, ruined).describe(&lang)`, `Direction::between`, `DistanceBand::between` (`NEAR_WITHIN`, `SHORT_WITHIN`, `DAY_WITHIN`, *to balance*).
- Facts (each with `ALL` and a `Selector`): `Relation` (`suits(gender)`, `is_kin`, `is_elder`), `Trait`, `State` (`can_become`: the dead stay dead), `Passerby`, `Errand` (`allowed_at(place)`), `TimeAgo`, `Occasion`, `Stake`, `ItemCategory` (`is_keepsake`), `CountBand` (`of(count)`), `Season`, `Weather`, `Goal`, `Reason` (`goal_weight`, `place_weight`). `RewardPart::{Coins(copper), Item(category), Training, Favour}`.
- **Telling:** `Request::new(text_seed, household, giver_index, home, goal)`, then set what the quest has: `asks` (1 or 2 `Ask { goal, count }`), `place`, `creatures`, `boss`, `item`, `destination`, `pet`, `reward`, `higher_reward` with `season`/`weather`, `reason` (a chain keeps it), `later_step`, `dark`, `continues`, `length` (`Short` for dailies), `carried`. `tell(&request) -> Telling { story, ending }`: send `story` with the offer; keep `ending` (with any `Lie`s and their `Truth`) until turn-in.
- Goals need: Find an item and a place; Slay creatures (or a boss) and a place; Deliver an item and a destination; Gather an item; Escort a place (a `pet`, or the relative); Explore and Visit a place.
- `Story { text_seed, reason, cast, facts, paragraphs }` (serde). `Cast { members, pet, boss, passerby }`, `member(role)`, `giver()`. **Carrying:** `story.carry(&[Role::Giver, Role::Relative], pet, boss) -> Carried` (cast plus the backstory's errand, time and occasion) into a later step's `Request::carried`; `Member::moved_to(state)` applies a finished step's state, if forward.
- **Wording:** `story.paragraphs_text(&lang)`, `title(&lang)`, `step_text(&lang, index, done)` ("Slay kobolds 2/6"), `return_text(&lang)`, `ending_text(&lang, &ending)`; `args(&lang)` (every value, for one's own messages); `wording_of(&lang, beat)`; `reward_text(&lang, &parts)`. All return `Result<_, lang::RenderError>`.

## Depends on

- `shared::rng`, `shared::creature` (`CreatureKind`, names), `shared::gender`, `shared::lang`, `shared::poi` (`PoiKind`, `CHUNK_SIZE`), `shared::item` (`coin_stacks`).

## Used by

- `shared::quest` (`telling.rs`: builds each quest's `Request` and tells it); later the server sends `Story`s and the client renders them.

## Notes

- Seeds: callers pass seeds of their own (never a poi's map seed, which is secret). The `Story`'s text seed reaches clients. Each draw derives its own purpose ("cast", "facts", "beats", "reason", "wording"), so changing one list or rule doesn't shift the others; a golden test pins one story's beats.
- Beats: paragraph 1 is one or two hooks, 2 up to two complications (twists first in later steps), 3 the asks, stakes and the reward (always named, owner). Short stories are one paragraph. A beat is drawn only when its reasons, darkness, twist rule and needs (`fits`) allow; asks follow the goals. Each beat may only use the values it is sure to have (see `fits.rs`); `plot_tests` renders every story to check.
- Dark beats only when `Request::dark` (the quest needs a bad reputation, decided by the quest from `shared::alignment`); such a story always has one. Nothing about alignment is in `Story`.
- Lies: now and then the errand told is false (the giver, or the witness for a rumour); the truth is in `Ending`, told at turn-in. A carried errand is never lied about anew.
- Wording: a beat's message `beat-<key>` holds wordings as attributes; one is picked by `hash(text seed, message id) % count`, so languages may have different counts, and adding wordings rewords old quests.
- Values beyond the facts: `items` (the item's plural, its message's `.plural`), `creatures-some` ("a kobold", "a few wolves"), and at turn-in `liar-is-giver` (`yes`/`no`) beside `liar-name` and `truth`. A Slay step with no creatures (a boss alone) renders `step-slay.place` ("Clear the cave").
- A beat's "sure to have" changes with `fits.rs` or `apply.rs`: update `sure` in `wording_tests.rs` with it; the test against varied quests fails if the table claims too much. English wordings were drafted with `research-tools/llm.py` in shorthand (`{he:relative}`, `{was|were:relative}`) expanded into Fluent, then chosen by hand and approved by the owner (Flavour text PR 4).
- Left over: given names aren't gendered (peoples' name sounds have no forms); skill names aren't worded (so `RewardPart::Training` names no skill); the Donor role and the catalogue's per-step Truth on facts other than the errand aren't built; season and weather come from the quest (the world has no seasons yet).
