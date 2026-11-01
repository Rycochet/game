# client: debug_menu

The debug menu: backtick or the warning button top left opens a full-screen layer (`widgets`, 90% of the window) listing actions, filtered by typing, chosen by click, tap, arrow keys and Return. Entries with a submenu end in "..." (`DebugAction::label`, by `has_submenu`), and their plain name is the submenu's title (owner's convention). Replaces the old demo keys. Works in native and web builds.

## Files

- `crates/client/src/debug_menu/mod.rs`: `DebugMenuPlugin`, `AutoWalk`, `TemperatureCycle`, `run_actions` (does what `DebugAction` says), `auto_walk`, `temperature_cycle`, `FactSources`.
- `crates/client/src/debug_menu/actions.rs`: `DebugAction` (every entry, alphabetical), its name, `is_available`, `is_on`, `matches`, `next_highlight`. Pure, unit tested. To add an entry: add it here (`ALL` and the matches) and to `run_actions`.
- `crates/client/src/debug_menu/menu.rs`: `DebugMenu`, `spawn_menu`, the warning icon (drawn in code), `take_input` (`PreUpdate`, `WidgetSystems::Modal`), `show_menu` (rebuilds the grid when what it says changes).
- `crates/client/src/debug_menu/panels.rs`: moveable panels the menu opens: Previous and Next for stairs, fields and herbs (`StepButton`), and Show Widgets (one of every widget). `OpenPanels`.

## Public API

- `DebugMenuPlugin`.
- `DebugAction` (message): do this, as if chosen. Used by `dev_tools` (`HOW_DEBUG_PANEL`).
- `DebugMenu` (resource): `open`, `filter`; `open_with` (native, for `HOW_DEBUG_MENU`).
- `AutoWalk(bool)` (resource): the player walks east one step per frame on the world map. `TemperatureCycle` (resource): `Off`, `Cooling`, `Warming`; the snow line falls to 0 and back through the server (`DevSnowLine`), and it turns itself off at the end.

## Depends on

- `common` (`ClickMode`, `DebugInfo`, `CurrentDungeon`, `GameView`, `HerbGrowthShift`, `TouchGesture`, `Weather`), `connection` (`ToServer`), `player` (`TakeStep`, `PlayerSystems`), `view3d` (`OrbitCamera`), `minimap` (`FullMap`), `widgets`, `dungeon_map` (`CROP_STAGES`, `HERB_STAGES`), `shared::movement`, `shared::showcase`, `shared::site`.

## Used by

- `dev_tools`.

## Notes

- Owner's design, after RimWorld's: choosing an entry closes the menu; a toggle shows its new state for `TOGGLE_SECONDS` (1) first. Disabled entries stay visible and do nothing. Columns fill top to bottom, then overflow into the next.
- Claude's choices: Return with nothing highlighted takes the first entry that can be used; arrow keys can highlight disabled entries; Esc, its close button and a click outside it close it too; backtick does nothing over the full map, as the menu would be the bigger of the two (owner's rule: a layer above is smaller); turning Cycle Temperature off puts the snow line straight back; only one of Teleport and Change Cell Mode can be on. Reset View is here as tablets have no Home key (Home still works).
- While open it takes all input: `take_input` clears keys, wheel and gestures, and its layer stops presses, so nothing behind it reacts.
- With the filter box focused in a browser, the game's keys see nothing, so its Return and Escape come through `widgets::SoftKeyPressed`, and arrow keys on an attached keyboard don't work until tapping elsewhere is added.
- Panels stay open with greyed buttons where they can't act. Stairs, fields and herbs changes are only what is drawn; the next level goes back to its own, except herbs (`HerbGrowthShift` stays).
- Suns and Moons (`bodies.rs`): a row per sun, a gap, a row per moon: a swatch of its colour and a slider along its kind's colours (`shared::celestial::colour_along`), and for suns a slider for intensity (`MOST_INTENSITY`; above about 1.15 on flat ground `world_map`'s cap holds it). Only this client's drawing, until the world is joined again. A generated body can be paler than anything on its slider: the mark goes to the nearest, and the swatch shows it exactly until it is moved.
- World Time and Suns and Moons toggle their panels, and their entries have checkboxes saying whether they are open (`OpenPanels::toggle`, `forget_closed` for close buttons).
- World Time's panel asks the server to move the world's clock an hour either way (`DevSetTime`), on either map, for everyone playing. It stops at the world's beginning.
- Change Cell Mode, Cycle Temperature, Forget Map, Forget Map (Server), Reveal Map, Teleport Mode, Teleport to POI Showcase and Teleport to Start need the server's dev tools (`common::ServerDevTools`, from `Welcome`), and are disabled where it refuses them. The embedded server always allows them; a standalone one with `HOW_DEBUG_TOOLS=true`.
- Reveal Map and Forget Map are answered by the server sending the level again as if arriving where the player stands (`EnterLevel`), or for forgetting the world map `ForgetWorldMap` and the chunks in sight. Both are disabled in towns, which arrive whole. Forget Map on the world map also forgets what the character remembers of every site. Forget Map (Server) does the same and has the server forget the site or, on the world map, every site (`DevForgetMapOnServer`), so they are made anew with the current generators; it works in towns too.
- Generate POI... (owner's design): a second layer (75%) with the same grid and search, listing every kind alphabetically. Choosing one closes both layers and opens a moveable panel (`panels::GenerateKind`, `GenerateForm`): the kind as words, ruin 0 to 5 (grey for kinds that can't be), the seed box with a new seed button (a drawn round arrow, as the font has none), and Generate (`DevGenerateSite`). Claude's choices: the panel stays open to try more seeds, and choosing another kind replaces it; Esc, the close button or a click outside goes back to the first layer with its filter kept; backtick closes everything; a new seed has up to 9 digits, though any `u64` can be typed. Where columns don't fit, the grid shows a window of them, moved by the wheel, by following the arrow keys, or by the `<` and `>` buttons in the title row, which show only then.
