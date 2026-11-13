# client: tracked_quest

The tracked quest under the minimap (owner): the quest followed on this planet (`quests::QuestBook::followed`: the one chosen with the Quest Log's Track, else the newest taken), in a dark box like the map's hover box. Its giver's marker and its name, a line for each objective that can be done now (steps not done whose earlier steps are) with its count, and "Return to ..." with a "?" in the quest's colour once only handing in is left. A click or tap on it opens the Quest Log on that quest. The dots on the maps are `minimap`'s (from `quests::TrackedDots`). Plan: `quests.md`.

## Files

- `crates/client/src/tracked_quest/mod.rs`: `TrackedQuestPlugin` (`Startup`: `spawn_box`; `Update`: `bind_box` then `show_tracked`). Tests in `tests.rs`.
- `tracked_quest/markup.rs`: `TRACKED` (`tracked/box`, top right, `span class="tip"` round `tracked/lines`; templates `head_<colour>` (a `mark` badge and the `name`), `line`, `return_<colour>` (`to` and the `ready` badge), one per colour as writing takes its class's colour only when made), `spawn_box`, `bind_box` (an observer on the box: a `dom::Click` anywhere in it sends `quests::OpenQuestLog`).
- `tracked_quest/lines.rs`: `objectives`, `return_line`.
- `tracked_quest/show.rs`: `show_tracked` (written again when the quest followed, its progress, the words, the view or `minimap::MinimapArea` change; placed `GAP` under the minimap's picture and as wide; hidden while connecting, with no quest followed, or while the full map is open).

## Public API

- `TrackedQuestPlugin`.

## Depends on

- `common` (`GameView`, `Wording`), `quests` (`QuestBook`, `MarkLook`, `done_steps`, `or_partly`, `OpenQuestLog`), `minimap` (`MinimapArea`), `dom` (`Dom`, `Click`, `document`, `top_strip`), `widgets` (`UiId`), `shared::protocol`, `shared::lang`.

## Used by

- Nothing.

## Notes

- "Return to ..." shows only once every step is done (Claude's choice: it is the next objective then; the Quest Log lists it always).
- Its words are the Quest Log's: the story's title, `step_text` with counts, `return_text`.
