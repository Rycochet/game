# client: widgets

The interface pieces, so everything looks alike. Use these rather than styling Bevy's `Node`s directly. Not what any screen shows or does: that stays with the module using them.

## Files

- `crates/client/src/widgets/mod.rs`: `WidgetsPlugin`, colours and sizes, `ButtonState`, `raised_box`, `panel`, `panel_title`, `spawn_tooltip`, `spawn_button`, `spawn_icon_button`, `spawn_check_button`, `spawn_close_button`, `show_button_states` (restyles buttons when their state changes).
- `crates/client/src/widgets/moveable.rs`: `spawn_moveable_panel`, `place_panels` (keeps them on the screen), `take_presses` (`PreUpdate`, takes clicks and touches for widgets). Unit tests.

## Public API

- `WidgetsPlugin`; `WidgetSystems` (system sets, `Presses` then `Modal`): anything covering the screen reads presses in `Modal`; anything else taking clicks in `PreUpdate` runs after both.
- `INK`, `PARCHMENT`, `CLOSE_PIXELS`, `BUTTON_PADDING`.
- `panel()`, `panel_title(text)`: a parchment panel, placed by absolute position, and the writing for its title.
- `raised_box()`, `spawn_tooltip(holder, marker)`: a box standing up from what is behind it; a named box, with `marker` on its text.
- `spawn_button(parent, state, label, font_size, node)`, `ButtonState` (`Normal`, `Selected`, `Active`, `Disabled`, `DisabledSelected`; a component, change it to restyle), `spawn_icon_button(parent, state, icon, node)`, `spawn_check_button(parent, state, checked, label, font_size, node)` (`None` leaves the checkbox's space, so words line up), `spawn_close_button(parent, node)`.
- `Pressable` (put it on a button): clicks on it become `ButtonPressed(entity)`, unless disabled, and go no further.
- `spawn_moveable_panel(commands, title, at)`: returns the panel and the entity to put its contents in. It fits its contents, is moved by its title bar and despawned by its close button; a press on it brings it to the front, as does `RaisePanel(entity)`.
- `stops_presses(entity)`: put it on a node for presses on it to go no further.

## Depends on

- `common` (`PointerPosition`, `PointerSystems`, `TouchGesture`).

## Used by

- `minimap`, `debug_menu`.

## Notes

- By touch: a tap arrives as a click (`touch`), so buttons and close buttons just work. A title bar moves by `TouchGesture::drag`, using where the finger landed (`drag_start`), since by the time a drag is known the finger may have left a thin bar. A drag starting anywhere else on a panel is taken and does nothing.
- Close buttons on moveable panels have a margin that also counts as on them, for fingers.
- The minimap's close and floor buttons look like widgets but keep their own hit tests from its layout constants (and the tests built on them), so they aren't `Pressable`.
- Layers: maps 10, warning button 15, full map 20 to 25, moveable panels from 30 up, debug menu above those, loading screen above all.
- Only the front-most thing under the pointer gets a press: a button under another panel gets nothing.
