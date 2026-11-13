# client: quest_log

The Quest Log (L, `common::Action::QuestLog`): a moving panel in markup with the character's quests on any planet they have quest history on. Left: the planet chooser (the one the player is on first, "Here", each named by its runes, a picture drawn from `shared::hall::RUNE_STROKES`, never a font), then the list grouped Active, Completed, Failed or abandoned (newest first), each quest after its giver's marker as a badge, and a line for each giver of dailies. Right: the quest chosen in full, with Track and Abandon for one under way on this planet, and buttons for the places it sends you to. Plan: `quests.md`.

## Files

- `crates/client/src/quest_log/mod.rs`: `QuestLogPlugin` (`Update`, in order: `notice_closed`, `toggle_log`, `read_server`, `read_events`, `show_log`). Tests in `tests.rs`.
- `quest_log/markup.rs`: `QUEST_LOG` (`quest_log/...`): `panel`, `planet` (a `select`), `list` in a `scroll`, `choose`, `detail` (`title`, `giver`, `state`, `paras`, `steps_head`, `steps`, `reward`, `targets_head`, `targets`, `track`, `abandon`), templates `heading`, `row_yellow` / `row_blue` / `row_orange` / `row_grey` (a `mark` badge and a `quest` button: one per colour, as writing takes its class's colour only when made), `line`, `para`, `step`, `target`.
- `quest_log/state.rs`: `LogPanel` (document, the planet shown (`None` here), the quest chosen, `abandoning`, other planets' logs as `Told`, the chooser's `choices`, `rows` and `targets` by index, rune pictures by planet, `planets_dirty`, `dirty`).
- `quest_log/open.rs`: `toggle_log` (L opens it, asking `QuestAsk::Planets`, or closes it; not while connecting), `notice_closed` (its close button, or leaving the game).
- `quest_log/read.rs`: `read_server` (`QuestNews::PlanetLog` into `others`; `quests::QuestPlanets` or this planet's `QuestBook` changing marks it dirty; `Welcome` forgets all).
- `quest_log/input.rs`: `read_events` (the chooser (another planet asks `QuestAsk::PlanetLog`), a row, Track (`QuestAsk::Track`, none if it was the one chosen), Abandon pressed twice (`QuestAsk::Abandon`), a place opening `minimap::FullMap` centred on its poi).
- `quest_log/show.rs`: `show_log` (the chooser when `planets_dirty`, the page when `dirty` or the view changes).
- `quest_log/planets.rs`: `choices`, `show_planets` (options by index, "Here" for the planet the player is on and no words for others, each option's runes through `Dom::set_option_pictures` when every planet's code is known).
- `quest_log/page.rs`: `id`, `id_at`, `shown` (the planet's entries and dailies, or `None` until told), `show_page` (headings, rows, daily lines, then `detail`).
- `quest_log/detail.rs`: `show_detail` (name, "From <giver> of <town>", where it stands, story, steps and Return, reward, places still to go by known name (`common::poi_name`), Track selected for `QuestBook::followed`, Abandon's words).
- `quest_log/entries.rs`: `Group` (`Active`, `Completed`, `Failed`; `of`, `heading`), `grouped`.
- `quest_log/runes.rs`: `picture` (a planet's code as a strip of ink-coloured runes).

## Public API

- `QuestLogPlugin`.

## Depends on

- `common` (`Action`, `CurrentWorld`, `KnownPois::poi_in`, `poi_name`, `GameView`, `Preferences`, `Session`, `Wording`, `local_date_time`), `quests` (`QuestBook`, `QuestPlanets`, `MarkLook`, `done_steps`, `or_partly`, `plain`, `reward_line`, `step_lines`), `minimap` (`FullMap`), `connection` (`ToServer`, `FromServer`), `dom` (`Dom`, `UiEvent`, `document`, `top_strip`), `widgets` (`UiId`, `INK`), `shared::protocol`, `shared::hall` (`GateCode`, `RUNE_STROKES`), `shared::lang`.

## Used by

- Nothing.

## Notes

- Words are in `shared/src/lang/en/quest-log.ftl`; steps, reward and "What to do:" are the talk panel's (`talk.ftl`), written through `quests`' word helpers.
- Another planet's quests are the server's to make again: until `PlanetLog` comes the page says "Asking...". Track, Abandon and the places show only for the planet the player is on.
- The places' buttons are greyed off the world map, as the full map shows a site's floors there.
- Only the panel's own buttons and chooser are used, so a tap works as a click; nothing opens it by touch yet (a HUD button is the owner's to place).
