# client: vitals

The player's Health and Mana, shown as a circle at the bottom middle of the window: Health on the left in red, Mana on the right in blue, each half filling from the bottom over black, with a border round each half (owner's design, 2026-11-06). Native and web.

## Files

- `crates/client/src/vitals/mod.rs`: `VitalsPlugin`; `spawn_circle` (Startup: the node, `UiId` `hud/vitals`, a `HoverTip`), `take_vitals` (puts `ServerMessage::Vitals` with no `who` in `common::OwnVitals`), `show_circle` (places and sizes it, writes `common::BottomHud`, paints when a level moves a quarter of a pixel, keeps the tip's words: "Health 18 / 20", "Mana 20 / 20"). `ACROSS_OF_HEIGHT` (0.1 of the window's height: half the minimap), `BOTTOM_GAP`.
- `crates/client/src/vitals/paint.rs`: `paint(side, health, mana)`: the picture's bytes in screen pixels, smoothed edges, the border the minimap's brown, bevelled lit from the top left so each half stands up like a button (owner, 2026-11-06), down the middle too.

## Public API

- `VitalsPlugin`. What other modules read is in `common` (`OwnVitals`, `BottomHud`).

## Depends on

- `common` (`OwnVitals`, `ToldVitals`, `BottomHud`, `GameView`, `interface_area`), `connection` (`FromServer`), `widgets` (`HoverTip`, `UiId`), `shared::vitals`, `shared::protocol`.

## Used by

- Nothing directly; `character` and `minimap` read what it writes in `common`.

## Notes

- The client refills what it was told itself, by `Time` since it came (`ToldVitals::now`, with fractions for a smooth fill), which stops while single player is paused, as the server's clock does. The server only sends `Vitals` when something other than refilling changes them.
- Hidden until told, and while `GameView::Connecting`. It takes no clicks (no `Pressable`), so clicks and taps under it reach the world; a finger held on it shows the tip, as for any `HoverTip`.
- The planned action bar goes five frames either side of it (Abilities and action bar chat).
- Party members' vitals come as `Vitals` with `who` set, which this ignores for now.
