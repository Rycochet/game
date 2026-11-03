# client: item_art

Pictures of items: each look is drawn once in 3D by a camera of its own, off screen, and kept as an image for the interface to show.

## Files

- `crates/client/src/item_art/mod.rs`: `ItemArtPlugin`, `Look`, `look_of`, `ItemPictures`, the render queue, the lights and each picture's camera.
- `crates/client/src/item_art/gem.rs`: `gem_parts`: colours, materials and meshes of gems (seven cuts from one builder, uncut as a lumpy sphere). Unit tests.
- `crates/client/src/item_art/ring.rs`: `ring_parts`: a cross-section swept round a circle for the band, patterns, mounts, and gems placed in settings, all tipped back together. Unit tests.
- `crates/client/src/item_art/body.rs`: `body_parts`: a plain standing body from `shared::body::Frame`, slightly turned and seen a little from above, filling a 1 × 2 picture. Unit test.
- `crates/client/src/item_art/armour/mod.rs`: what every location's models share: `Stuff` (parts by what they are made of), `Model`, `plate` (a shell from rows of points and normals), `face`, `colour_of`, `matte`, `darker`.
- `crates/client/src/item_art/armour/hands.rs`: `build_hand` / `build_pair` (both hands in place on a `Frame`, for sets and the character picture), `material_of`; `hands_parts` lays a pair down for its picture. Unit tests.
- `crates/client/src/item_art/armour/shoulders.rs`: `build_pair` (both shoulders in place on a `Frame`, for sets and the character picture); `shoulders_parts` sets the two pieces side by side for the picture. Unit tests.
- `crates/client/src/item_art/blade.rs`: `blade_parts`: weapons from their rolled form, built in millimetres from lofted sections and tubes, then fitted to their cells. Unit tests.

## Public API

- `ItemArtPlugin`.
- `Look::Hands { hands: HandsLook, quality }`, `HandsLook::new(form, digits, cells)`: a pair of hand pieces; how many digits each hand has shows.
- `Look::Shoulders { shoulders: ShouldersLook, quality }`, `ShouldersLook::new(form, cells)`: a pair of shoulder pieces.
- `Look::Body { kind }`: a bare body, for kinds with a frame; worn things will be added to it for a character's picture.
- `Look`, `RingLook`, `BladeLook`: everything a picture shows, and so what pictures are kept by. Ids stay out of it, so things that look the same share a picture. `look_of(item)`: an item's, if it has a form. `Look::cells`, `longest`, `picture_size(pixels)`: its proportions.
- `ItemPicture { look, pixels }` (component, `pixels` along the longest side): on an `ImageNode`, it shows that picture. `widgets::item_picture` is the usual way.
- `ItemPictures` (resource): `picture(look, pixels, images)` gives the image straight away, empty until drawn, and the same one if asked again.

## Depends on

- `shared::item`, `shared::item_form`, `shared::blade`, `shared::armour`, `shared::body`, `shared::creature`, `shared::material`, `shared::rng`. No other module.

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

- A picture has the proportions of what its item takes up, not turned: square for gems and rings, long for weapons. It must be asked for at 64 pixels per cell of its longest side for its black edge and margin to be right. A picture turned a quarter is laid out the other way round about the same middle (`widgets::picture_node`). Where one cell shows it, `widgets::picture_in_square` keeps its proportions.
- A weapon points up with its edge to the right, and its length is squeezed to fit but its width much less (one cell shows at most 15 cm across), or a pike would be a black line. It is placed by its bounds, so anything rolled fits, which a test checks. Lengths in `BladeLook` are to the nearest centimetre, so that nearly the same weapons share a picture. No fine curves on heads: at this size their outlines are what show.
- Size isn't in `Look`: a picture fills what the item takes up, at any size. Only sizes against each other show: a setting against its ring, a gem against its setting, which is how a gem too big looks it.
- Metal is half metallic rather than fully: fully metal shows only what it reflects, and nothing is round it to reflect. Patterns are cheap: twisted turns the cross-section on its way round, hammered tilts each small face's normal, engraved darkens bands of faces.
- Pictures are kept in memory for good, about 16 KB each at one cell, and drawn again next time the game starts. Keeping them on disk was measured and not done: see `optimisations.md`.
- Armour is built in millimetres in place on `shared::body::Frame`, with parts by what they are made of (`armour::Stuff`), so sets and the character picture can reuse the same builders. `body.rs`'s `loft`, `limb`, `blob` and `mesh_of` are shared within `item_art` for this, and `trunk_half` / `trunk_depth` give the trunk's surface as drawn, for armour to keep clear of.
- Shoulders: surfaces are functions of two numbers giving a point and its normal, from which plates, bands of lames or lamellae, scales, studs and mail lines are made. The cop is a dome over the shoulder joint, lames are bands round the upper arm flaring out, wings follow the trunk 45 mm out; rigid pieces sit 35 mm out from the skin (outside breastplate and gorget), hanging guards 28 mm out from the arm, capes 22 mm. Styles shape plate as on gauntlets. Japanese lamellae are lacquered black; ailettes are painted in their heraldry. The picture shows each piece of a pair turned a little to show its outside, side by side; capes and the wing as worn.
- Hands: each hand hangs from the wrist, palm to the thigh, thumb forwards. Plates are thin shells (`plate`) from rows of points with their normals; lames step out 1.4 mm over the next so their edges show; mail and quilting are shaded in alternate strips. A cuff is 28 mm out from the forearm, over a vambrace. Styles change plate shapes: flutes (Gothic, Maximilian), points and cusps (Gothic, Elven), square cuffs (Kastenbrust), thick plates and a heavy knuckle bar (Dwarven), etched bands (Greenwich). The picture lays both down, fingers to the right, the right hand above with its thumb up, tipped back a little, and fits them by their bounds, as a test checks.
