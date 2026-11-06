# UI design

The palette to plan screens from: what each piece is, how it should look and behave here, and whether we have it. It began as the owner's guide built on Material UI's components (2 November 2026), so it uses their names; the game keeps its own parchment look. Update it as needs change. How the code does it: `client/widgets.md`.

Status: **have**, **PR** (planned in the widget cleanup, PR 2, 3 or 4), **later** (build when a screen needs it), **no** (no use in this game so far).

## Shared rules

- **Colours:** named, in `widgets/theme.rs`. Parchment and ink, with raised, selected and disabled colours, green for allowed and red for refused. One look: no light and dark modes.
- **Text:** Lexend, in four named sizes (`TextSize`): small 13, medium 16, large 18, title 24. Words as written, not uppercase.
- **Size:** everything is drawn at the player's interface size (Settings, Interface): Auto keeps its share of the window, with 100% in a 1920 × 1080 window; or 50% to 300%. "Enable Touch" keeps it at least 100%. The minimap and full map keep their share of the window instead. Design at 100%; check 50% and 200%.
- **Spacing:** gaps and padding are multiples of 4 px. Edges are exempt.
- **Shape:** square corners for now.
- **Raised things:** lit along the top and left, shaded along the bottom and right. Panels, tooltips and open lists also have one soft shadow.
- **Feedback:** every clickable thing has hovered and pressed looks, and a ripple from the point pressed. The hover look doubles as the focus look later.
- **Ids:** every input widget has a `UiId` (`form/name`), unique among those on show, so the debug API can find it; give new ones their ids as they are made.
- **Input:** whatever a mouse can do, a finger can do. Gamepad later. Tab focus and screen readers are not planned for now.

## Inputs

| Piece | Status | Here |
| --- | --- | --- |
| Button | have | Normal, selected, active, disabled. A main style (one per area) and a quiet one with no box until hovered. No spinner. |
| Icon button | have | Square, with padding and an optional tooltip naming it. |
| Small icon button | have | 22 px, for controls on the edge of something, such as the sort and filter buttons above a grid. An icon dropdown is one, opening a list of options with their pictures, some greyed. |
| Close button | have | A normal raised button, with a letter X. |
| Checkbox | have | A whole clickable row with a drawn tick, for picking things out of a list (the debug menu's toggles); settings use switches. **Ask the owner before using an indeterminate dash:** they are unsure about it. |
| Toggle row | have | Joined buttons, one chosen, all visible: for 2 to 5 options. Stands in for radio buttons. |
| Colour picker | have | Square swatches edge to edge in a box like an active button, wrapping like words; one line as tall as a dropdown. A swatch is a colour or a picture (patterns, gradients). The chosen one has a flat border with a white one inside; each names itself in a tooltip, and the label above says what is chosen. For looks: hair, skin, coat, feathers. |
| Slider | have | Picture or plain track with a wide handle. Ticks, value bubble, arrow keys and ranges later. |
| Text box | have | Sunken, a line tall even when empty, focus border, error state, optional label above, optional grey words after the text (Connect's port found). No floating label. A thin caret anywhere in the writing, solid while typing and fading gently twice a second when idle; selection by keys, mouse or finger (double click a word, triple click all), kept paler without the keyboard; cut, copy and paste by keys; long writing slides. No right-click menu for now (owner, 2026-11-06: maybe after feedback). A box may take items: one dropped on it is written in at the caret, shown as its picture in its quality's frame, named on hovering, and one place for editing. |
| Text with items | have | Writing to read with items in it, shown as in a box that takes them (`ItemText`). |
| Dropdown | have | One widget for Select and Autocomplete, with search as an option, and an arrow. Clear button and multiple choice later. |
| Key box | have | A text box's look, showing a key binding ("Shift + G") or nothing, centred. A click or tap makes it wait ("Press key", or the modifiers held so far, "Shift + …") for the next key, with any modifiers held, or a modifier pressed and let go alone; Esc or a click elsewhere cancels, Backspace empties it. Disabled grey; an error edge when its binding was taken by another. Settings, Controls. A gamepad's box would wait for a button ("Press button"). |
| Switch | have | A pill about a phone's size (52 × 32), with a tick in the left half of its track and a cross in the right; the round thumb covers one, so off shows the cross. Green track when on, grey when off. A press anywhere flips it and the thumb slides across (jumps with Reduce Motion). For settings that apply at once: every on/off setting in Settings. |
| Fab, ButtonGroup, Rating, SpeedDial | no | |

## Showing things

| Piece | Status | Here |
| --- | --- | --- |
| Text | have | `TextSize`. |
| Tooltip | have | One dark look. A delay on buttons and on `HoverTip` rows (Settings: what a setting does, its cost and why it is disabled), none on items or what is under the pointer. A finger held down shows a row's. Never the only way to learn something: fingers have no hover. |
| Icon | have | Pictures, or drawn in code. |
| Scroll area | have | Wheel, finger or bar. |
| List | have | Rows of buttons in the debug menu and open dropdowns. |
| Triangle chooser | have | Only in making a character (`character_create`): three triangles turning round a middle on a ring in a button's colour, with curved two-headed arrows in the gaps, always the same way up, a dot in each, each with a radar chart behind it (the kind's bonus as a gold inner layer, round-ended). Drawn into a picture; dots dragged, the whole turned by dragging beside them; keys 1–3, arrows, Q/E. |
| Tree diagram | have | Only on the Skills page (`skills_page`): round nodes on rings round a middle, joined by thin lines, names under them; dragged to move and zoomed about the pointer or fingers (spreading apart, words staying sharp), deeper names hiding when far out; a click or tap chooses a node, shown by an orange edge, with details in a line under it. |
| Badge | have | A stack's count: white with a black outline, bottom centre inside the item's frame. |
| Table, Chip, Avatar, Divider | later | |

## Feedback

Pick by how much it should interrupt: Snackbar least, Dialog most.

| Piece | Status | Here |
| --- | --- | --- |
| Dialog | have | `spawn_dialog` over full-screen layers: title, content, buttons. The quit question uses it. |
| Darkening | have | What is outside a layer; each layer's adds up. |
| Progress bar | have | From the loading screen. |
| Snackbar, Alert | later | Likely soon: "Saved", "Connection lost". |
| Spinner | have | A ring with a gap, turning on the real clock, for something under way that can't say how far it has got (Connect checking an address). Small, beside or in what it is about. |
| Skeleton | no | |

## Surfaces and moving about

| Piece | Status | Here |
| --- | --- | --- |
| Panel | have | One style of edge, shadow and parchment. **Moveable panels are told from static ones by the lighter stripes either side of the title**, which static panels don't have. A new panel opens in front of those already open. |
| Full-screen layer | have | 90% of the window, 75% for one above it. |
| Tabs | have | Folder-style, for switching between views of one thing (Settings, the Character panel). |
| Menu | later | Right-click item menu: use, split, drop. |
| Card, Accordion, AppBar, Drawer, Breadcrumbs, Pagination, Stepper, BottomNavigation, Link, ImageList | no | |

## Items, slots and containers

| Piece | Status | Here |
| --- | --- | --- |
| Item | have | Any contiguous shape of cells, shown with a picture or colour, with an edge showing quality. Held at true size and solid, for now: the green and red cells already show where it goes. |
| Ability | have | An ability's icon in a frame 1.5 cells across, named on hovering (`widgets::spawn_ability`): square if active, round if passive, greyed with a dull red frame when it can't be used now. Action bar slots (frames to drop them in, with key labels and an automatic sweep) come next. |
| Item slot | have | Any size of item shrunk to fit. Hint icons come with type limits: leave room for them. |
| Container | have | A grid in a panel (`widgets::spawn_container`). Fixed width; fixed height, or growing and scrolling for what has no limit, such as a shop or the ground. Its weight under it when it is carried or carriable (`12.34kg / 45.00kg`), never for stock. Controls (sort, filter) at its top right, slots (Drop, bin) at its bottom right. |
| Stack | have | Items that look the same in one place, with a count; dropping on one joins it. Limits are the server's. Split by shift+click or a tap: a dark box with the name, a slider and the stack to pick up. A mixed stack has a red "!" in its corner, and any press opens the box with a row for each kind and one for all. |

Rules:

- A drop succeeds only if the whole footprint lands on empty cells inside the grid. The preview shows green or red. Anything refused snaps back.
- Items turn in 90-degree steps while held; they never flip.
- Right-click turns only while holding, so a right-click menu can use it otherwise.

| Action | Mouse | Touch | Gamepad (later) |
| --- | --- | --- | --- |
| Pick up | Press and drag; click | Press and hold, then drag | A / Cross |
| Split a stack | Shift+click | Tap | |
| Place | Release; click | Lift finger | A / Cross |
| Turn held item | Right-click; R | Tap with a second finger | Shoulder button |
| Cancel | Esc; drop somewhere invalid | Drop somewhere invalid | B / Circle |
| Details | Hover | Tap | Y / Triangle |
| Open a bag | Double press | Double tap | |

A simple tap shows an item's name, and later the item info boxes that will replace it. A quick slide stays free for scrolling.

## Quick chooser

| I need to... | Use | Not |
| --- | --- | --- |
| Pick one of 2 to 5 options, all visible | Toggle row | Dropdown (hides choices) |
| Pick one of 5 to 15 options | Dropdown | Toggle row (too wide) |
| Pick from a long list | Dropdown with search | Dropdown without |
| Turn a setting on or off | Switch | Dropdown of On and Off |
| Tick things in a list | Checkbox | |
| Choose a rough number | Slider | Text box |
| Force a decision before going on | Dialog | |
| Switch between views of one thing | Tabs | Toggle row |
| Show several clickable things in a stack | List of buttons | |
