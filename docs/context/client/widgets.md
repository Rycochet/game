# client: widgets

The interface pieces, so everything looks alike. Use these rather than styling Bevy's `Node`s directly. Not what any screen shows or does: that stays with the module using them.

## Files

- `crates/client/src/widgets/mod.rs`: `WidgetsPlugin`, colours and sizes, `ButtonState`, `raised_box`, `panel`, `panel_title`, `spawn_tooltip`, `spawn_button`, `spawn_icon_button`, `spawn_check_button`, `spawn_close_button`, `show_button_states` (restyles buttons when their state changes).
- `crates/client/src/widgets/fullscreen.rs`: `spawn_fullscreen_layer`, `darkening`, `leaves_presses`, `layer_z`, `layer_share`. Unit tests.
- `crates/client/src/widgets/text_box.rs`: `TextBox`, `TextFocus`, `TextEdited`, `SoftKeyPressed`, `spawn_text_box`, and the systems giving boxes the keyboard. Unit tests.
- `crates/client/src/widgets/soft_keyboard.rs`: in browsers, a hidden HTML input for the on-screen keyboard. Does nothing natively.
- `crates/client/src/widgets/dropdown.rs`: `Dropdown`, `DropdownChosen`, `spawn_dropdown`, and the open list. Unit tests.
- `crates/client/src/widgets/filter_list.rs`: `matches_filter`, `next_highlight`, `HighlightStep`, shared by the debug menu and dropdowns with search. Unit tests.
- `crates/client/src/widgets/slider.rs`: `Slider`, `SliderMoved`, `spawn_slider`, and the systems moving them. Unit tests.
- `crates/client/src/widgets/tabs.rs`: `spawn_tabs`, `TabChosen`, `press_tabs`. Unit test.
- `crates/client/src/widgets/moveable.rs`: `spawn_moveable_panel`, `place_panels` (keeps them on the screen), `take_presses` (`PreUpdate`, takes clicks and touches for widgets). Unit tests.

## Public API

- `WidgetsPlugin`; `WidgetSystems` (system sets, `Presses`, `Front`, then `Modal`): anything covering the screen reads presses in `Modal`, or `Front` to be ahead of those (the system menu); anything else taking clicks in `PreUpdate` runs after both.
- `INK`, `PARCHMENT`, `CLOSE_PIXELS`, `BUTTON_PADDING`.
- `panel()`, `panel_title(text)`: a parchment panel, placed by absolute position, and the writing for its title.
- `raised_box()`, `spawn_tooltip(holder, marker)`: a box standing up from what is behind it; a named box, with `marker` on its text.
- `spawn_button(parent, state, label, font_size, node)`, `ButtonState` (`Normal`, `Selected`, `Active`, `Disabled`, `DisabledSelected`; a component, change it to restyle), `spawn_icon_button(parent, state, icon, node)`, `spawn_check_button(parent, state, checked, label, font_size, node)` (`None` leaves the checkbox's space, so words line up), `spawn_close_button(parent, node)`.
- `Pressable` (put it on a button): clicks on it become `ButtonPressed(entity)`, unless disabled, and go no further.
- `spawn_moveable_panel(commands, title, at)`: returns the panel and the entity to put its contents in. It fits its contents, is moved by its title bar and despawned by its close button; a press on it brings it to the front, as does `RaisePanel(entity)`.
- `spawn_slider(parent, value, track, width)`: a track (a picture, or plain) with a mark along it. Values are `0..=1`. `SliderMoved(entity, value)` says the player moved it; add a marker to tell which.
- `spawn_text_box(parent, TextBox { text, prefix, digits_only, takes_keys }, font_size, node)`: a click or tap gives it the keyboard. `takes_keys` keeps all keys from the game while it has them (Return, Esc or a click elsewhere gives them up); off for a box in something that reads keys itself, which sets `TextFocus` to its box and reads `TextEdited(entity)` and `SoftKeyPressed`.
- `spawn_dropdown(parent, Dropdown { options, selected, searchable }, font_size, node)`: `DropdownChosen(entity, index)` says what the player chose. With `searchable`, the open list has a text box with the keyboard, narrowing it as letters are typed.
- `spawn_tabs(parent, labels, selected, font_size)`: a row of buttons of which one is chosen, like radio buttons; returns the row. `TabChosen(row, index)` says the player chose another.
- `stops_presses(entity)`: put it on a node for presses on it to go no further.
- `spawn_fullscreen_layer(commands, depth, content)`: a hidden, centred full-screen panel, 90% of the window for depth 0 and 75% for depth 1 (`layer_share`), darkening everything outside it. Returns the layer (show and hide it; a press outside the panel is its `ButtonPressed`, which should close only that layer) and the panel to fill. `LAYER_EDGE`.
- `spawn_sized_layer(commands, depth, width, height, content)`: the same with a panel of this size rather than its depth's (`Val::Auto` fits its contents).
- `darkening(depth)`, `leaves_presses(entity)`, `layer_z(depth)`: for a panel placing itself, as the full map does: the darkening alone, with presses on it hidden from widgets underneath but left for its owner to read from the mouse.

## Depends on

- `common` (`PointerPosition`, `PointerSystems`, `TouchGesture`).

## Used by

- `minimap`, `debug_menu`, `system_menu`.

## Notes

- Sliders by touch: a tap moves the mark there, and a drag starting on one moves it (`drag_sliders`, before `take_presses`, which would otherwise take a drag on a panel).
- Dropdowns: an open list takes all input, in `Presses` so nothing else sees it. Up and Down move the highlight, Return chooses it (or the first), Esc or a click outside closes it, and the wheel or a finger scrolls. Only a window of `ROWS` rows is drawn rather than clipping more, as a clipped node still takes presses here. Nothing in the game uses them yet: see Show Widgets (owner kept them when Generate POI became a submenu).
- On-screen keyboard: browsers only allow focusing an input during the tap, so a `pointerup` listener on the canvas focuses the hidden input if the tap is on a box, or a dropdown with search, whose places are sent each frame. Untested on a real iPad.
- By touch: a tap arrives as a click (`touch`), so buttons and close buttons just work. A title bar moves by `TouchGesture::drag`, using where the finger landed (`drag_start`), since by the time a drag is known the finger may have left a thin bar. A drag starting anywhere else on a panel is taken and does nothing.
- Close buttons on moveable panels have a margin that also counts as on them, for fingers.
- The minimap's close and floor buttons look like widgets but keep their own hit tests from its layout constants (and the tests built on them), so they aren't `Pressable`.
- Layers: maps 10, warning button 15, moveable panels from 30 up, full-screen layers from 1,000,000 (debug menu, full map), loading screen above all. Outside a layer's panel, each layer's darkening adds up (owner's design), so nothing under one can be used, whatever is behind it.
- Only the front-most thing under the pointer gets a press: a button under another panel gets nothing.
