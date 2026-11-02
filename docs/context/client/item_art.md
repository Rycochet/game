# client: item_art

Pictures of items: each look is drawn once in 3D by a camera of its own, off screen, and kept as an image for the interface to show.

## Files

- `crates/client/src/item_art/mod.rs`: `ItemArtPlugin`, `Look`, `look_of`, `ItemPictures`, the render queue, the lights and each picture's camera.
- `crates/client/src/item_art/gem.rs`: `gem_parts`: colours, materials and meshes of gems (seven cuts from one builder, uncut as a lumpy sphere). Unit tests.
- `crates/client/src/item_art/ring.rs`: `ring_parts`: a cross-section swept round a circle for the band, patterns, mounts, and gems placed in settings, all tipped back together. Unit tests.

## Public API

- `ItemArtPlugin`.
- `Look`, `RingLook`: everything a picture shows, and so what pictures are kept by. Ids stay out of it, so things that look the same share a picture. `look_of(item)`: an item's, if it has a form.
- `ItemPicture { look, pixels }` (component): on an `ImageNode`, it shows that picture. `widgets::item_picture` is the usual way.
- `ItemPictures` (resource): `picture(look, pixels, images)` gives the image straight away, empty until drawn, and the same one if asked again.

## Depends on

- `shared::item`, `shared::item_form`, `shared::material`, `shared::rng`. No other module.

## Used by

- `debug_menu` (Item Pictures), `widgets` and `inventory` (grids and the clothes panel).

## Notes

- Colours and finishes are the client's, and meshes aren't rolled: everything to do with what a thing is comes from its saved form, so the same item looks the same anywhere, though pixels may differ between graphics cards.
- Up to `SLOTS` pictures draw at once, each camera on a render layer of its own (16 up), so a panel full of new things doesn't stall a frame. The rest wait their turn.
- Something new to draw isn't drawn until the graphics card's program for it has compiled, so cameras stay until `StillCompiling` is 0 and then a few frames more. Without that the first pictures after starting come out empty. Same idea as Bevy's loading screen example.
- Three directional lights (owner's idea: strongest from the top right, so no face is black and metal has something to catch), shared and there only while drawing, as Bevy allows 10 directional lights in all. Bevy skips a light for a view not on its layers, so these and the world's don't mix, and each camera has its own `AmbientLight`.
- A cut gem is one mesh drawn twice: inside of the far faces opaque and darker, then the near faces 40% see-through (owner's request). Being convex, its faces can't sort wrongly. No refraction: it costs a lot, and flat faces of varied shade read well at this size.
- Better quality means more contrast between faces and a sharper shine.
- Every picture has a black edge (owner's request), 4 of its pixels, so 2 as shown: `OUTLINE_COPIES` black copies of each mesh, moved round and behind it. It goes by what the camera sees, so any model gets it with nothing of its own. A model must leave that much room inside `VIEW_WIDTH`.
- Pictures are drawn at twice the size they are shown, which is what that thickness assumes.

- A picture is square, and is stretched to fit a shape that isn't: only square kinds have models so far.
- Size isn't in `Look`: a picture fills what the item takes up, at any size. Only sizes against each other show: a setting against its ring, a gem against its setting, which is how a gem too big looks it.
- Metal is half metallic rather than fully: fully metal shows only what it reflects, and nothing is round it to reflect. Patterns are cheap: twisted turns the cross-section on its way round, hammered tilts each small face's normal, engraved darkens bands of faces.
- Pictures are kept in memory for good, for now. Keeping them on disk is planned (`decisions.md`, 2026-11-02, generated items).
