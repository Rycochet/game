# client: dom

The whole interface is markup: HTML's look with XML's closing rule, no CSS and no scripts. A template is checked strictly, built into Bevy UI nodes, and changed by id. Every screen, panel, dialog and piece over the game's view is a document built here; what the player does comes back by id. Input follows the browser's rules: one search finds what the pointer is on, and only that reacts. The old `widgets` module is now the toolkit underneath: it draws and runs each control inside its element, and holds item pictures, shaders and the interface size (`widgets.md`). The plan this came from, and why: `docs/context/html-interface.md`. How each piece should look: `docs/context/ui-design.md`.

## Writing a screen

1. Write its template with `html!("name", r#"…"#)` (or `include_str!` of an `.html` file beside the module) as a `static`, and give the module a test calling `Template::assert_good()` on it.
2. Spawn `document(&TEMPLATE)` where it goes: as a child of a node, on `top_strip()` / `bottom_strip()` for pieces over the game's view, or on `top_strip()` alone for a document that is only a `panel moveable` or a `dialog` (they stand apart, in their own layers).
3. Change it by id through `Dom` (`set_text`, `set`, `set_flag("hidden" | "disabled", …)`), only when something changes.
4. Read `UiEvent { id, value }` in `Update`, matching on the id. Code that already knew its widgets by components can instead give the built widgets those components by id, in a system on `Added<UiId>` (`system_menu/requesters.rs`): presses then arrive as `ButtonPressed` in `PreUpdate`, a frame earlier than `UiEvent`.
5. For what draws itself, write a `canvas` (its owner fills it with nodes), or the game's own `object`, `object-grid`, `object-slot` (told what to show by a component). Despawn the document when the screen goes.

## Files

- `crates/client/src/dom/mod.rs`: `DomPlugin` (`PostUpdate`, before layout: `clear_surfaces`, `build_documents`, `apply_changes`, `tidy_surfaces`, then restyling, controls, surfaces, objects and `DrawObjects`), the re-exports.
- `crates/client/src/dom/template.rs`: `Template { name, source }`, the `html!` macro, `checked()`, `assert_good()` (tests only).
- `crates/client/src/dom/error.rs`: `MarkupError` (`at`, `what`), `Place` (line and column, from 1).
- `crates/client/src/dom/markup.rs`: markup as written (`RawNode`, `RawElement`, `RawAttr`). In `markup/`: `cursor.rs`, `parse.rs` (`parse`), `text.rs` (`tidy`: spaces as HTML's; `decode`: `&amp;` and the like).
- `crates/client/src/dom/tag.rs`: `Tag`, what each may hold, where it may go, which need an `id`, their own attributes.
- `crates/client/src/dom/attrs.rs`: `Attrs` (common attributes as fields; the rest kept as written, read with `text`, `number`, `has`), `is_known`. In `attrs/`: `values.rs` (`Length`, `Align`, `VAlign`), `own.rs` (`OWN`: each element's own attributes and their values), `set.rs` (`Attrs::set`, `clear`: the one path for markup and code), `write.rs` (tests and the debug API: attributes back as markup).
- `crates/client/src/dom/check.rs`: `check`: raw markup into `Checked` / `CheckedNode`, with ids, forms, `for` and where elements may go.
- `crates/client/src/dom/theme.rs`: `Theme` (resource), `Look`. `theme/over_view.rs`: looks over the game's view or a dark backdrop (the loading bar, the hover box, the status text, popups).
- `crates/client/src/dom/layout.rs`: `node_for`: an element's `Node` from its attributes, look and place in a table.
- `crates/client/src/dom/parts.rs`: `Document`, `DocumentOf`, `document()`, `document_in()`, `Element`, `Inert`, `Canvas`, `Solid`, `Opaque`, `AlwaysBars`, `Hovered`; private `TextStyle`, `Words`, `InTable`, `Bar`, `TemplateBody`.
- `crates/client/src/dom/build.rs`: `Builder`, `build_documents`. In `build/`: `table.rs`, `progress.rs`, `surfaces.rs` (`panel`, `dialog`, `footer`, `popup`, `scroll`), `title_row.rs` (a big dialog's title and close button), `container.rs`.
- `crates/client/src/dom/surfaces.rs`: `follow_surfaces` (a panel's or dialog's `hidden`; a moving panel's `title` and first place; a popup's `hidden`, `left`, `top`), `clear_surfaces`, `tidy_surfaces`.
- `crates/client/src/dom/popup.rs`: `Popup` (`locked`), `spawn_popup`, `press_locks`, `close_popups` (set `ClosePopups`); `popup/drag.rs`: `drag_popups`; `popup/hover.rs`: `KeptOpen`, `close_hovers` (a `hover` popup's closing by itself). Tests.
- `crates/client/src/dom/strip.rs`: `top_strip()`, `bottom_strip()`.
- `crates/client/src/dom/restyle.rs`: `restyle`: `Node`, background, tooltip, picture and bar of every new or changed `Element`.
- `crates/client/src/dom/help_mark.rs`: the "?" after the label of every `span.labelled` row with a non-empty `HoverTip` (from `title` or code; `place_help_marks`, after `restyle`, adds and removes it), drawn from `icons/sbed/help.png`; a row built by code names its label with `HelpAfter`, and gets the mark as that label's next sibling. Pointing at it shows the row's tip (it is in the row); `help_mark/pin.rs`: `pin_tips` (`PreUpdate`, between `DomInput::Find` and `Presses`) pins the tip beside it on a click or tap (`widgets::PinnedTip`), and lets it go on any other press or key, or a second press on it. Its own click goes no further, so it never presses the label's control. Tests.
- `crates/client/src/dom/pictures.rs`: `Pictures` (resource), `picture_for`.
- `crates/client/src/dom/change.rs`: `Dom`, `DomChange`. `change/apply.rs`: `apply_changes` (exclusive). `change/options.rs`: options of a `select`, tab bar or toggle row, and a `select`'s options' pictures. `change/rows.rs`: template rows copied into a table.
- `crates/client/src/dom/controls.rs`: `Builder::control` (the widget inside a control's element), `add`. In `controls/`: `buttons.rs`, `fields.rs` (text box, slider, key box), `choices.rs` (`select`, `toggle-row`, `tab-bar`, `colour-picker`), `event.rs` (`UiEvent`, `UiValue`), `bridge.rs` (`report`: widgets' messages into `UiEvent`s), `sync.rs` (`follow_elements`: attributes into widgets), `label.rs` (a click on a label; `fade_labels`: a label's writing faint while its control is disabled).
- `crates/client/src/dom/input.rs`: `DomInput`, `add`. In `input/`: `hit.rs` (`Hit`, `PointerCapture`, `Finder`, `find_target`, `find_now` for tests), `hover.rs` (`follow_hover`), `events.rs`, `dispatch.rs` (`send_events`).
- `crates/client/src/dom/objects.rs`: `show_objects`: each `object` drawn again when what is known of it changes (items from `common::KnownItems`, abilities from `KnownAbilities`).
- `crates/client/src/dom/object_grid.rs`: `GridShown`, `GridThing`, `GridAim`, `GridBox`, `ThingCell`, set `DrawObjects`, `draw_object_grids`.
- `crates/client/src/dom/object_slot.rs`: `SlotShown`, `SlotThing`, `SlotLook`, `draw_object_slots`.
- `crates/client/src/dom/object_ability.rs`: `AbilityThing`, `KnownAbilities`, `AbilityGridShown`, `AbilitySlotShown`, `AbilityCell`, `SlotPart`, `draw_ability_grids`; `object_ability/slot.rs`: `draw_ability_slots`.
- `crates/client/src/dom/hold.rs`: `HeldObject`, `HeldThing`, `HeldWhat`, `HeldItem`, `Hand`, `HeldOver`, `Dropped`, set `HoldObjects`. In `hold/`: `follow.rs` (`follow_held`, `is_click`), `show.rs` (`show_held`).
- `crates/client/src/dom/debug_api.rs` (debug API only): `client.ui.at`, `client.ui.html`.

## Public API

Documents and changes:

- `html!`, `Template` (`assert_good()` in every module's tests).
- `document(&TEMPLATE)`: the bundle to spawn; `Document(&Template)` is on its root. With `DocumentOf(id)` too, it is one of several alike (a bag's, a shop's) and its ids are `form#<id>/name` (`UiId::which` reads it back). `document_in(&TEMPLATE, element)` builds one inside another document's element, taking its room (Settings' Graphics page).
- `top_strip()`, `bottom_strip()`: as wide as the window along that edge, no height; the document's pieces are placed from it.
- `Dom` (system parameter): `set_text`, `set(id, name, value)`, `set_flag(id, name, on)`, `clear`, `raise(id)` (a moving panel to the front), `start_at(id, at)` (where a just-spawned moving panel first goes), `copy_template(template, into, index)` (rows into a `table`), `empty(id)`, `set_options(id, pairs)` (a `select`, `tab-bar` or `toggle-row`; never none; set `value` after), `set_option_pictures(id, pictures)` (a `select`'s options each a picture before its words, on it and in its list, a line high and as wide as its shape: the Quest Log's planets' runes; after `set_options`, which clears them). Ids are `form/name`, and `form/name.<index>` inside a copy. Changes are made in order at the end of the frame, after new documents are built; one that can't be made is logged and changes nothing.
- `Theme`: `look(tag, class)`, `set`, `has`, `row(size)`, `box_row(size)`, fields `rows`, `boxes`, `footer_button`, `progress`; `Theme::parchment()` is the game's. `Look`: text size, colour, margin, gap, background, `tip`, `icons`, `bar`.
- `Pictures::name(name, handle)`: a picture made in code, for `<img src="picture:<name>"/>`.
- `Element { tag }` (on every element), `Canvas` (on a `canvas`, `object-grid`, `object-slot`), `Inert` (skipped by the search, with all it holds), `Tag`, `Attrs`, `Length`, `Align`, `VAlign`, `Checked`, `CheckedNode`, `MarkupError`, `Place`.

Controls and events:

- `UiEvent { id, value }` (message, read in `Update`). `UiValue`: `Pressed`, `Flag(bool)`, `Text(String)`, `Number(f32)` (between the slider's `min` and `max`), `Choice { index, value }`, `Key(Option<Binding>)`, `Dismissed` (a press outside a dialog, or its close button), `Closed` (a moving panel closed by its button, or a popup closing; its owner despawns the document).
- Changing a control from code: `dom.set(id, "value", …)`, `set_flag(id, "disabled" | "checked" | "selected", on)`, `set_text` for a button's words, `set(id, "class", "main")` / `clear(id, "class")`. A choice's `value` is its option's `value`, or its words.

Input:

- `DomInput` (system sets, `PreUpdate`, after `InputSystems` and `PointerSystems`, in this order): `Find` (`Hit` is worked out), `Presses` (controls and panels take presses: `ButtonPressed`), `Send` (events), `Front`, `Modal`. Anything covering the screen reads its presses and keys in `Modal` (or `Front`, to be ahead: the system menu); anything else taking clicks in `PreUpdate` runs after them all.
- `HelpAfter(label)`: on a row built by code with a `HoverTip`, for its "?" to go right after `label`, as markup's `span.labelled` rows get one (Server Options' rows fixed by the environment).
- `Hit` (resource): `target` (an element, or a `Solid` node), `chain` (the target, then what holds it, outwards), `at`; `first(pick)`. Whether the pointer is on the interface is `hit.target.is_some()`.
- `Finder` (system parameter): `target_at`, `chain_at`, `chain_of`, for another point (where a finger landed).
- `Solid`: a node that isn't an element but can be the target and hides what is behind it. Required by everything looked for under the pointer (`Pressable`, `ButtonState`, panel parts, `HoverTip`, `ItemName`, scroll areas and bars, writing with items, the grids, slots and their cells). Anything else (a count, a ripple, an aim's tint) is never what the pointer is on.
- `Opaque` (requires `Solid`): a surface a press doesn't bubble out of (a dialog's panel, an open list, a popup, View Planet's bar, the action bar's rows), so blank parchment is never a press on the darkening behind.
- `Hovered`: on each element of the chain, and nothing else.
- Events, sent to the target and up through what holds it until stopped (`event.propagate(false)`): `Press`, `Release`, `Click { count }`, `ContextMenu`, `Wheel { lines }`; `PointerEnter` / `PointerLeave` to one element. Watch with `.observe(|click: On<Click>| …)`.
- `PointerCapture(Option<Entity>)`: that entity is the target until the left button is let go.

Objects and holding:

- `GridShown` (`columns`, `rows`, `things`: `GridThing`s; `aim`: `GridAim`; `takes_gifts`) on an `object-grid` (or any node) draws it in `DrawObjects`; `GridBox` names its cells' node; each drawn thing's cell carries `ThingCell { grid, item, rotation, grip }`, for the owner to mark `.after(DrawObjects)`.
- `SlotShown` (`look`: `SlotLook::Worn` / `Kept`; `icon`; `thing`: `SlotThing`; `aim`) on an `object-slot`.
- `AbilityGridShown`, `AbilitySlotShown` (`thing`, `key`, `auto`, `on`, `queued`, `shade`, `aim`), `AbilityCell`, `SlotPart` (`On`, `Queued`, `Shade`); `KnownAbilities` (`set(name, AbilityThing)`) for `<object type="ability">`.
- `HeldObject` (resource): the one thing held, `pick_up(thing, picked)`, `put_back`, `keep_up`. `HeldThing` (`id`, `item` or `ability`, `rotation`, `grip`, `Hand`: `Mouse`, `Clicked`, `Finger`, `Kept`). `Dropped { thing, at, over }` (message), `HeldOver { front, cell }`. Owners pick up `.before(HoldObjects)` and read drops `.after` it.
- `ClosePopups` (set): a system despawning a popup on `Closed` goes `.after` it.
- `BuildDocuments` (set, `PostUpdate`): where documents are built and changes by id applied; a system spawning a document to be built this frame goes `.before` it.
- `KeptOpen(Vec<UiId>)` (resource): the `hover` popups whose subject the pointer is on, pushed by their owners each frame, read and emptied in `PreUpdate`.
- `Popup { locked, hover }`.
- For other modules' tests: `dom::test_app()`, `dom::test_find(app, id)`, `dom::find_now`.

## Elements

`div`, `span`, `table` / `tr` / `td` / `th`, `form`, `template`, `br`, `p`, `h1`, `h2`, `small`, `label`, `img`, `progress`, `hr`, `canvas`; controls `button`, `input` (`type="text"`, `"checkbox"`, `"range"`), `select`, `toggle-row`, `tab-bar`, `colour-picker` (each holding `option`s), `key-box`; surfaces `panel`, `dialog` (with `footer`), `popup`, `scroll`; the game's own `object`, `object-grid`, `object-slot`, `container` (with `header` and `footer`).

Attributes on all: `id`, `form`, `class`, `title` (a tooltip; a panel's or dialog's heading), `hidden`, `disabled`, `inert`, `nowrap`, `fills`, `align`, `valign`, `top`, `left`, `right`, `bottom`, `width`, `height`, `margin`, `margin-<side>`. Their own: `src`, `alt` (`img`); `value`, `max` (`progress`); `colspan` (`td`, `th`); `cellspacing`, `cellpadding` (`table`); `for` (`label`); `selected`, `hold` (`button`); `type`, `value`, `checked`, `readonly`, `label`, `switch`, `min`, `max`, `step`, `inputmode="numeric"`, `prefix`, `suffix`, `accepts-items`, `src` (`input`); `value`, `searchable` (`select`); `value`, `selected`, `label`, `src` (`option`); `value` (`toggle-row`, `tab-bar`, `colour-picker`, `key-box`); `moveable`, `grip` (`panel`); `hover` (`popup`); `modal`, `depth`, `closable` (`dialog`); `max-height`, `auto`, `scrollbars="always"` (`scroll`); `type`, `for` (`object`); `type`, `for`, `columns`, `rows`, `grows`, `places`, `stacks`, `compact` (`object-grid`); `accepts`, `for`, `key` (`object-slot`).

Classes the theme knows: `p.warning`, `p.faint`, `p.medium`, `p.refused`, `p.status`, `p.status-warning`, `table.striped`, `span.joined`, `span.labelled`, `span.columns`, `span.apart`, `span.tip`, `span.place`, `span.mark-yellow` / `-blue` / `-orange` / `-grey` (a quest marker's badge, `theme/marks.rs`, colours from `common`), `div.tight`, `div.joined`, `div.sections`, `div.groups`, `div.plain`, `div.rows` / `form.rows`, `small.joined`, `label.field`, `dialog.sections`, `button.main`, `.quiet`, `.danger`, `.folder`, `.small`, `select.small` (the icon dropdown), `input.error`, `key-box.warned`, `img.gold`, `progress.loading`.

Forms in use: `hud` (the corner buttons), `menu`, the requesters' (`quit`, `ironman`, `refused`, `slow_drawing`, `going_online`, `going_offline`, `locking_down`, `reopening`, `online_failed`), `settings`, `settings.<page>`, `settings.keep_graphics`, `credits`, `connect`, `connect.certificate`, `connect.forget`, `server_options`, `login`, `loading`, `character_select`, `character_create`, `debug.menu`, `debug.submenu`, `debug.<panel>`, `inventory`, `clothes`, `bag#<id>`, `ground`, `split`, `nearby`, `talk#<key>`, `quest_log`, `accounts`, `step_pad`, `notices`, `hover`, `status`, `vitals`, `minimap`, `full_map`; code-made ids also use `planet`, `abilities`, `skills`. A moving panel is `<form>/panel`.

## Still code

What the move to markup left, on purpose or for the owner:

- **`widgets::leaves_presses`** and the full map's darkening (`widgets::darkening`, in `minimap/spawn.rs`): a press there is left for the owner to read from the mouse. The Skills page's canvas and View Planet's layer use it too. Markup has no see-through dark over the window yet: **open, for the owner**.
- **The maps' own search for what the pointer is on** (`minimap/input.rs`, `hold.rs`, `floors::under`): the maps' markup is all `inert`, and their code works from the window's size with extra reach for fingers. Moving it to `Hit` changes how the maps feel: **open, for the owner**.
- **Control `spawn_*` functions called by code filling canvases and places the markup leaves**: Character Select's cells, Character Create's looks and difficulty, Server Options' difficulty and router rows and its copy button, View Planet's bar, the Skills page, the debug menu's grid, Give Item's preview and the split box (`spawn_item_cells`), the full map's close button, the Killed requester (`spawn_dialog`). They are the same functions `dom` shows controls with, so nothing waits on them.
- **The login corner's words**: only its place is markup. Its words hold the account's email address, and `client.ui.html` writes back whatever `dom` holds, so they are never given to `dom`.

## Depends on

- `common`: `UiPointer`, `in_interface`, `PointerSystems`, `Binding`, `KnownItems`.
- `widgets`: the widget inside each control and surface (`spawn_button`, `spawn_switch`, `TextBox`, `Dropdown`, `spawn_moveable_panel`, `spawn_grip_panel`, `spawn_dialog`, …) and their messages (`ButtonPressed`, …), `MoveablePanel::start_at`, `PanelTitleOf`, `RaisePanel`, `UiId`, `HoverTip`, `Progress`, `Spinner`, `item_text`, item pictures, `TextSize` and colours.

## Used by

Every module with interface: `system_menu`, `debug_menu`, `login`, `loading_screen`, `character_select`, `character_create`, `inventory`, `talk`, `quest_log`, `accounts_panel`, `abilities`, `character`, `step_pad`, `notices`, `hover`, `ui`, `vitals`, `minimap`; `widgets` (`Hit`, `Finder`, `Solid`, `Opaque`, `DomInput`, `GridShown`); `view3d` (`Hit`: the wheel zooms only on the view); `target` (`Hit`).

## Notes

Markup:

- Our own parser, not a library: the dialect is small and strict, ids must be `&'static str` slices of the source for `UiId`, and errors want a line and column.
- An `id` needs a form (`<form id>` round it, or `form="…"`). `form`, `template` and `canvas` must have an `id`; every control too, and an `input` its `type`. A template's ids are checked apart from the document's, and a `template` is never laid out.
- Words: runs of spaces become one, none at either end; the gap between things in a row is the look's, not a space.
- In a template with a colour (`value="#be3c32"`), write `r##"…"##`. A text box's `prefix` comes only from the template (`&'static str`).
- Not all of `Dom`, the events and the re-exports is used yet: the `#[allow]`s go when something does.

Layout (owner, 2026-11-11: automatic):

- Block elements (`div`, `form`) stack; the rest flow in a wrapping row, never wider than what holds them. `br` and `hr` take a whole line. `nowrap` keeps a row on one line, a filling control taking what the others leave.
- Spacing belongs to what holds things: stacked 8 apart, a row 4; `tight` 4, `sections` 24, `joined` touching. Nothing has a margin in the theme; `margin` is for the odd case.
- Sizes are the theme's: a control fills what it is stacked in and is its own size in a row, at least the theme's row tall for the writing round it (`Theme::rows`), an icon button a square of that, a footer's buttons at least `footer_button` wide, a text box as tall as a dropdown (`Theme::boxes`). `width` and `height` only for what is particular to one screen.
- `fills`: takes the room left in what it is stacked in. `align` also sets which side each line of words is against. `valign` is `top`, `middle` (a row's default), `bottom`, or `stretch` in a row.
- Tables: a grid whose rows and cells are all the table's children; a `tr` is a strip behind its cells, and hiding it hides them. Columns fit their contents; the first `fills` cell's column takes the rest.
- `restyle` is the one place a `Node` is made, so markup and `Dom` can't disagree. Changing `class` later doesn't change words already inside.
- `progress` is a bar or a spinner by whether the markup gives a `value`; that doesn't switch later.
- A top-level document is laid out by the node it is given. Never a form that `fills` the window over the view: being an element, it would take every press (#424's hover box did, until the strips). A piece that takes the pointer has no form round it, only its own `form="…"` (the vitals ball).

Controls and surfaces:

- A control is two entities: its element (layout, attributes) and the widget inside it (`Control` / `ControlOf`). The id is on the widget, so `client.ui.*` and code finding widgets by `UiId` work unchanged; `Dom` finds the element from it. Hide a control by its element, so its room goes too.
- A checkbox is just the box: its words go in a `label`, or the `label` attribute for the wide kind. `readonly`: pressing only says `Pressed`. A click on a label presses its control, or gives a text box the keyboard. A markup text box keeps the keyboard (`TextBox::takes_keys`) unless its owner reads Return, Esc or arrows itself.
- Toggle rows, tab bars and colour pickers hold `option`s, as a `select` does. A `key-box`'s `value` is a binding's saved name, or `Escape` (shown, never bound).
- A button holds words, or one `img` (32 across; `class="small"` for the small kind). `hold`: pressed only while held.
- `panel`: parchment where written; `moveable` (title bar, close button) or `grip` (a thin grip along its top), each standing apart from its document in the widgets' layers, with `SurfaceOf(document)`, despawned with it. A moving panel follows its `title` always and its `left` / `top` until first placed (both in `%`: a share of the window, `PanelPlace::Share`), so one of several alike is spawned, then titled and placed through `Dom` in the same system. Its element is always laid out; only its widget is hidden.
- `dialog`: a requester over everything, buttons in its `footer`; `modal` is a big layer of its layer's share (or its own `width` / `height`), its `title` in a row along the top, `closable` adding a close button (`<form>/close`). `depth` is the layer. A dialog's words are the menus' size.
- `scroll`: `max-height` or `height` (pixels, or `%` of the window's height), or `fills`; as wide as what holds it; `auto` scrolls by itself; `scrollbars="always"` (`AlwaysBars`).
- `popup`: a dark box in front of the panels at its `left` and `top`, taking presses. Esc closes any; a press elsewhere closes an unlocked one; the padlock at its top right locks it (`Popup::locked`), and a locked one drags by its top 24 pixels, unmarked (owner, 2026-11-11: the pointer will show it by context later). A new one is unlocked. Closing says `Closed` by its id. `hover`: about what the pointer rests on (`Popup::hover`): unlocked, it also closes once the pointer has been off it, and off its subject, for 0.3 s (`GRACE`, real time); its owner says the pointer is on its subject by pushing its box's id into `KeptOpen` each frame. No pointer at all (a finger lifted) keeps it, until a press elsewhere. Item popups (`item_popup`).
- `container`: the frame round a grid: `title` top left, `header` controls kept right, what it holds stacked, `footer` last with its things kept right (one with `fills` takes the left). `span class="place"` has a grid's colour behind it (a Nearby row, tinted by its owner while aimed at).

Input:

- The search drills down: a point not on an element isn't on anything in it. Nodes that aren't elements may hold things anywhere (the widgets do), so all inside them is tried and the one furthest in front wins. `inert` and hidden things are passed over with all they hold.
- Our own search, not `bevy_picking`'s: its UI backend tries every node, takes its own pointers rather than `UiPointer` (mouse or finger in interface pixels, which fake input moves too), and only tells markers apart for the whole interface.
- A left press on anything of the interface is taken from the mouse (`widgets`' `take_presses`, in `Presses`) unless the nearest panel part leaves it to its owner (`leaves_presses`). A press on an element is taken in `send_events` too, but a right press on a `canvas`, `object-grid` or `object-slot` is left for its owner (turning a held item).
- A press bubbles from the target to the nearest button, stopping at a panel part or an `Opaque` surface, so nothing covered reacts or lights up.
- A click goes to the nearest thing both the press and the letting go were on; a finger's tap arrives as a left click (`touch`).
- Every search for what is in front goes through `Hit` or `Finder`: hover tips, items' names, items in writing, what a held thing is over, grid and bar cells, hold buttons, scroll areas, and a finger on a slider or in a text box. Sliders and text boxes still work out where along themselves the pointer is.
- Not built: several pointers at once, the capture phase, focus moved by keys, a long press as `ContextMenu`, a canvas saying a point isn't on it.

Objects:

- `<object type="item" for="<uuid>"/>`: the item as a grid shows it, its popup opening on resting the pointer there (`item_popup`); an unknown one is asked of the server (`KnownItems`) and is empty until answered, or `?`. With no `for` it takes no room. `type="ability"`: the ability by name from `KnownAbilities` (filled by `abilities`), with an `AbilityCell`; the plan's `background-color`, `class="active"` and `"enabled"` aren't read.
- Items in text: any writing in markup with `${item:<uuid>}` in it shows the item among the words (`widgets::item_text`), its popup opening on resting the pointer there. Turning that off per element, and copying the text form, aren't built: nothing wants them.
- Grids and slots are places their owner fills, as canvases are. A grid draws its cells, things and aim again only as far as changed. The elements' attributes aren't read into `GridShown` and `SlotShown` yet. Abilities flow in rows, one place each.
- Holding knows no rules: `follow_held` (`PreUpdate`, `HoldObjects`) turns the thing (right press, a second finger's tap, `Action::TurnItem`), moves it, says what it is over and writes `Dropped` when its hand lets go; `show_held` draws it above everything, `Inert`. Each owner takes only its own kind from `Dropped`. The split box and a held finger's picking up stay `inventory`'s.
