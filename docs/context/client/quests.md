# client: quests

What the client knows of quests, all from the server (`shared::protocol::QuestNews`): the character's quests on this planet, the markers over the doors of the town they are in, and the planets with quests for the Hall's gates. It draws the "!" and "?" over givers' doors, the Hall's gates and world map pois; the maps draw their own from `QuestPois`, and the talk panel is `talk`'s. Plan: `quests.md`.

## Files

- `crates/client/src/quests/mod.rs`: `QuestsPlugin` (`Update`: `read_server` then `find_quest_pois`; `PostUpdate` after transforms: `show_marks`). Tests in `tests.rs`.
- `quests/book.rs`: `QuestBook`, `done_steps`, `DoorMarkers`, `GateQuests`.
- `quests/read.rs`: `read_server` (`Log` replaces the book, `Changed` one entry, `Markers` the doors, `Planets` the gates; `Welcome` forgets all).
- `quests/look.rs`: `MarkLook` (`of(Marker)`, `new(kind, ready)`, `glyph`, `colour`, `rgb`): yellow normal, blue daily, orange artifact, grey locked (owner).
- `quests/poi_marks.rs`: `QuestPois`, `find_quest_pois` and `marks_of` (each open step's poi or town gets "!", a ready quest's giver's town "?", which wins; the wilds round a town have no poi).
- `quests/anchors.rs`: where each marker goes in the view: `door_anchors` (each door with a marker, on its outside face, `OVER_DOOR` cells over the ground), `gate_anchors` (each Hall gate in the stretch whose planet, `GatePlace::address().world().id()`, has quests: "?" if one is ready, else "!", yellow), `poi_anchors` (each poi token shown with a mark).
- `quests/over_view.rs`: `show_marks`: plain Bevy UI nodes (not markup, like health bars), each a sign of `SIZE` with four black copies round it for a ring, projected through `view3d::ViewCamera` each frame, kept while what they mark and how they look stay the same; `Pickable::IGNORE`.

## Public API

- `QuestsPlugin`.
- `QuestBook` (resource: `entries` by `QuestId`, the log's), `done_steps(&LogEntry)`.
- `DoorMarkers` (resource: `town` last told and `markers` by building): `talk` asks the giver of this town.
- `GateQuests` (resource: planets with quests by `WorldId`, and whether one is ready).
- `QuestPois` (resource: `MarkLook` by poi chunk): `minimap` draws them.
- `MarkLook`: how a marker looks, for anything drawing one.

## Depends on

- `common` (`CurrentDungeon`, `DrawnGround`, `GameView`, `Portal`), `connection` (`FromServer`), `dungeon_map` (`CELL_SIZE`), `view3d` (`ViewCamera`, `TokenView`, `TOKEN_DIAMETER`), `shared::protocol`, `shared::quest`, `shared::hall`.

## Used by

- `talk` (`DoorMarkers`, `QuestBook`, `done_steps`), `minimap` (`QuestPois`, `MarkLook`).

## Notes

- Door markers are kept until markers for another town come (not cleared on a new level), as the server sends them only when the player comes to a town; they are drawn only over buildings of the level shown.
- The server sends `Planets` on coming into the Hall, from every planet's saved book (`Progress::ready`), so other planets' "?" needs no quest made again.
- Alignment-hidden quests never reach the client: no grey "!", no hint.
