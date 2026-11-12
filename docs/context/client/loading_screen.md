# client: loading_screen

The screen covering everything from the first frame until the game can take input: "Hall of Worlds" and, at the bottom, a bar saying what is happening. It comes back while the connection is lost, and shows why if the server refuses or the client won't trust it. At the startup menu (`common::Session::AtStart`) it is only the title, smaller and at the top, with the system menu over it.

## Files

- `crates/client/src/loading_screen/mod.rs`: `LoadingScreenPlugin`; `next_step` (from frames run and frames since the welcome), `title_font_size`, `bar_shows`, `take_all_input` (`PreUpdate`, before the widgets: clears keys, mouse, wheel and fingers while up, but not at the startup menu, which takes them, nor while `common::MakingCharacter`, `common::CharacterSelect::open` or `common::LoggingIn::shows`, whose screens take them), `LOADING` (the screen as markup: `loading/screen` holding the `loading/title` canvas and the `loading/bar` progress bar, class `loading`), `spawn_screen` (the document, on its own dark node over everything), `bind_screen` (`PostUpdate`, straight after the document is built and before layout, so in the first frame: the title in its font, already its size, and the bar's words marked for their colour), `show_screen` (through `Dom`: the bar hidden at the startup menu, while making a character and while logging in; its words and how full it is), `fit_title` (size and place by window and `Session`), `remove_page_screen` (web only). Unit tests. `frames.rs`: `keep_frame_rate`. `screen.rs`: `LOADING`, `spawn_screen`, `bind_screen`, `title_node`.
- `crates/client/src/loading_screen/takes_input.rs`: `takes_input`, when `take_all_input` runs (up after Play, but not while choosing or making a character or logging in).

## Public API

- `LoadingScreenPlugin`.
- `HoldLoadingScreen(bool)` (resource): keeps the screen up, for `HOW_DEBUG_LOADING_HOLD`.
- It writes `common::LoadingStep`, which is what others read.

## Depends on

- `common` (`LoadingStep`, `ServerStatus`, `GameView`, `TouchGesture`, `PointerSystems`, `frame_settings`, `KeepDrawing`, `Session`, `MakingCharacter`, `CharacterSelect`), `dom` (`Template`, `Dom`, `document`, `DrawObjects` for ordering), `widgets` (`WidgetSystems`, for ordering; `layer_z`, `UiId`).

## Used by

- `dev_tools` (`HoldLoadingScreen`). `connection` reads `common::LoadingStep`.

## Notes

- Its `GlobalZIndex` is just under the first full-screen layer (`widgets::layer_z(0) - 1`), above everything else, so the system menu's layers can be the startup menu over it.
- To add a step for startup work: add it to `LoadingStep`, then give it a place in `next_step` and words and a fill in `bar_shows`. The bar's positions are fixed for now, not timed.
- `connection` waits out `Starting`, or the word would never be seen: it would connect before the first frame is drawn.
- The title's font is built in with `include_bytes!` to be there on the first frame. `TITLE_EMS_WIDE` is measured from the font: change it with the font or the words. The font (`assets/fonts/HallOfWorldsTitle.ttf`) has only the title's letters: other words need it cut again (`ATTRIBUTION.md`).
- While up, frames keep their focused rate even out of focus (`common::frame_settings`), as they do afterwards too while `common::KeepDrawing` is on.
- On the web, `index.html` has its own copy of this screen saying "Downloading...", for until the wasm has loaded and started. `remove_page_screen` removes it when `Starting` ends, so there is no gap. Its colours, sizes and words are copied: change them in both places. `#ended` in the same page is the same screen for after quitting (`system_menu`). Its fonts come from the same server as the page, and its title stays hidden until its font has loaded.
