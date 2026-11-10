# client: app_icon

The game window's icon (native): the stylised HW, a large bronze W behind a smaller gold H, from the title font. The pictures are drawn by `cargo xtask app-icon` (`xtask/src/app_icon.rs`, design in `app_icon/svg.rs`) into `crates/client/assets/app_icon/` and kept in git. The program file's icon comes from `crates/client/build.rs` (`icon.ico`), and the web page's from `index.html` (`icon-<size>.png`, `apple-touch-icon.png`), not from this module.

## Files

- `crates/client/src/app_icon/mod.rs`: `AppIconPlugin`, which runs nothing on the web.
- `crates/client/src/app_icon/set.rs`: `set_window_icon`, through winit (Bevy has no setting for it), once the primary window exists. On Windows it is loaded from the program file's resource (id 1) for the title bar and the taskbar. Elsewhere it is `icon-128.png`, built in with `include_bytes!`.

## Public API

- `AppIconPlugin`.

## Depends on

- Nothing of the game's: Bevy's `WINIT_WINDOWS` and the `winit` crate (native only, the same version as Bevy's).

## Notes

- macOS ignores window icons. It would need an app bundle with an `.icns`, which we don't build.
- To change the design, edit `xtask/src/app_icon/svg.rs` (colours, the H's size and place, the ring round it, the shadow), then run `cargo xtask app-icon` and commit the pictures.
