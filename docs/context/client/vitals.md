# client: vitals

The player's Health and Mana, shown as a circle at the bottom middle of the window: Health on the left in red, Mana on the right in blue, each half filling from the bottom over black, with a border round each half (owner's design, 2026-11-06). Native and web.

## Files

- `crates/client/src/vitals/mod.rs`: `VitalsPlugin`; `VITALS` (the circle as markup: one `canvas`, `vitals/ball`, placed 8 above the window's bottom and hidden to begin with), `spawn_circle` (Startup: the picture, and the document on `dom::bottom_strip()`), `bind_circle` (once built, gives the canvas its picture and `widgets::stops_presses`, so a right press on it goes no further either), `take_vitals` (puts `ServerMessage::Vitals` with no `who` in `common::OwnVitals`). `ACROSS_OF_HEIGHT` (0.1 of the window's height: half the minimap), `BOTTOM_GAP` (a test checks the markup has it).
- `crates/client/src/vitals/show.rs`: `show_circle` (writes `common::BottomHud` and `common::VitalsCircle`, paints when a level moves a quarter of a pixel, and through `Dom` keeps the canvas's `hidden`, `width`, `height`, `left` (the window's middle) and `title` (the tip: "Health 18 / 20", "Mana 20 / 20"), each only when it differs from what was last told: `Told`).
- `crates/client/src/vitals/killed.rs`: `spawn_killed`, `show_killed`: the requester saying "You have been killed, try again!" with OK (`killed/ok`), shown on `ServerMessage::Killed` and blocking the game until closed (owner, 2026-11-07, for now); with `permanent` (Ironman) it says the character is gone, a layer higher, over the creation screen that follows.
- `crates/client/src/vitals/paint.rs`: `paint(side, health, mana)`: the picture's bytes in screen pixels, smoothed edges, the border the minimap's brown, bevelled lit from the top left so each half stands up like a button (owner, 2026-11-06), down the middle too.

## Public API

- `VitalsPlugin`. What other modules read is in `common` (`OwnVitals`, `BottomHud`, `VitalsCircle`).

## Depends on

- `common` (`OwnVitals`, `ToldVitals`, `BottomHud`, `GameView`, `interface_area`), `connection` (`FromServer`), `dom` (`Dom`, `document`, `bottom_strip`), `widgets` (`UiId`), `shared::vitals`, `shared::protocol`.

## Used by

- Nothing directly; `character` and `minimap` read what it writes in `common`.

## Notes

- The client refills what it was told itself, by `Time` since it came (`ToldVitals::now`, with fractions for a smooth fill), which stops while single player is paused, as the server's clock does. The server only sends `Vitals` when something other than refilling changes them.
- Hidden until told, and while `GameView::Connecting`. Presses on it go no further (it is an element), so nothing under it is walked to; a finger held on it shows the tip, as for any `title`.
- The canvas has no `form` element round it, only its own `form="vitals"`: a form in a strip has no height, and the pointer is never on what lies outside its form.
- The action bar (`abilities`) stands five slots either side of it, by `common::VitalsCircle`.
- Party members' vitals come as `Vitals` with `who` set, which this ignores for now.
