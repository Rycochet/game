# shared: road

Roads on the world map between towns, from the world alone, so the order anything is looked at doesn't matter. Towns come from the secret pois seed, so only the server works roads out (`server::roads`) and sends them; the client draws what it is sent (`world_map`). Towns are seen by all, so roads give nothing away. Only something to look at for now: PR 3 of the roads plan makes them speed up travel (1.25× to 2×, checked by the server step by step).

## Files

- `crates/shared/src/road.rs`: `Road`, `RoadLink`, `Roads`, constants. In `road/`: `kind.rs` (`RoadKind`), `network.rs` (which towns are joined), `route.rs` (the A* the road follows).

## Public API

- `RANGE` (6 chunks, the owner's choice), `MARGIN` (one chunk: how far a road may stray outside its towns' box), `ROAD_VERSION` (bump when roads change, so saved ones are worked out again).
- `RoadKind`: `Path`, `Track`, `Road`, `Highway`, ordered smallest first (`max` is the larger); `ALL`, `of_town`, `between` (the smaller town decides), `slope_percent` (100, 93, 87, 80).
- `RoadLink { a, b }`: the two towns' chunks, smaller first (`new` sorts). `Road { link, kind, points }`: grid points one step apart from town `a`'s poi to `b`'s, `x` not wrapped; `bounds()`.
- `Roads::new(&WorldParams)`, `.links_in_area(pois, min, max) -> Vec<(RoadLink, RoadKind)>` (every link whose road may pass through the area, sorted), `.road(pois, link) -> Option<Road>` (`None` with water between).

## Depends on

- `shared::movement` (`WorldGround`, `WorldConditions`, the step and slope costs), `shared::poi` (`PoiMap`, `ChunkPos`, `CHUNK_SIZE`), `shared::showcase` (its plain has no roads), `shared::world`, `serde`.

## Used by

- `server` (`roads`: the road thread, saves, `connections::send_roads`), `shared::protocol` (`ServerMessage::Road`), client `common` (`KnownRoads`), `world_map` (drawing, clearing woods), `minimap` (lines on the maps).

## Notes

- Which towns: two within `RANGE` are joined unless a third town is nearer both than they are to each other (a relative neighbourhood graph), so no long roads run beside shorter ones. A link only depends on towns within `2 × RANGE + MARGIN` of the area, which is why an area's answer doesn't depend on how it was cut up (tested).
- The route is A* with the walking rules' costs in a box `MARGIN` round both towns, at high tide and with nothing frozen, so the tide and weather never cut a road. Larger roads count slopes for less (`slope_percent`; owner: highways 20% more likely to change elevation), so they run straighter. Roads never step onto water. Town `b` is taken the nearer way round the planet.
- Overlapping roads aren't merged: each is traced alone and they share points where the land leads them the same way. Where they overlap, the larger is drawn on top (owner's rule), and later counts for speed.
- On the test world an area 12 chunks across has about a dozen roads, worked out in about 0.3 s together on the test profile.
