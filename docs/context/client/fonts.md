# fonts

Sets the game's default font, Lexend, used by all text with no font of its own. Not font sizes (the modules showing text choose those, `widgets` for its pieces), nor the loading screen's title font (`loading_screen` owns that).

## Files

- `src/fonts/mod.rs`: FontsPlugin; embeds `assets/fonts/Lexend-Variable.ttf` and puts it at the default font id

## Public API

- `FontsPlugin`, nothing else: other modules use `TextFont::from_font_size(...)` as usual, and can pick a weight with `with_font_weight`.

## Depends on

- Nothing but Bevy.

## Used by

- `main.rs`: registers the plugin, straight after `DefaultPlugins`.

## Notes

- Must be added after `DefaultPlugins`, whose text plugin puts Bevy's own font (Fira Mono) at the same id. That one stays compiled in, as turning off `default_font` would mean listing all of Bevy's other features by hand, but is never used.
- Lexend covers Latin, extended Latin and Vietnamese, with no Greek, Cyrillic or CJK.
- Lexend is not monospaced, so changing numbers move the text after them.
