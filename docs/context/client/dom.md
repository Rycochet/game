# client: dom

The interface as markup: HTML's look with XML's closing rule, no CSS and no scripts. A template is checked strictly, built into Bevy UI nodes, and changed by id. The whole plan, the rules of the markup and what is still to come: `docs/context/html-interface.md`. So far it has layout, text, pictures, progress and canvases, and the input rules (what the pointer is on, and events sent up through what holds it), which the old `widgets` read too; there are no controls yet, so the old `widgets` still make everything players use.

## Files

- `crates/client/src/dom/mod.rs`: `DomPlugin` (`PostUpdate`, before layout: `build_documents`, `apply_changes`, `restyle`, in that order), the re-exports.
- `crates/client/src/dom/template.rs`: `Template { name, source }`, the `html!` macro, `checked()`, `assert_good()` (tests only).
- `crates/client/src/dom/error.rs`: `MarkupError` (`at`, `what`), `Place` (line and column, from 1).
- `crates/client/src/dom/markup.rs`: markup as written (`RawNode`, `RawElement`, `RawAttr`). In `markup/`: `cursor.rs` (`Cursor`: a place moving along the source), `parse.rs` (`parse`), `text.rs` (`tidy`: spaces as HTML's; `decode`: `&amp;` and the like).
- `crates/client/src/dom/tag.rs`: `Tag`, what each may hold (`Holds`), where it may go, which need an `id`, their own attributes.
- `crates/client/src/dom/attrs.rs`: `Attrs`, `is_known`. In `attrs/`: `values.rs` (`Length`, `Align`, `VAlign`, reading numbers), `set.rs` (`Attrs::set`, `clear`: one attribute by name, the one path for markup and code).
- `crates/client/src/dom/check.rs`: `check`: raw markup into `Checked` / `CheckedNode`, with ids, forms, `for` and where elements may go.
- `crates/client/src/dom/theme.rs`: `Theme` (resource), `Look`.
- `crates/client/src/dom/layout.rs`: `node_for`: an element's `Node` from its attributes, look and place in a table.
- `crates/client/src/dom/parts.rs`: the components: `Document`, `document()`, `Element`, `Inert`, `Canvas`; private `TextStyle`, `Words`, `InTable`, `Bar`, `TemplateBody`.
- `crates/client/src/dom/build.rs`: `Builder`, `build_documents`. In `build/`: `table.rs`, `progress.rs`.
- `crates/client/src/dom/restyle.rs`: `restyle`: `Node`, background, tooltip, picture and bar of every new or changed `Element`.
- `crates/client/src/dom/pictures.rs`: `Pictures` (resource), `picture_for`.
- `crates/client/src/dom/change.rs`: `Dom`, `DomChange`. `change/apply.rs`: `apply_changes` (an exclusive system).
- `crates/client/src/dom/input.rs`: `DomInput` (system sets), `add`. In `input/`: `hit.rs` (`Hit`, `PointerCapture`, `Finder`, `find_target`, `find_now` for tests), `hover.rs` (`follow_hover`), `events.rs` (the events), `dispatch.rs` (`send_events`, `Buttons`).
- `crates/client/src/dom/attrs/write.rs` (tests and the debug API): `Attrs::written()`, attributes back as markup.
- `crates/client/src/dom/debug_api.rs` (debug API only): `client.ui.at`, `client.ui.html`.

## Public API

- `html!("name", r#"…"#)` (or `include_str!` of a file beside the module): a `Template`, usually a `static`. The source is always part of the program.
- `document(&TEMPLATE)`: the bundle to spawn (as a child of where it goes) for the template to be built there by the end of the frame. `Document(&Template)` is on its root.
- `Dom` (system parameter): `set_text(id, words)`, `set(id, name, value)`, `set_flag(id, name, on)`, `clear(id, name)`, `copy_template(template, into, index)`, `empty(id)`. Ids are as written, `form/name`, and `form/name.<index>` inside a copy of a template. Changes are made in order at the end of the frame, after new documents are built, so a system may spawn a document and fill it in one go. One that can't be made (no such id, a value the attribute can't take) is logged and changes nothing.
- `Template::assert_good()`: every module with templates has a test calling it for each.
- `Theme`: `look(tag, class)`, `set(tag, class, look)`, `has`; `Theme::parchment()` is the game's. `Look`: text size, colour, margin, gap, background.
- `Pictures::name(name, handle)`: a picture made in code, for `<img src="picture:<name>"/>`.
- `Element { tag }` (component, on every element), `Canvas` (on a `canvas`: its owner adds what it draws), `Inert` (on an `inert` element; nothing reads it until the hit test of PR 2).
- `Tag`, `Attrs`, `Length`, `Align`, `VAlign`, `Checked`, `CheckedNode`, `MarkupError`, `Place`.
- `Hit` (resource, `PreUpdate`, `DomInput::Find`): `target`, the thing in front at the pointer (an element, or a `Solid` node), `chain` (the target, then everything it is in, outwards) and `at`; `first(pick)` finds the nearest of the chain that `pick` takes. Everything asking what the pointer is on reads this, so only one thing is ever in front.
- `Solid` (component): on a node that isn't an element, it can be the target and hides what is behind it. Elements always can. The old widgets' `Pressable`, `ButtonState` and panel parts require it.
- `Hovered` (component): on each element of the chain, and on nothing else.
- Events, sent to the target and then up through what holds it until an observer stops one (`event.propagate(false)`): `Press`, `Release`, `Click { count }` (2 for a double click), `ContextMenu` (right button), `Wheel { lines }`; and `PointerEnter` / `PointerLeave`, to one element only. Watch with `commands.entity(element).observe(|click: On<Click>| …)`.
- `PointerCapture(Option<Entity>)` (resource): set it for that entity to be the target wherever the pointer goes, until the left button is let go.
- `DomInput::Find` / `Send` (system sets, `PreUpdate`): `Find` is before `WidgetSystems::Presses`, and `Send` between that and `WidgetSystems::Front`.
- `Finder` (system parameter): `target_at(at)`, `chain_of(target)`, for asking about another point.

## Elements so far

`div`, `span`, `table` / `tr` / `td` / `th`, `form`, `template`, `br`, `p`, `h1`, `h2`, `small`, `label`, `img`, `progress`, `hr`, `canvas`. Attributes on all: `id`, `form`, `class`, `title`, `hidden`, `disabled`, `inert`, `align`, `valign`, `top`, `left`, `right`, `bottom`, `width`, `height`, `margin` and `margin-<side>`. Their own: `src`, `alt` (`img`); `value`, `max` (`progress`); `colspan` (`td`, `th`); `cellspacing`, `cellpadding` (`table`); `for` (`label`). Classes the theme knows: `p class="warning"`, `table class="striped"`.

## Depends on

- `common`: `UiPointer`, `in_interface`.
- `widgets`: `UiId` (ids), `HoverTip` (`title`), `Progress` / `spawn_progress_bar` and `Spinner` (`progress`), `TextSize` and the colours. This turns round as widgets become elements; the last PR of the plan removes it.

## Used by

- `debug_menu`: Show Widgets' markup sample (`panels/html_sample.rs`).
- `widgets`: `Hit` (what is pointed at and pressed), `Solid`, `DomInput`.
- `view3d`: `Hit` (the wheel zooms only with the pointer on the view).

## Notes

- Our own parser, not a library: the dialect is small and strict, ids must be `&'static str` slices of the source for `UiId`, and errors want a line and column.
- An `id` needs a form: a `<form id="…">` round it or its own `form="…"`. A `form` element has no `UiId` of its own; `form`, `template` and `canvas` must have an `id`.
- A template's ids are checked apart from the document's (each copy gets them numbered), and a `template` is never laid out.
- Words: runs of spaces become one, none at either end, and words made only of spaces are dropped, so the gap between things in a line is the look's `gap` (4 px), not a space in the markup.
- Block elements (`div`, `form`) stack; the rest flow in a wrapping row. `br` and `hr` are as wide as what they are in, so they take a whole line of a flow.
- Tables are a grid whose rows and cells are all children of the table (a grid only places its own children): a `tr` is a strip behind its cells, and hiding it hides them (`restyle` looks at a changed row's cells again).
- `progress` is built as a bar or a spinner by whether it has a `value` in the markup; setting or clearing `value` later doesn't switch between them.
- Changing `class` later changes an element's own look, but not the size or colour of words already inside it.
- `restyle` is the one place a `Node` is made, at spawning and on any change, so markup and `Dom` can't disagree.
- Not all of `Dom`, the events and the re-exports is used yet: the `#[allow]`s on them go as screens become markup.
- The search drills down: a point that isn't on an element isn't on anything in it, so its insides are never looked at (markup never lays anything out beyond what holds it). Nodes that aren't elements may hold things anywhere (the old widgets do), so everything in them is tried and the one furthest in front wins; that part shrinks as screens become markup. `inert` and hidden things are passed over with all they hold.
- Our own search, not `bevy_picking`'s: its UI backend tries every node in the stack rather than drilling down, takes its own pointers rather than `UiPointer` (the finger or the mouse, in interface pixels, which fake input moves too), and only tells markers apart for the whole interface.
- A press on an element goes no further (`send_events` takes it from the mouse), so the game behind never sees it. One on another `Solid` node is left to whoever owns it, as before.
- A click goes to the nearest thing both the press and the letting go were on. A finger's tap arrives as a left click (`touch`), so it is a `Click` too.
- Not built yet: more than one pointer at once, the capture phase, focus and keys, drag and drop, a long press as `ContextMenu`, and a canvas saying a point isn't on it. Sliders, text boxes, scroll areas, hold buttons and item grids still do their own searches until they become elements.
