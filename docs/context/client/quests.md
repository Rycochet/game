# client: quests

What the client knows of quests, all from the server (`shared::protocol::QuestNews`): the character's quests on this planet, the markers over the doors of the town they are in, and the planets with quests for the Hall's gates. It draws the "!" and "?" (and the grey "?" for a quest under way) over givers' doors, the Hall's gates and world map pois; the maps draw their own from `QuestPois`, the talk panel is `talk`'s and the Quest Log `quest_log`'s; both write quests' words through `words.rs`. Plan: `quests.md`.

## Files

- `crates/client/src/quests/mod.rs`: `QuestsPlugin` (`Update`: `read_server`, `find_quest_pois`, `find_tracked_dots`; `PostUpdate` after transforms: `show_marks`). Tests in `tests.rs`.
- `quests/book.rs`: `QuestBook` (entries, `dailies`, `tracked`; `followed`), `done_steps`, `DoorMarkers`, `GateQuests`, `QuestPlanets`.
- `quests/read.rs`: `read_server` (`Log` replaces the book with its dailies and tracked quest, `Changed` one entry, `Tracked`, `Markers` the doors, `Planets` the gates (those with quests `taken`) and `QuestPlanets`; `Dots` into `LevelDots`; `Welcome` forgets all).
- `quests/look.rs`: `MarkLook` (`of(Marker)`, `of_state(kind, QuestState)` for the log, `new(kind, ready)`, `glyph`, `class` (its badge's `span.mark-<colour>`), `colour`, `rgb`): yellow normal, blue daily, orange artifact, grey locked "!" or under way "?" (owner); the colours are `common`'s `QUEST_*`.
- `quests/words.rs`: `or_partly`, `plain`, `step_lines` (each step with "(done)"), `reward_line`: for `talk` and `quest_log`.
- `quests/dots.rs`: `LevelDots` (the server's `QuestNews::Dots`), `TrackedDots` and `find_tracked_dots` (the followed quest's colour and dots: on the world map each open step's poi or town, or the giver's town to hand in (`world_chunks`), placed by `KnownPois`; on a level the server's cells), `OpenQuestLog`.
- `quests/poi_marks.rs`: `QuestPois`, `find_quest_pois` and `marks_of` (each open step's poi or town gets "!", a ready quest's giver's town "?", which wins; the wilds round a town have no poi).
- `quests/anchors.rs`: where each marker goes in the view: `door_anchors` (each door with a marker, on its outside face, `OVER_DOOR` cells over the ground), `gate_anchors` (each Hall gate in the stretch whose planet, `GatePlace::address().world().id()`, has quests: "?" if one is ready, else "!", yellow), `poi_anchors` (each poi token shown with a mark).
- `quests/over_view.rs`: `show_marks`: plain Bevy UI nodes (not markup, like health bars), each a sign of `SIZE` with four black copies round it for a ring, projected through `view3d::ViewCamera` each frame, kept while what they mark and how they look stay the same; `Pickable::IGNORE`.

## Public API

- `QuestsPlugin`.
- `QuestBook` (resource: `entries` by `QuestId`, the log's; `dailies`; `tracked`, the one chosen; `followed()`, that one while under way, else the newest taken: PR 6's), `done_steps(&LogEntry)`.
- `QuestPlanets` (resource: every planet with quest history, as last told, for the Quest Log).
- `or_partly`, `plain`, `step_lines`, `reward_line`: quests' words.
- `DoorMarkers` (resource: `town` last told and `markers` by building): `talk` asks the giver of this town.
- `GateQuests` (resource: planets with quests by `WorldId`, and whether one is ready).
- `QuestPois` (resource: `MarkLook` by poi chunk): `minimap` draws them.
- `MarkLook`: how a marker looks, for anything drawing one.
- `LevelDots` (resource: the server's dot cells by quest, for the level the player is on), `TrackedDots` (resource: `rgb` and `places` of the followed quest's dots, world map points on the world map, else cells): `minimap` draws them.
- `OpenQuestLog(QuestId)` (message): asks the Quest Log to open on a quest (`tracked_quest` sends it, `quest_log` reads it).

## Depends on

- `common` (`CurrentDungeon`, `CurrentWorld`, `DrawnGround`, `GameView`, `Portal`, `QUEST_*` colours), `connection` (`FromServer`), `dungeon_map` (`CELL_SIZE`), `view3d` (`ViewCamera`, `TokenView`, `TOKEN_DIAMETER`), `shared::protocol`, `shared::quest`, `shared::hall`.

## Used by

- `talk` (`DoorMarkers`, `QuestBook`, the words), `quest_log` (`QuestBook`, `QuestPlanets`, `MarkLook`, `done_steps`, the words), `minimap` (`QuestPois`, `MarkLook`, `TrackedDots`), `tracked_quest` (`QuestBook`, `MarkLook`, `done_steps`, `or_partly`, `OpenQuestLog`).

## Notes

- Door markers are kept until markers for another town come (not cleared on a new level), as the server sends them only when the player comes to a town; they are drawn only over buildings of the level shown.
- A door shows the server's one marker per giver (owner: coloured "?", grey "?", "!", grey "!", first found).
- The server sends `Planets` on coming into the Hall (and when the Quest Log asks), from every planet's saved book (`Progress::ready`), so other planets' "?" needs no quest made again.
- Alignment-hidden quests never reach the client: no grey "!", no hint.
