# client: skills_page

The Skills page: the Character panel's Skills tab (owner, 2026-11-07: it was a full-screen layer), showing the skills the character knows as a radial tree round a plain circle for them, moved by dragging and zoomed by the wheel or a pinch, with a line about the chosen skill under it. Not the skills themselves (`shared::skill`) nor the panel and its tabs (`inventory`, which spawns the page as a `common::SkillsPage` node). Native and web.

## Files

- `crates/client/src/skills_page/mod.rs`: `SkillsPagePlugin`.
- `skills_page/spawn.rs`: `fill_page` (each new `common::SkillsPage`): a row of buttons (`PageButton::{ZoomOut, ZoomIn, Fit}`), the `Canvas` (640 × 480, so the panel fits it; presses on it are left to the page), its `Origin` (the middle, which nodes are placed from) and the `Details` line. `FORM` (`skills`).
- `skills_page/layout.rs`: `layout(skills)` (pure, unit tested): `Placed { id, at, from }` for each known skill, `RINGS`, `MAX_SLICE`, `SIZES` (each level's circle at zoom 1; 50% larger than at first, owner), `size_of`. `layout/rings.rs`: `radii`: each ring pushed out, inside out, until no two circles touch (`GAP` between them).
- `skills_page/view.rs`: `SkillsView` (resource: pan, zoom, `max_zoom`, chosen and hovered skill, press and drag, `fit_wanted`, `shown`: the tab laid out, which the other systems wait for), `zoom_about`, `fit`, `on_page`. Unit tested.
- `skills_page/build.rs`: `build_tree` (draws the nodes and lines again, while shown, only when which skills are known changes or the page is new: `Drawn`, unit tested), `SkillNode`, `SkillLine`, `RingOf` (a ring's skill), `ring_value`, `tenths`.
- `skills_page/values.rs`: `show_values`: keeps each `RingOf` ring's `SkillRing` up to date while open, changing only those that differ, so practice from every blow doesn't draw the tree again (that made the rings flicker and, with the ring widget, crashed).
- `skills_page/place.rs`: `place_tree`: places and sizes from the view each frame while open (circles grow with the zoom); lines are thin nodes turned by `UiTransform`; the hovered skill, or else the chosen one, gets an orange `Outline`, leaving the border for the coming ring.
- `skills_page/input.rs`: `take_input` (`PreUpdate`, after `dom::DomInput::Modal`): notes whether the tab is shown (its canvas has a size), and fits the tree each time it comes on show.
- `skills_page/details.rs`: `show_details`, `describe` (unit tested).
- `skills_page/icons.rs`: `icon_path(id)`: each skill's game-icons.net icon from a table by path (`ICONS`), or the nearest above it that has one, so single weapons and kinds show their type's unless they have their own; every icon is listed in `xtask/icons.txt` (a test checks every skill has a drawn one). The owner may change any.

## Public API

- `SkillsPagePlugin`. It fills `common::SkillsPage`; `inventory` opens the panel on that tab by `Action::Skills` (K).

## Depends on

- Assets: `icons/` (game-icons.net, drawn by `cargo xtask icons`).
- `common` (`SkillsPage`, `Action`, `Preferences`, `UiPointer`, `TouchGesture`, `in_interface`, `GameView`), `character` (`CharacterSheet`), `debug_menu` (`DebugMenu`), `minimap` (`FullMap`), `system_menu` (`SystemMenu`): it leaves the input to those while they are open; `widgets`, `shared::skill`.

## Used by

- Nothing directly; `inventory` spawns its page.

## Notes

- Layout (owner, 2026-11-06: only known skills, learned by doing): a plain circle for the character in the middle, a ring per level (`RINGS`). The known top levels go clockwise from the top in the tree's order, sharing the circle by the square root of how many known skills end under each, so a small branch has room beside a big one; below them each node gets a share of its parent's slice by the ends under it, and a node's children spread no wider than its level allows (`MAX_SLICE`: 90°, 72°, 60°) round its own line, so every branch points outwards, even a lone one (owner, 2026-11-07: a new character's lone Athletics sat at the bottom with its children level with the middle). Tried first and dropped: a fixed ninth for each top level, which crowded everything known into a few slices.
- Zooming grows the circles as much as the room between them (owner, 2026-11-07), and the rings are sized so no two circles touch at zoom 1, so they never touch at any zoom. Fitted on opening and by Fit or Home: the tree's own extent, circles included, centred and as big as fits (never past zoom 1), so a lopsided tree fills the page; zooming in stops once the smallest level's circles are twice the size the middle had when fitted (`max_zoom`, owner); out to 0.1.
- Input while shown, only over the canvas and in front there (a panel over it keeps its presses): a drag (mouse or finger) moves the tree, the wheel or a pinch zooms about the pointer or the fingers, a click or tap within a node (plus `NODE_SLOP` for fingers) chooses it and anywhere else on the canvas chooses nothing, the node under the pointer is hovered (not while dragging), arrow keys move it, + and - zoom, Home fits it; those keys, the wheel and gestures are then the page's, so nothing behind moves. Elsewhere everything is left alone. Nodes aren't widgets that take presses: the page hit-tests them itself, so a drag may start on one. The tree is fitted each time the tab comes on show, not when a skill is learned while it is (Fit does that).
- Ids: `skills/zoom_out`, `zoom_in`, `fit`, `you` (the middle), `node.<number>` (each skill by `SkillId::number`).
- Each skill's circle is coloured by the attributes it leans on (`widgets::attribute_colour` with `Shade::Skill`, between abilities' dark and pastel: skills serve both kinds of ability, owner) and shows its icon in white with a black outline (`widgets::spawn_outlined_icon`), as abilities do. No names under skills, and the middle is a plain circle with no name (owner, 2026-11-07).
- The line under the tree says the hovered skill, or else the chosen one: its path, its own value, what a check with it counts, its practice and the attributes it leans on. No hover tips.
- Each skill has a ring round the inside of its edge for its own value (`widgets::skill_ring`, full at 100, coloured by fifths Common to Legendary); a specialisation's second, thinner ring inside it waits for specialising to be built.
- The line under the tree leads with the skill's own value, then what a check with it counts: "Lore: 35.2 (35.2 in checks)".
- Not yet: the gamepad.
