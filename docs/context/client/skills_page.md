# client: skills_page

The Skills page: a full-screen layer showing the skills the character knows as a radial tree round their name, moved by dragging and zoomed by the wheel or a pinch, with a line about the chosen skill under it. Not the skills themselves (`shared::skill`) nor the Abilities page's button that opens it (`abilities`). Native and web.

## Files

- `crates/client/src/skills_page/mod.rs`: `SkillsPagePlugin`.
- `skills_page/spawn.rs`: `spawn_page` (Startup): the layer (depth 0), title row (`PageButton::{ZoomOut, ZoomIn, Fit, Close}`), the `Canvas` (presses on it are left to the page), its `Origin` (the middle, which nodes are placed from) and the `Details` line. `FORM` (`skills`).
- `skills_page/layout.rs`: `layout(skills)` (pure, unit tested): `Placed { id, at, from }` for each known skill, `RINGS`, `MAX_SLICE`.
- `skills_page/view.rs`: `SkillsView` (resource: pan, zoom, chosen skill, press and drag, `fit_wanted`), `zoom_about`, `fit`, `on_page`. Unit tested.
- `skills_page/build.rs`: `build_tree` (draws the nodes and lines again when what is known or the name changes, while open), `SkillNode`, `SkillLine`, `NodeLabel`, `tip`, `tenths`.
- `skills_page/place.rs`: `place_tree`: positions from the view each frame while open; lines are thin nodes turned by `UiTransform`; deeper names hide when zoomed far out; the chosen node's edge is orange.
- `skills_page/input.rs`: `take_input` (`PreUpdate`, `WidgetSystems::Modal`).
- `skills_page/details.rs`: `show_details`, `describe` (unit tested).
- `skills_page/icons.rs`: `icon_path(id)`: each skill's game-icons.net icon from a table by path (`ICONS`), or the nearest above it that has one, so single weapons and kinds show their type's unless they have their own; every icon is listed in `xtask/icons.txt` (a test checks every skill has a drawn one). The owner may change any.
- `skills_page/name.rs`: `OwnName`, `read_name`: the character's name from `Welcome`, for the middle.

## Public API

- `SkillsPagePlugin`. It is opened through `common::SkillsPageOpen` (the Abilities page's Skills button sets it) or `Action::Skills` (K).

## Depends on

- Assets: `icons/` (game-icons.net, drawn by `cargo xtask icons`).
- `common` (`SkillsPageOpen`, `Action`, `Preferences`, `UiPointer`, `TouchGesture`, `in_interface`, `GameView`), `character` (`CharacterSheet`), `connection` (`FromServer`), `debug_menu` (`DebugMenu`), `minimap` (`FullMap`), `system_menu` (`SystemMenu`): it leaves the input to those while they are open; `widgets`, `shared::skill`.

## Used by

- Nothing directly; `abilities` and `system_menu` through `common::SkillsPageOpen`.

## Notes

- Layout (owner, 2026-11-06: only known skills, learned by doing): the character's name in the middle, a ring per level (`RINGS`). The known top levels go clockwise from the top in the tree's order, sharing the circle by the square root of how many known skills end under each, so a small branch has room beside a big one; below them each node gets a share of its parent's slice by the ends under it, and a node's children spread no wider than its level allows (`MAX_SLICE`: 90°, 72°, 60°) round its own line, so every branch points outwards, even a lone one (owner, 2026-11-07: a new character's lone Athletics sat at the bottom with its children level with the middle). Tried first and dropped: a fixed ninth for each top level, which crowded everything known into a few slices.
- Zooming spreads the nodes apart rather than scaling them, so writing stays sharp; zoom 0.3 to 3, fitted on opening (never past 1) and by Fit or Home.
- Input while open: a drag on the canvas (mouse or finger) moves the tree, the wheel or a pinch zooms about the pointer or the fingers, a click or tap within a node (plus `NODE_SLOP` for fingers) chooses it and anywhere else chooses nothing, arrow keys move it, + and - zoom. K, Esc, the close button or a press outside close it; Esc is left to it by the system menu. It then takes every key, the wheel, gestures and the left button's presses, so nothing behind it moves. Nodes aren't widgets that take presses: the page hit-tests them itself, so a drag may start on one.
- Ids: `skills/zoom_out`, `zoom_in`, `fit`, `close`, `you` (the middle), `node.<number>` (each skill by `SkillId::number`).
- Each node shows its icon in ink (the white icons tinted), the middle none.
- Hovering a node names it with what a check with it counts; the line under the tree says the chosen skill's path, that total, its own value, its practice and the attributes it leans on.
- Not yet: rings showing progress towards a cap (none exists until specialising), and the gamepad.
