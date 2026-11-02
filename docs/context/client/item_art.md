# client: item_art

Pictures of items: each look is drawn once in 3D by a camera of its own, off screen, and kept as an image for the interface to show.

## Files

- `crates/client/src/item_art/mod.rs`: `ItemArtPlugin`, `Look`, `look_of`, `ItemPictures`, the render queue, the lights and each picture's camera.
- `crates/client/src/item_art/gem.rs`: `gem_parts`: colours, materials and meshes of gems (seven cuts from one builder, uncut as a lumpy sphere). Unit tests.

## Public API

- `ItemArtPlugin`.
- `Look`: everything a picture shows, and so what pictures are kept by. Ids stay out of it, so things that look the same share a picture. `look_of(item)`: an item's, if it has a form.
- `ItemPictures` (resource): `picture(look, pixels, images)` gives the image straight away, empty until drawn, and the same one if asked again.

## Depends on

- `shared::item`, `shared::item_form`, `shared::material`, `shared::rng`. No other module.

## Used by

- `debug_menu` (Item Pictures).

## Notes

- Colours and finishes are the client's, and meshes aren't rolled: everything to do with what a thing is comes from its saved form, so the same item looks the same anywhere, though pixels may differ between graphics cards.
- Up to `SLOTS` pictures draw at once, each camera on a render layer of its own (16 up), so a panel full of new things doesn't stall a frame. The rest wait their turn.
- Something new to draw isn't drawn until the graphics card's program for it has compiled, so cameras stay until `StillCompiling` is 0 and then a few frames more. Without that the first pictures after starting come out empty. Same idea as Bevy's loading screen example.
- Three directional lights (owner's idea: strongest from the top right, so no face is black and metal has something to catch), shared and there only while drawing, as Bevy allows 10 directional lights in all. Bevy skips a light for a view not on its layers, so these and the world's don't mix, and each camera has its own `AmbientLight`.
- A cut gem is one mesh drawn twice: inside of the far faces opaque and darker, then the near faces 40% see-through (owner's request). Being convex, its faces can't sort wrongly. No refraction: it costs a lot, and flat faces of varied shade read well at this size.
- Better quality means more contrast between faces and a sharper shine.
- Every picture has a black edge (owner's request), 4 of its pixels, so 2 as shown: `OUTLINE_COPIES` black copies of each mesh, moved round and behind it. It goes by what the camera sees, so any model gets it with nothing of its own. A model must leave that much room inside `VIEW_WIDTH`.
- Pictures are drawn at twice the size they are shown, which is what that thickness assumes.
- Size isn't in `Look`: a picture fills what the item takes up, at any size.
- Pictures are kept in memory for good, for now. Showing them in grids, and keeping them on disk, are planned (`decisions.md`, 2026-11-02, generated items).
