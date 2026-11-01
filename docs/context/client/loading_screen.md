# client: loading_screen

The screen covering everything from the first frame until the game can take input: "Hall of Worlds" and, at the bottom, a bar saying what is happening. It comes back while the connection is lost, and shows why if the server refuses or the client won't trust it.

## Files

- `crates/client/src/loading_screen/mod.rs`: `LoadingScreenPlugin`; `next_step` (from frames run and frames since the welcome), `title_font_size`, `bar_shows`, `take_all_input` (`PreUpdate`, before the widgets: clears keys, mouse, wheel and fingers while up), `keep_frame_rate`, `remove_page_screen` (web only). Unit tests.

## Public API

- `LoadingScreenPlugin`.
- `HoldLoadingScreen(bool)` (resource): keeps the screen up, for `HOW_DEBUG_LOADING_HOLD`.
- It writes `common::LoadingStep`, which is what others read.

## Depends on

- `common` (`LoadingStep`, `ServerStatus`, `GameView`, `TouchGesture`, `PointerSystems`, `frame_settings`), `widgets` (`WidgetSystems`, for ordering).

## Used by

- `dev_tools` (`HoldLoadingScreen`). `connection` reads `common::LoadingStep`.

## Notes

- To add a step for startup work: add it to `LoadingStep`, then give it a place in `next_step` and words and a fill in `bar_shows`. The bar's positions are fixed for now, not timed.
- `connection` waits out `Starting`, or the word would never be seen: it would connect before the first frame is drawn.
- The title's font is built in with `include_bytes!` to be there on the first frame. `TITLE_EMS_WIDE` is measured from the font: change it with the font or the words.
- While up, frames keep their focused rate even out of focus (`common::frame_settings`).
- On the web, `index.html` has its own copy of this screen saying "Downloading...", for until the wasm has loaded and started. `remove_page_screen` removes it when `Starting` ends, so there is no gap. Its colours, sizes and words are copied: change them in both places. Its fonts come from the same server as the page, and its title stays hidden until its font has loaded.
