# client: debug_menu

The debug menu: backtick or the warning button top left opens a full-screen list of actions, filtered by typing, chosen by click, tap, arrow keys and Return. Replaces the old demo keys. Works in native and web builds.

## Files

- `crates/client/src/debug_menu/mod.rs`: `DebugMenuPlugin`, `AutoWalk`, `TemperatureCycle`, `run_actions` (does what `DebugAction` says), `auto_walk`, `temperature_cycle`, `FactSources`.
- `crates/client/src/debug_menu/actions.rs`: `DebugAction` (every entry, alphabetical), its name, `is_available`, `is_on`, `matches`, `next_highlight`. Pure, unit tested. To add an entry: add it here (`ALL` and the matches) and to `run_actions`.
- `crates/client/src/debug_menu/menu.rs`: `DebugMenu`, `spawn_menu`, the warning icon (drawn in code), `take_input` (`PreUpdate`, `WidgetSystems::Modal`), `show_menu` (rebuilds the grid when what it says changes).
- `crates/client/src/debug_menu/panels.rs`: moveable panels the menu opens: Previous and Next for stairs, fields and herbs (`StepButton`), and Show Widgets (one of every widget). `OpenPanels`.
- `crates/client/src/debug_menu/soft_keyboard.rs`: in browsers, a hidden HTML input for the on-screen keyboard. Does nothing natively.

## Public API

- `DebugMenuPlugin`.
- `DebugAction` (message): do this, as if chosen. Used by `dev_tools` (`GAME_PANEL`).
- `DebugMenu` (resource): `open`, `filter`; `open_with` (native, for `GAME_DEBUG_MENU`).
- `AutoWalk(bool)` (resource): the player walks east one step per frame on the world map. `TemperatureCycle` (resource): `Off`, `Cooling`, `Warming`; the snow line falls to 0 and back through the server (`DevSnowLine`), and it turns itself off at the end.

## Depends on

- `common` (`ClickMode`, `DebugInfo`, `CurrentDungeon`, `GameView`, `HerbGrowthShift`, `TouchGesture`, `Weather`), `connection` (`ToServer`), `player` (`TakeStep`, `PlayerSystems`), `view3d` (`OrbitCamera`), `widgets`, `dungeon_map` (`CROP_STAGES`, `HERB_STAGES`), `shared::movement`, `shared::showcase`, `shared::site`.

## Used by

- `dev_tools`.

## Notes

- Owner's design, after RimWorld's: choosing an entry closes the menu; a toggle shows its new state for `TOGGLE_SECONDS` (1) first. Disabled entries stay visible and do nothing. Columns fill top to bottom, then overflow into the next.
- Claude's choices: Return with nothing highlighted takes the first entry that can be used; arrow keys can highlight disabled entries; Esc and its close button close it too; turning Cycle Temperature off puts the snow line straight back; only one of Teleport and Change Cell Mode can be on. Reset View is here as tablets have no Home key (Home still works).
- While open it takes all input: `take_input` clears keys, wheel and gestures, and its root stops presses, so nothing behind it reacts.
- On-screen keyboard: browsers only allow focusing an input during the tap, so a `pointerup` listener on the canvas focuses the hidden input if the tap is on the filter box, whose place is sent each frame. With it focused the game's keys see nothing, so the input's Return and Escape come through `take_keys`, and arrow keys on an attached keyboard don't work until tapping elsewhere is added. Untested on a real iPad.
- Panels stay open with greyed buttons where they can't act. Stairs, fields and herbs changes are only what is drawn; the next level goes back to its own, except herbs (`HerbGrowthShift` stays).
- Suns and Moons (`bodies.rs`): a row per sun, a gap, a row per moon: a swatch of its colour and a slider along its kind's colours (`shared::celestial::colour_along`), and for suns a slider for intensity (`MOST_INTENSITY`; above about 1.15 on flat ground `world_map`'s cap holds it). Only this client's drawing, until the world is joined again. A generated body can be paler than anything on its slider: the mark goes to the nearest, and the swatch shows it exactly until it is moved.
- World Time and Suns and Moons toggle their panels, and their entries have checkboxes saying whether they are open (`OpenPanels::toggle`, `forget_closed` for close buttons).
- World Time's panel asks the server to move the world's clock an hour either way (`DevSetTime`), on either map, for everyone playing. It stops at the world's beginning.
- Back to Start, Change Cell Mode, Cycle Temperature, Forget Map, Poi Showcase, Reveal Map and Teleport Mode need the server's dev tools (`common::ServerDevTools`, from `Welcome`), and are disabled where it refuses them. The embedded server always allows them; a standalone one with `HALL_OF_WORLDS_DEV_TOOLS=1`.
- Reveal Map and Forget Map are answered by the server sending the level again as if arriving where the player stands (`EnterLevel`), or for forgetting the world map `ForgetWorldMap` and the chunks in sight. Both are disabled in towns, which arrive whole.
- Drilling down through several layers isn't built, as nothing uses it yet.
