# client: character_create

The screen for making a new character: name, kind, gender and its looks, and the three triangles choosing the nine attributes, each with a two-layer radar chart. Shown while `common::MakingCharacter` is set ("+" on Character Select); it sends `CreateCharacter`, or on Cancel `common::CharacterChoice::CancelNew`. Choosing between characters is `character_select`'s.

## Files

- `crates/client/src/character_create/mod.rs`: `CharacterCreatePlugin`, its systems' order.
- `draft.rs`: `Draft` (resource: name, kind, gender (Neutral to begin with), appearance, `Build`, the triangle keys move), `playable`, `attributes` (`Attributes::for_kind`, as the server works it out), `bonus` (the kind's and the gender's, for the gold layer), `choose_kind` (looks start again), `choose_gender`, `ready` (a name `shared::name::checked` takes), `message` (sends the checked name).
- `screen.rs`: the screen as markup (`dom`: `CHARACTER_CREATE`): a full-screen `modal` dialog, depth 0, hidden, its two halves in the middle: the choices on the left (name with the random name button, kind, the gender's toggle row, a canvas for the kind's looks, a canvas for the difficulty's dropdown, what the attributes come to, Begin and Cancel side by side) and the 480-pixel triangles' canvas on the right, Reset and Random above it at the right; and the Ironman requester as a second dialog (`ironman`, depth 1). `bind_screen` gives each part its marker, puts `LooksBox` and `DifficultyBox` in their canvases, and makes the triangles' canvas the `Chooser` (its picture, `Pressable`, the corners' names). Markers, `FORM`.
- `looks.rs`: `spawn_looks` (only the looks the kind and gender have, `Trait::applies`: no Beard list for a female, made again on a new gender; a `widgets` colour picker for each trait whose choices are all colours, its label saying the chosen one by `show_look_labels`; a dropdown for the rest: height, hair style and beard), `PatternPictures` (each pattern's picture painted once: specks seeded, the same every time), `LookLabel`.
- `form.rs`: `show_screen` (shown by `MakingCharacter` through `Dom`, a fresh `Draft`, the kinds given to the dropdown (`Dom::set_options`) and the name box focused each time; with `common::CharacterRefused` set, the draft and every box kept, the reason in red under the name (`name_refused`, the box's red edge), put away once the name is typed in), `read_choices` (the name box, folded by `shared::name::fold` and cut to 24 as typed, kind list, look lists and colour pickers), `fill_looks` (the kind's look lists, again on a new kind), `press_begin` (Begin, disabled without a name, sends and clears `MakingCharacter`; Cancel sends `CharacterChoice::CancelNew`, back to Character Select or the startup menu), `LookList`.
- `geometry.rs`: the chooser's units (a 0 to 1 square, y down): triangle middles by row and turn (Body at the top at 0, clockwise, as `shared::sheet::Build::points` counts), corners always the same way up (power at the top, finesse lower right, resistance lower left, owner 2026-11-06), the gaps between triangles (`gaps`), dots to places and back (`dot_at` clamps to the triangle and keeps the weights adding up), turning, charts, distances to shapes.
- `picture.rs`: `paint` draws the picture pixel by pixel (`PIXELS` 640 square): the ring the triangles turn on, in a button's colour, with a curved two-headed arrow in each gap just inside it (`picture/ring.rs`), then each triangle, its axes, the chart of what it gives in the row's colour, the kind's bonus over it in gold, the outline (thicker for the triangle the keys move) and the dot as a ring. Each row's colour is `widgets::row_rgb`, shared with abilities' backgrounds.
- `difficulty.rs`: the Difficulty dropdown (`DifficultyBox` holder after the looks, `fill_difficulty` makes it again when `PlayDifficulty::allowed` or the choice changes; disabled with only one), `choose_difficulty` (into `Draft::difficulty`; Ironman only after the requester's yes, `spawn_ironman_requester`, a dialog above the screen). `show_screen` starts the draft on `Preferences::difficulty`, or the nearest allowed.
- `gender.rs`: `choose_gender` (the toggle row's `UiEvent` into the draft by `Draft::choose_gender`, which drops a beard for Female; `show_screen` sets it back to Neutral each time).
- `buttons.rs`: `press_triangle_buttons`: Reset (every dot to `Dot::CENTRE`) and Random (`random_dot`: anywhere in its triangle, evenly, weights adding up) above the triangles' canvas; the turn stays.
- `name.rs`: `NameRolls` (seeded from the clock once), `suggest_names`: a name for the kind (`CreatureKind::random_name`) when the screen opens (not when it opens again after a refusal), on another kind while the name is still the last suggested (`Draft::suggested`), and on the random button beside the name box.
- `input.rs`: `Holding`; `point_at_triangles` (a press near a triangle holds its dot, anywhere else turns them all by how far the pointer goes round; held until the mouse and fingers let go; a finger slide from the picture starts one too, as panels take slides), `keys_for_triangles` (1/2/3, arrows, Q/E; not while a text box has the keys).
- `show.rs`: `draw_triangles`, `place_labels` (three-letter names straight above the top corner and below the lower ones, so the two lower corners meeting between triangles stay apart, moved as they turn), `show_values` (each row's attributes and points as words).

## Public API

- `CharacterCreatePlugin`. Nothing else: others talk to it through `common::MakingCharacter`.

## Depends on

- `dom` (`UiEvent`, `UiValue`, `Dom`), `shared::gender`, `common` (`MakingCharacter`, `CharacterRefused`, `CharacterChoice`, `PlayDifficulty`, `Preferences`, `difficulty_words`, `IRONMAN_QUESTION`, `UiPointer`, `TouchGesture`, `in_interface`), `connection` (`ToServer`), `widgets` (layer, text box, dropdowns, button, `TextFocus`, `UiId`), `shared::sheet`, `shared::creature`, `shared::protocol`.

## Used by

- Nothing directly. `connection` sets `MakingCharacter` on `CharacterChoice::New` (clearing `CharacterRefused`), and again with `CharacterRefused` on `ServerMessage::CharacterRefused` (from Character Select, which opens on first joining a server or from the startup menu's Characters); `loading_screen` leaves the input and hides its bar while it is set.

## Notes

- Widget ids: `character_create/name`, `/random_name`, `/kind`, `/gender` (a `toggle-row`: Male, Female, Neutral), `/look.<n>` (a colour picker or a dropdown), `/name_refused` (the reason a name was turned down), `/difficulty`, `/ironman_yes`, `/reset_triangles`, `/random_triangles`, `/ironman_no`, `/begin`, `/cancel`, `/triangles` (the picture, a `Pressable`).
- The bonus layer is the kind's bonus and the gender's together (`Draft::bonus`), redrawn when either changes: the shape round the three bonus points (each on its axis, a bonus of 0 at the middle) grown by a small radius, so it is always at least a round dot and only real bonuses push it out (owner, 2026-11-06). The dot is a ring so a dot in the middle doesn't hide it.
- Charts reach their corners at `SHOWN_MOST` (250), the same for every kind, so kinds can be compared.
- Drawing uses floats: it is only for showing. What is sent is the integer `Build`, and the server works out the attributes itself.
- Mouse dragging and turning are checked with the debug API (`client.drag`); fingers follow the same code but haven't been tried on a touch screen.
