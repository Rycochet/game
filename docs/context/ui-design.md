# UI design

The palette to plan screens from: what each piece is, how it should look and behave here, and whether we have it. It began as the owner's guide built on Material UI's components (2 November 2026), so it uses their names; the game keeps its own parchment look. Update it as needs change. How the code does it: `client/widgets.md`.

Status: **have**, **PR** (planned in the widget cleanup, PR 2, 3 or 4), **later** (build when a screen needs it), **no** (no use in this game so far).

## Shared rules

- **Colours:** named, in `widgets/theme.rs`. Parchment and ink, with raised, selected and disabled colours, green for allowed and red for refused. One look: no light and dark modes.
- **Text:** Lexend, in four named sizes (`TextSize`): small 13, medium 16, large 18, title 24. Words as written, not uppercase.
- **Size:** everything is drawn at the player's interface size (Settings, Interface): Auto keeps its share of the window, with 100% in a 1920 × 1080 window; or 50% to 300%. "Enable touch" keeps it at least 100%. The minimap and full map keep their share of the window instead. Design at 100%; check 50% and 200%.
- **Spacing:** gaps and padding are multiples of 4 px. Edges are exempt.
- **Shape:** square corners for now.
- **Raised things:** lit along the top and left, shaded along the bottom and right. Panels, tooltips and open lists also have one soft shadow.
- **Feedback:** every clickable thing has hovered and pressed looks, and a ripple from the point pressed. The hover look doubles as the focus look later.
- **Input:** whatever a mouse can do, a finger can do. Gamepad later. Tab focus and screen readers are not planned for now.

## Inputs

| Piece | Status | Here |
| --- | --- | --- |
| Button | have | Normal, selected, active, disabled. A main style (one per area) and a quiet one with no box until hovered. No spinner. |
| Icon button | have | Square, with padding and an optional tooltip naming it. |
| Close button | have | A normal raised button, with a letter X. |
| Checkbox | have | A whole clickable row with a drawn tick. **Ask the owner before using an indeterminate dash:** they are unsure about it. |
| Toggle row | have | Joined buttons, one chosen, all visible: for 2 to 5 options. Stands in for radio buttons. |
| Slider | have | Picture or plain track with a wide handle. Ticks, value bubble, arrow keys and ranges later. |
| Text box | have | Sunken, focus border, error state, optional label above. No floating label. |
| Dropdown | have | One widget for Select and Autocomplete, with search as an option, and an arrow. Clear button and multiple choice later. |
| Switch | later | For settings that apply at once, once Settings has real options. Until then checkboxes do it. |
| Fab, ButtonGroup, Rating, SpeedDial | no | |

## Showing things

| Piece | Status | Here |
| --- | --- | --- |
| Text | have | `TextSize`. |
| Tooltip | have | One dark look. A delay on buttons, none on items or what is under the pointer. Never the only way to learn something: fingers have no hover. |
| Icon | have | Pictures, or drawn in code. |
| Scroll area | have | Wheel, finger or bar. |
| List | have | Rows of buttons in the debug menu and open dropdowns. |
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
| Spinner, Skeleton | no | |

## Surfaces and moving about

| Piece | Status | Here |
| --- | --- | --- |
| Panel | have | One style of edge, shadow and parchment. **Moveable panels are told from static ones by the lighter stripes either side of the title**, which static panels don't have. A new panel opens in front of those already open. |
| Full-screen layer | have | 90% of the window, 75% for one above it. |
| Tabs | have | Folder-style, for switching between views of one thing (Settings). |
| Menu | later | Right-click item menu: use, split, drop. |
| Card, Accordion, AppBar, Drawer, Breadcrumbs, Pagination, Stepper, BottomNavigation, Link, ImageList | no | |

## Items, slots and containers

| Piece | Status | Here |
| --- | --- | --- |
| Item | have | Any contiguous shape of cells, shown with a picture or colour, with an edge showing quality. Held at true size and solid, for now: the green and red cells already show where it goes. |
| Item slot | have | Any size of item shrunk to fit. Hint icons come with type limits: leave room for them. |
| Container | have | A grid in a panel. Fixed width; fixed height, or growing and scrolling for what has no limit, such as a shop or the ground. |
| Stack | have | Items that look the same in one place, with a count; dropping on one joins it. Limits are the server's. Splitting and mixed stacks to come. |

Rules:

- A drop succeeds only if the whole footprint lands on empty cells inside the grid. The preview shows green or red. Anything refused snaps back.
- Items turn in 90-degree steps while held; they never flip.
- Right-click turns only while holding, so a right-click menu can use it otherwise.

| Action | Mouse | Touch | Gamepad (later) |
| --- | --- | --- | --- |
| Pick up | Press and drag; click | Press and hold, then drag | A / Cross |
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
| Turn something on or off | Checkbox (Switch later) | |
| Choose a rough number | Slider | Text box |
| Force a decision before going on | Dialog | |
| Switch between views of one thing | Tabs | Toggle row |
| Show several clickable things in a stack | List of buttons | |
