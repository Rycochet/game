# client: character_create

The screen for making a new character: name, kind and its looks, and the three triangles choosing the nine attributes, each with a two-layer radar chart. Shown while the server waits for one (`common::MakingCharacter`); it sends `CreateCharacter` and nothing else. Choosing between characters is not here (later, before this screen).

## Files

- `crates/client/src/character_create/mod.rs`: `CharacterCreatePlugin`, its systems' order.
- `draft.rs`: `Draft` (resource: name, kind, appearance, `Build`, the triangle keys move), `playable`, `attributes` (`Attributes::for_kind`, as the server works it out), `choose_kind` (looks start again), `ready` (has a name), `message`.
- `screen.rs`: `spawn_screen` (at start, hidden: a full-screen layer, depth 0, choices on the left and the 480-pixel picture on the right), markers, `FORM`.
- `looks.rs`: `spawn_looks` (a `widgets` colour picker for each trait whose choices are all colours, its label saying the chosen one by `show_look_labels`; a dropdown for the rest, so far height), `PatternPictures` (each pattern's picture painted once: specks seeded, the same every time), `LookLabel`.
- `form.rs`: `show_screen` (shown by `MakingCharacter`, a fresh `Draft` and the name box focused each time), `read_choices` (the name box, kind list, look lists and colour pickers), `fill_looks` (the kind's look lists, again on a new kind), `press_begin` (disabled without a name; sends and clears `MakingCharacter`), `LookList`.
- `geometry.rs`: the chooser's units (a 0 to 1 square, y down): triangle middles by row and turn (Body at the top at 0, clockwise, as `shared::sheet::Build::points` counts), corners always the same way up (power at the top, finesse lower right, resistance lower left, owner 2026-11-06), the gaps between triangles (`gaps`), dots to places and back (`dot_at` clamps to the triangle and keeps the weights adding up), turning, charts, distances to shapes.
- `picture.rs`: `paint` draws the picture pixel by pixel (`PIXELS` 640 square): the ring the triangles turn on, in a button's colour, with a curved two-headed arrow in each gap just inside it (`picture/ring.rs`), then each triangle, its axes, the chart of what it gives in the row's colour, the kind's bonus over it in gold, the outline (thicker for the triangle the keys move) and the dot as a ring.
- `name.rs`: `NameRolls` (seeded from the clock once), `suggest_names`: a name for the kind (`CreatureKind::random_name`) when the screen opens, on another kind while the name is still the last suggested (`Draft::suggested`), and on the random button beside the name box.
- `input.rs`: `Holding`; `point_at_triangles` (a press near a triangle holds its dot, anywhere else turns them all by how far the pointer goes round; held until the mouse and fingers let go; a finger slide from the picture starts one too, as panels take slides), `keys_for_triangles` (1/2/3, arrows, Q/E; not while a text box has the keys).
- `show.rs`: `draw_triangles`, `place_labels` (three-letter names straight above the top corner and below the lower ones, so the two lower corners meeting between triangles stay apart, moved as they turn), `show_values` (each row's attributes and points as words).

## Public API

- `CharacterCreatePlugin`. Nothing else: others talk to it through `common::MakingCharacter`.

## Depends on

- `common` (`MakingCharacter`, `UiPointer`, `TouchGesture`, `in_interface`), `connection` (`ToServer`), `widgets` (layer, text box, dropdowns, button, `TextFocus`, `UiId`), `shared::sheet`, `shared::creature`, `shared::protocol`.

## Used by

- Nothing directly. `connection` sets `MakingCharacter` on `Characters` with none to choose (on first joining, or after the debug menu's Delete Character, when the screen opens over the game and `player` takes no steps meanwhile); `loading_screen` leaves the input and hides its bar while it is set.

## Notes

- Widget ids: `character_create/name`, `/random_name`, `/kind`, `/look.<n>` (a colour picker or a dropdown), `/begin`, `/triangles` (the picture, a `Pressable`).
- The bonus layer is the shape round the three bonus points (each on its axis, a bonus of 0 at the middle) grown by a small radius, so it is always at least a round dot and only real bonuses push it out (owner, 2026-11-06). The dot is a ring so a dot in the middle doesn't hide it.
- Charts reach their corners at `SHOWN_MOST` (250), the same for every kind, so kinds can be compared.
- Drawing uses floats: it is only for showing. What is sent is the integer `Build`, and the server works out the attributes itself.
- Mouse dragging and turning are checked with the debug API (`client.drag`); fingers follow the same code but haven't been tried on a touch screen.
