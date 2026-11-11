# Plan: the interface as HTML

Status: **PRs 1 and 2 of 11 built** (markup and layout; one hit test and the events: `docs/context/client/dom.md`); the rest is the plan. Written 2026-11-10 with the owner, whose answers are at the end and in `docs/context/decisions.md`.

What PR 1 settled differently from the plan below:

- **The parser is our own**, not `quick-xml`: the dialect is small and strict, ids have to be `&'static str` slices of the source to become `UiId`s, and errors want a line and column. No new dependency.
- **Changes by id are queued** (`Dom`) and made in order at the end of the frame, after new documents are built, so a system can spawn a document and fill it at once.
- **Each module tests its own templates** (`Template::assert_good()`), rather than one registry of them all, so modules don't have to know each other's.
- **An `id` needs a form** (a `<form id>` round it, or `form=`), so every id is `form/name` as `UiId`s already are.
- **A `tr` is a strip behind its cells**, which are the table's own children (a grid only places its children).

What PR 2 settled:

- **The hit test is our own**, drilling down as the owner asked. `bevy_picking`'s UI backend was read first: it tries every node in the stack, uses its own pointers rather than the game's (`UiPointer`), and can only require markers for the whole interface.
- **Events are Bevy's entity events**, passed up from the target through what holds it (`On<Click>`, stopped with `propagate(false)`), rather than a message type of our own. Events by id for screens' own code come with the controls (PR 3).
- **The old widgets share the search** for what lights up and what is pressed; their sliders, text boxes, scroll areas and grids keep their own until they become elements.
- **Left for the PRs that need them:** several pointers at once, the capture phase, focus and keys (with the text input), drag and drop (with panels and items), a long press as a right click, and canvases with shapes of their own.

## Why

- **A mod API later.** Mods describe interface as markup and get events back by id, without touching Bevy entities or our Rust internals.
- **More abstract handling.** Screens say *what* they hold; one place decides how it looks (the theme) and how input reaches it (the event rules).
- **Nothing crossing where it shouldn't.** Today each kind of widget does its own hit test (`moveable/drag.rs::take_presses`, the hold buttons, sliders, dropdowns, the minimap's own, `Floating`, `stops_presses`, `leaves_presses`, `OverInterface`). They mostly agree, but a covered button can still show a hover look, and every new widget is a new chance to get it wrong. One hit test with the browser's rules removes the whole class of bug.

## What it is, in one paragraph

Screens are written as HTML with XML's closing rule, inline in Rust (`r#"..."#`) or in an `.html` file beside the module. A small strict parser turns it into a tree of elements; each element becomes a Bevy UI node, looked after by a new client module (working name `dom`). Looks come from a Rust theme table keyed by element (and `class`), not CSS. Code changes the tree through a small API by id and reads what the player did as events by id. Input follows the browser's Pointer Events and focus rules, so only the front-most thing under the pointer is hovered or pressed. Anything that must draw itself (the maps, the Skills tree, item grids) is a `<canvas>` or a custom element: the document gives it a place and its input, and its own Rust code draws.

## Rules of the markup

1. **HTML element names and attributes, where HTML has the thing.** A dropdown is `<select>`, a tooltip is `title="..."`, a disabled button is `<button disabled>`, a spinner is `<progress/>` with no value, as in a browser.
2. **XML's closing rule.** Every element is closed: `<div>…</div>`, or self-closed when it has nothing inside: `<br/>`, `<img src="icon:gear"/>`, `<input id="volume" type="range"/>`. `<br>` alone is an error.
3. **HTML's yes/no attributes.** `<button disabled>`, `<dialog modal>`, `<input checked/>` as in HTML (also `disabled="disabled"`). Strict XML forbids a bare attribute, so the parser is XML with this one HTML allowance.
4. **HTML-alike, not HTML.** Our own elements take short names and needn't follow HTML's hyphen rule for custom elements: `<object/>`, `<scroll>`, `<panel>`; a hyphen only where two words read better (`<key-box>`, `<object-grid>`). (Owner, 2026-11-10.)
5. **No CSS.** No `style`, no `<style>`, no selectors. Places and sizes are six attributes, `top`, `left`, `right`, `bottom`, `width`, `height` (interface pixels, or `%` of the parent), normally left out: elements size themselves from their contents and the theme. Any of them places the element in its parent: `left` and `right` together give its width, `right` alone keeps it against the right edge, and so on; a side told three ways (`left`, `right` and `width`) is an error. `align` and `valign` line up children, as in HTML 4. Spacing between elements is the theme's, but an element can override it with `margin`, `margin-top`, `margin-right`, `margin-bottom`, `margin-left` (`margin-right="0"`: the next button touches this one). Margins are multiples of 4, 0 included; anything else is a parse error.
6. **No scripts.** No `<script>`, no `onclick`. Markup only describes; behaviour is Rust (later the mod API). This is what makes it safe to load a mod's markup.
7. **Every input has an id.** An element that can be used (button, input, select, key box, ...) without an `id` is a parse error. The `id` is always the element's own name, and the only name events come back by and how code and the debug API find things. They are today's `UiId`s: `<form id="settings">` round inputs makes `settings/volume`, or an input outside it says `form="settings"`, as in HTML.
8. **Labels, and `for`, as in HTML.** `<label>` either wraps its control (`<label><input id="mute" type="checkbox"/> Mute</label>`) or names it (`<label for="volume">Volume</label>`). Clicking the label works the control; pointing at the label hovers the control too (browsers do this for `:hover`), so a whole check row lights up. Anything that stands for something else names it with `for` the same way: `<item id="sword" for="<item uuid>"/>`, its own `id` for events and `for` the item it shows.
9. **Strict.** Unknown elements, unknown attributes, a `for` naming no id, two inputs with one id in a document, an input without an id: all errors, with line and column, found by a unit test that parses every template in the codebase. Nothing is silently ignored.
10. **Whitespace as in HTML.** Runs of spaces and line breaks in text become one space, so templates can be indented freely.
11. **Markup is built in.** Templates are written inline (`r#"..."#`) or kept in `.html` files beside their module and pulled in when compiling (`include_str!`), so a build never reads interface files from disk. Mods' markup is a decision for the mod API, later.

## Elements

"Look" says whether it draws anything of its own. Everything not in these tables is an error until it is needed.

### Layout (draw nothing, no padding)

| Element | Does | Today |
| --- | --- | --- |
| `div` | A block: its children stack top to bottom. | `Node` with column flex |
| `span` | Inline: its children (text, buttons, pictures) flow left to right, wrapping. | `Node` with row flex |
| `table`, `tr`, `td`, `th` | Rows and columns that line up; `colspan`; `cellspacing`, `cellpadding` as HTML 4. `th` is header text. Settings and Controls rows. | hand-built rows, `ROW_TINT` |
| `form` | Names the inputs inside it (`form/name`). Draws nothing. | the form half of `UiId` |
| `template` | Never shown: a piece code copies to fill lists (`connect/server.2`). | loops in Rust |
| `br` | A line break in inline flow. | — |

### Text and pictures (theme gives size and spacing)

| Element | Does | Today |
| --- | --- | --- |
| `p` | A paragraph: medium text, with the theme's space round it. | `TextSize::Medium` |
| `h1`, `h2` | Title (24) and large (18) text. | `TextSize::Title`, `Large` |
| `small` | Small text (13). | `TextSize::Small` |
| `label` | See rule 8. | `field_label`, check rows |
| `img` | A picture: `src="icon:<name>"` (icon font), `src="picture:<name>"` (one drawn in code, such as the new-seed arrow), or an asset path. `alt` names it. | icons, `new_seed_icon` |
| `progress` | `value`/`max`: a bar with words over it. No `value`: the spinner. | `spawn_progress_bar`, `Spinner` |
| `hr` | A divider (ui-design.md lists it as later). | — |

### Controls (need an `id`)

| Element | Does | Today |
| --- | --- | --- |
| `button` | A button; text, an `img`, or both inside. `class="main"`, `"quiet"`, `"danger"`, `"folder"`, `"small"` for the theme's variants. | `spawn_button`, `spawn_icon_button`, `spawn_small_icon_button`, `ButtonStyle` |
| `input type="checkbox"` | A checkbox; with HTML's `switch` attribute (Safari already has it) a switch. `checked`. | `spawn_check_button`, `spawn_switch` |
| `input type="text"` | A text box. `value`, `maxlength`, `inputmode="numeric"` for digits only (real HTML). Our own extras: `accepts-items`, `prefix`, `suffix`. | `spawn_text_box` |
| `input type="range"` | A slider: `min`, `max`, `step`, `value`; `src` for a picture track. | `spawn_slider` |
| `input type="radio"` in a `fieldset` | Radios sharing a `name` in a `fieldset`, drawn as joined buttons. | `spawn_toggle_row` |
| `select`, `option` | A dropdown. `searchable`; `option disabled` is greyed; `option` may hold an `img`. `class="icon"`: the icon dropdown. | `spawn_dropdown`, `spawn_icon_dropdown` |
| `key-box` | Waits for a key; `value="Shift+KeyG"`. | `spawn_key_box` |
| `colour-picker` with `option`s | Swatches; each `option` has `value` (a colour) or `src` (a picture) and `label`. | `spawn_colour_picker` |
| `tab-bar` with `button`s | Folder tabs. | `spawn_tabs` |
| `button hold` | Pressed only while held. | `HoldButton` |

### Surfaces

| Element | Does | Today |
| --- | --- | --- |
| `dialog` | `modal`: a full-screen layer (darkening, 90% / 75%), everything outside it inert, as `showModal()` does. Without `modal`: a requester-sized panel with its buttons along the bottom. Esc closes the top one (HTML's `cancel`). | `spawn_fullscreen_layer`, `spawn_dialog`, `spawn_sized_layer` |
| `panel` | A parchment panel; `title`; `moveable` (stripes, dragged by its title, close button), `grip` (the thin grip). Placed by the six attributes (rule 5); the movement pad is `right="16" bottom="16"`. | `panel`, `spawn_moveable_panel`, `spawn_grip_panel` |
| `scroll` | `height` or `max-height`; scrolls by wheel, finger or bar; remembered by id. `auto` for the credits' slow scroll. Its bar shows only when there is more than fits, unless `scrollbars="always"` on it, or the player's new **Always Show Scroll Bars** switch (Settings, Interface; off by default) shows every bar, which also shows a developer what can scroll. | `spawn_scroll_area`, `AutoScroll` |

### Custom game elements

Each is a Rust type registered with the document: it builds its own insides (hidden from the markup, as a web component's shadow tree is), says which of its parts the pointer is on, and sends events under its own `id`. This is the route mods will extend later. What a drop or a press *means* stays with the module that owns the screen.

| Element | Does | Attributes | Today |
| --- | --- | --- | --- |
| `object` | One thing shown: `type="item"` or `type="ability"`, `for` naming which (an item's uuid, an ability). An item: its picture or cells, quality frame, glow, set edge, stack count, the mixed "!". An ability: its icon outlined in black over a background colour, square or round by the ability's own type (active or passive, so the markup never says), greyed when it can't be used now. Either is named on hovering, and is what items in text become. | `type`, `for`; for abilities `background-color` (American spelling; otherwise the colour of the attributes it leans on), `class="active"` (on: a toggle such as Sneak, or used automatically), `class="enabled"` (usable now: greying it doesn't stop it being dragged, so it isn't `disabled`) | `spawn_item_cells`, `ItemLook`, inline items in `item_text/`; `spawn_ability`, `AbilityLook`, `attribute_colour` |
| `object-slot` | A place holding one thing, shrunk to fit: the clothes panel, the held hands, the bin and Drop icons, the action bar's slots (the key's name in outlined words, the sliding metal edge when automatic, the queued stripes; right click or a held finger for automatic). Dragging on, between and off; a held thing it won't take shows red. A drop is a `drop` event saying what was held, of which type; the owner decides what it means. | `accepts` (`"item"`, `"ability"` or `"item ability"`), `for` (what is in it, if anything), `key` | `inventory`'s clothes and slots, `abilities`' bar |
| `object-grid` | Every grid of things: today's carried grids, `StockGrid` and the Abilities page's list all become one. Pick up, place, turn, split, double press to open a bag, green and red cells under a held item. A drop is a `drop` event with what was held and where; the owner decides whether that is a move, a gift, putting it on the ground or taking an ability off the bar. | `type`; `for` (the container, or whose abilities); `columns`; `rows` or `grows`; `places="kept"` (the container's own places, which the server keeps: inventory, bags) or `places="packed"` (a list laid out on show, oldest first, first fit, a drop landing where let go or in the first free place: shops, the ground); `stacks="false"` (shops); `compact` (every thing takes one place, whatever its shape: dev tools; always so for abilities, which have no shape) | `spawn_item_grid`, `inventory/panels/`, `StockGrid`, the Abilities page |
| `container` | The frame round a grid: title, controls at the top right, weight under it, slots at the bottom right. Could be plain markup instead; decide in PR 9. | `title` | `spawn_container` |
| `vitals` | The Health and Mana ball: red left, blue right, filling from the bottom, each half its own tip. | — | `vitals` |
| `character` | A character drawn to one scale, as in Character Select. Only if it stays a 3D drawing rather than an `img` of a picture made in code. | `for` (the character) | `character_select` |

The skill ring (how good a skill is) isn't an element: the Skills tree's canvas draws its own, as it zooms and takes the wheel itself. If rings are wanted anywhere else later, they are taken out into an element then (owner, 2026-11-10).

### Canvases

A `canvas` is a box the document places and sizes; its owner's code draws into it (Bevy nodes, an image, a camera's target) and gets its pointer events in local coordinates, deciding for itself what was hit. Each needs its `id`.

| Canvas | What its code does | Today |
| --- | --- | --- |
| The game view | World map, wilderness, site levels and the Hall, with everything in the 3D world: tokens, pois, doors, stairs, bags, cell clicks (travel, Teleport and Change Cell modes), targeting, gate picking, the camera's turn, tilt and zoom by mouse and two fingers, and the bars and signs drawn over tokens. One canvas, under everything: whatever no element above takes reaches it. | `world_map`, `dungeon_map`, `hall_view`, `view3d`, `target`, `ground`, `entities`, `touch` |
| Minimap | The round map: drag, the arrow to the player. Says its corners aren't on it. Its close and floor buttons become ordinary `button`s over it. | `minimap` |
| Full map | The M map: drag, zoom, floors. | `minimap` |
| View Planet | The turning globe and moons; its bar of choices is markup. | `planet_view` |
| Skills tree | Pan, zoom about the pointer or fingers, nodes picked by distance, the skill rings. | `skills_page` |
| Character creation's triangles | Dots dragged, the ring turned, keys. | `character_create` |

### Considered and left out

`section`, `article`, `nav`, `header`, `footer` (only meaning without CSS is to screen readers, not planned: use `div`); `ul`, `ol`, `li` (later, if we show lists of text); `a` (no links in game); `b`, `i`, `em`, `strong`, `font` (one font weight; colour is the theme's); `textarea` (later, for chat); `iframe`, `object`, `embed` (no); `marquee` (tempting for the credits, but `scroll auto` says it); `style`, `script` (rules 5 and 6).

## Global attributes

`id`, `class` (picks a theme variant, nothing else), `title` (tooltip after the usual wait; for buttons today's `ButtonTooltip`, for anything else `HoverTip`), `hidden` (`Display::None`, keeps its place in the tree), `disabled`, `inert` (not hit by the pointer at all and not focusable: today's `Floating`, for counts over items, outlined words and what follows the pointer), `align`, `valign`, `top`, `left`, `right`, `bottom`, `width`, `height`, `margin` and its four sides, `form`, `for`.

## Looks: the theme

- One Rust table, `Theme`, from element, `class` and state (normal, hovered, pressed, selected, disabled, focused, error) to colours, borders, padding and spacing, font size and the shadow. Today's `theme.rs`, `ButtonState::colours` and the sizes in `widgets/mod.rs` move into it.
- Spacing comes from elements by default: `div` has none, `p` has the theme's paragraph gap, a `panel` its padding, a `table` its `cellspacing` default, a `button` its margin. Keeps the 4 px rule in one place; `margin` attributes override it where a screen needs (rule 5).
- Swapping the theme (later, a mod) restyles everything without touching any template.
- Hover, pressed and selected looks, the ripple and the tooltip come from the input rules below, never from each widget.

## Input: the browser's rules

The goal: input behaves as on a web page, for mouse and touch alike. In detail:

1. **Pointers.** The mouse is one pointer, each finger another (a pen later), as in Pointer Events.
2. **One hit test per pointer per frame**, for the whole screen, drilling down the tree rather than checking every element. The top-level pieces (the game view, each panel, each dialog, an open list, a tooltip) are tried front to back by layer; inside the first one the point is on, only the child the point is on is gone into, children tried front-most first, and so on down. A subtree the point isn't on is never looked at, so the cost is about the depth of the tree, not its size. This needs one rule the layout keeps: **an element never reaches outside its parent**, for hits at least (anything that must, such as an open dropdown list or a tooltip, is a top-level piece of its own). `hidden` and `inert` elements are skipped with all they hold, as is anything scrolled out of sight, and everything outside the top modal `dialog`. The deepest element found is the **target**. A canvas or custom element can say a point is not on it (the round minimap's corners), and the search goes back up and on behind it.
3. **Hover is the target and its ancestors**, plus a control whose label is in that chain. Nothing else is hovered: a button behind a panel never lights up, because the panel is the target. A finger hovers only while it is down, as touch does in browsers.
4. **Events go capture, target, bubble**, as in the DOM: `pointerdown`, `pointerup`, `pointermove`, `pointerenter`/`pointerleave` (no bubbling), `click`, `dblclick`, `contextmenu` (right click, or a long press by finger), `wheel`, `keydown`, `keyup`, `input`, `change`. A handler may stop the event going further, or prevent its default action.
5. **Default actions after dispatch**, unless prevented: focusing a text box, the pressed look and ripple, a label working its control, a checkbox flipping, a select opening, Esc closing the top dialog.
6. **Click** goes to the nearest element both the down and the up were on, after the up (a finger's tap is a click, as now). A finger that moves past a small slop on something that scrolls becomes a scroll (`pointercancel`, no click): "a quick slide stays free for scrolling".
7. **Pointer capture** for drags: a slider's handle, a panel's title, a scroll bar, a held item take the pointer until it is let go, so nothing else hovers or reacts on the way.
8. **Wheel** goes to the target and bubbles to the nearest `scroll`. A panel or dialog takes and discards any wheel that reaches it, scrolling something or not, so wheeling over a window never zooms the camera. Only a wheel over the game view itself reaches the game.
9. **Focus and keys.** One focused element. Keys go to it and bubble to the document; whatever no element takes goes to the game's controls (`common::Controls`). Tab order is not planned (as now), but focus is what a gamepad will move later.
10. **Drag and drop** (items onto grids, slots, text boxes): the held thing is `inert`, the hit test finds what is under it, and that element gets `dragover` and `drop` with what is held. Replaces `DroppedOnText` and the grids' own searching.
11. **The game view is the bottom of the document.** `OverInterface`, `stops_presses`, `leaves_presses` and the order of `WidgetSystems` all become "was the target the game view's canvas?". Two-finger camera gestures are the game view's own handling of its pointer events.

### Hit testing: Bevy's picking or our own

Bevy 0.20 has `bevy_picking` with a UI backend (already compiled in: `notices` uses `Pickable::IGNORE`). It already does stacking order, blocking (`Pickable::should_block_lower`), per-pointer hover and touch ids. The 2026-11-01 decision turned down Bevy's `Interaction` (it doesn't take clicks away from other systems, and reacts to a finger landing), not picking.

Plan (owner: worth trying): PR 2 starts with a short spike using `bevy_picking`'s hit test as the source of the target, with our own dispatch (rules 3 to 11) on top. Its UI backend, as far as known, tests every pickable node in stacking order rather than drilling down the tree, so the spike measures it with a full screen of panels open. If it is slow, or can't respect clipping, `inert` or custom shapes, write our own drill-down hit test (rule 2) instead: one function over the element tree and its `ComputedNode`s. Either way there is one hit test, and everything reads it.

## Items in text

- Writing that holds items (chat later, item names, the Give Item box) shows each `${item:<uuid>}` as an `<object type="item" for="<uuid>"/>` element, by default, everywhere; an element can turn it off where the raw text is wanted.
- Such an item is the same element as anywhere else: its picture framed in its quality's colour, named on hovering. It needs an `id` only if code wants its events; one only shown gets none.
- Copying, where copying is allowed (text boxes, chat), copies the text form, `${item:<uuid>}`, so pasting it anywhere that shows items shows the item again.

## Placing pieces, and the player's own layout later

Top-level pieces of the screen (the movement pad, the action bar, the vitals ball, the minimap, the status text) are each a root element with a unique id, placed by the six attributes. That is what will later let a player move and keep them: a saved layout is just new `top`/`left`/`right`/`bottom` values by id, laid over the markup's. Not in these PRs, but nothing in them should make it harder (owner, 2026-11-10).

## How code uses it

A sketch; names to settle in PR 1.

```rust
// Spawning: inline markup, parsed once, checked by a test.
let doc = Document::spawn(&mut commands, parent, html!(r#"
    <panel title="Connect" moveable>
      <form id="connect">
        <label for="address">Address</label>
        <input id="address" type="text"/>
        <p id="status" hidden/>
        <button id="go" class="main">Connect</button>
      </form>
    </panel>
"#))?;

// Changing it: by id, never by entity.
dom.set_text("connect/status", "Checking…");
dom.set("connect/go", Attr::Disabled(true));
dom.copy_template("connect/server-row", "connect/servers", index);

// Reading what the player did: one message type, by id.
for event in ui_events.read() {
    match (event.id.as_str(), &event.kind) {
        ("connect/go", UiEventKind::Click) => { /* … */ }
        ("connect/address", UiEventKind::Input(text)) => { /* … */ }
        _ => {}
    }
}
```

- `html!` (or `include_str!` of a file beside the module) gives the source, built into the program (rule 11); a test walks a registry of every template and parses it, so a typo fails `cargo xtask test`, not the game.
- No template language (`{{name}}`). Values are set by id after spawning; lists copy `<template>`s. Small, and the same for mods.
- Owners only see events from their own documents (see the next section).
- The tree is retained, not rebuilt each frame: an `Element` component per node holds the tag and attributes, and a system restyles the Bevy `Node` when they change.

## Keeping things apart

- **Each document has an owner** (a module now, a mod later). Its events go only to that owner; its ids are unique inside it, and its forms are registered to it, so a mod can't take `settings/…`.
- **A document only draws inside its own root**, except through a `dialog` or `panel`, which get their layer from the document system (bands as today: maps 10, panels from 30, layers from 1,000,000, open lists 100 above, loading screen top). No element picks its own z.
- **Custom game elements' insides are hidden**: markup can't reach into an `object-grid`'s cells by id, and its events come out under the element's own id.
- **No scripts, strict parsing** (rules 6 and 9).

## Debug API

- `client.ui.list`, `get`, `set`, `press` keep working by id; `press` becomes "send a click through the dispatcher at the element's centre", so it obeys the same rules as the player.
- New: `client.ui.html` returns the live tree as markup (handy for checking a screen without a screenshot), and `client.ui.at x=… y=…` returns the hit test's target and hover chain, to check that nothing covered reacts.

## What needs custom handling

What markup can't describe, by kind. Everything else is plain markup.

**Custom game elements and canvases**: see their tables under Elements.

**Not elements at all:** the held item following the pointer (an `inert` top-level piece code moves each frame), the lightning flash (an `inert` full-screen piece), and the soft keyboard's hidden browser input.

**Built in, but standard to markup**: the text box (caret, selection, clipboard, items dropped in), the open dropdown list, the key box's waiting, the switch's slide, the slider, the colour picker, scroll areas, moving panels, hold buttons. Their Rust stays; a template just writes `<input type="text"/>`.

**Things that become attributes**: `Floating` is `inert`; outlined white words are `class="outlined"`; `HoverTip` and `ButtonTooltip` are `title`; the shader looks (aura, set edge, stripes) are part of the `object` and `object-slot` elements, not markup.

## What stays as it is

- The widgets' insides that work: text editing (`text_box/edit.rs`, carets, selection, clipboard, soft keyboard), the dropdown's list, the switch's slide, scroll memory, materials and shaders (aura, set edge, stripes, skill ring), item pictures. They become the implementations behind elements rather than being rewritten.
- `UiScale` and the interface size.
- The 3D views' own picking of tokens, pois and cells, inside the game view's canvas.

## PRs

Each passes `cargo xtask test`, keeps files under the size limit, and updates the context files it touches. Old `spawn_*` functions keep working until the last PR, so the game is never half broken.

1. **Markup and layout.** *Built.* New client module `dom`: the parser (our own, see the top), the element set so far, validation, the `Theme` table, spawning to Bevy nodes, text, `img`, `progress`, `canvas`, `template`, and changes by id (`Dom`). No input yet. Show Widgets has a sample of markup under its widgets.
2. **One hit test and the event rules.** *Built.* `dom::Hit` (the drill-down search), the hover chain, events passed up from the target, capture, click and double click, the wheel staying with what the pointer is on. The old widgets light up and press by the same search, so nothing covered reacts. `client.ui.at`, `client.ui.html`.
3. **Controls.** `button`, `label`, checkbox and switch, text, range, radio rows, `select`, `key-box`, `colour-picker`, `tab-bar`, hold buttons, `UiEvent`, debug API by id. Show Widgets fully in HTML.
4. **Surfaces.** `dialog`, `panel`, `scroll` (with Always Show Scroll Bars in Settings), tooltips from `title`, drag and drop, items in text.
5. **Menus.** System menu, Settings, Connect, Server Options, Credits, the quit and other requesters.
6. **Debug.** Debug menu and its panels.
7. **Getting in.** Login, Character Select, Character Create (triangles as a canvas), loading screen.
8. **Items.** `object`, `object-slot` and `object-grid` for both types, then the inventory, bags, trade, ground, split box, `container`, items in text.
9. **Character panel.** Character, Abilities (the same elements with `type="ability"`; the action bar's slots `accepts="ability"` for now), Skills (canvas). Needs 8.
10. **The rest of the screen.** Minimap and full map (canvases), step pad, vitals, HUD, notices, hover box, Accounts panel.
11. **Remove the old way.** The `spawn_*` functions, per-widget hit tests, `Floating`, `stops_presses`, `leaves_presses`, `OverInterface`, `WidgetSystems`' ordering rules. Rewrite `widgets.md` as `dom.md`, update `ui-design.md`, `debug-api.md`, the table in `CLAUDE.md`, and record the decision.

5 to 10 don't depend on each other once 4 has merged (except 9, which needs 8), so their order is the owner's.

## Owner's answers (2026-11-10)

1. **Items:** `<object type="item" id="..." for="<uuid>"/>`, `id` always the element's own name for events, `for` as a label's. Items in text become elements by default; copying copies the text form.
2. **Names:** HTML-alike, not HTML: our own elements needn't have a hyphen.
3. **Placing:** all six of `top`, `left`, `right`, `bottom`, `width`, `height`; two sides imply the size, one side aligns to it. Top-level pieces with unique ids, so the player can lay out their own interface later.
4. **Our own attributes** on our own elements (`accepts-items`, `prefix`, `suffix`) are fine.
5. **Spacing:** the theme's by default, overridden by `margin` and `margin-top`, `-right`, `-bottom`, `-left`.
6. **Files:** no interface files read at run time; `.html` files beside a module are fine if pulled in when compiling.
7. **Hit test:** try `bevy_picking` first, and drill down the tree rather than test every element (see Hit testing).
8. **Variants:** `class`.
9. **Items and abilities are one set of elements** (`object`, `object-slot`, `object-grid`) told apart by `type`, so a slot can later take either: the action bar holding potions. Putting items on the bar is a game rule change for later (a TODO), not part of this plan.

Also asked for: a `scroll` can always show its bar (`scrollbars="always"`, and a player's switch for all in Settings, Interface, off by default: for accessibility as well as development), and panels swallow the wheel. Margins are multiples of 4 (0 allowed), checked by the parser, as it looks cleaner.

