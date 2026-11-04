# shared: road

Roads on the world map between towns, from the world alone, so the order anything is looked at doesn't matter. Towns come from the secret pois seed, so only the server works roads out (`server::roads`) and sends them; the client draws what it is sent (`world_map`). Towns are seen by all, so roads give nothing away. Steps along a road are faster: 1.25× on paths up to 2× on highways, the larger road counting where they share a step; both sides charge by the same `RoadSteps`, and the server checks every step.

## Files

- `crates/shared/src/road.rs`: `Road`, `RoadLink`, `Roads`, constants. In `road/`: `kind.rs` (`RoadKind`, `FASTEST_PERCENT`), `network.rs` (which towns are joined), `route.rs` (the A* the road follows), `steps.rs` (`RoadSteps`).

## Public API

- `RANGE` (6 chunks, the owner's choice), `MARGIN` (one chunk: how far a road may stray outside its towns' box), `ROAD_VERSION` (bump when roads change, so saved ones are worked out again).
- `RoadKind`: `Path`, `Track`, `Road`, `Highway`, ordered smallest first (`max` is the larger); `ALL`, `of_town`, `between` (the smaller town decides), `speed_percent` (125, 150, 175, 200: owner's figures), `slope_percent` (100, 93, 87, 85). `FASTEST_PERCENT` (200: what route searches must allow for).
- `RoadSteps::new(world_width)` (`Default`: no wrapping, for tests), `.add(&road)` (the larger kind wins a shared step), `.kind_of(from, to)` (only a step between consecutive points of a road, either way round, on any copy of the planet; crossing a road doesn't count), `.is_empty()`.
- `RoadLink { a, b }`: the two towns' chunks, smaller first (`new` sorts). `Road { link, kind, points }`: grid points one step apart from town `a`'s poi to `b`'s, `x` not wrapped; `bounds()`.
- `Roads::new(&WorldParams)`, `.links_in_area(pois, min, max) -> Vec<(RoadLink, RoadKind)>` (every link whose road may pass through the area, sorted), `.road(pois, link) -> Option<Road>` (`None` with water between).

## Depends on

- `shared::movement` (`WorldGround`, `WorldConditions`, the step and slope costs), `shared::poi` (`PoiMap`, `ChunkPos`, `CHUNK_SIZE`), `shared::showcase` (its plain has no roads), `shared::world`, `serde`.

## Used by

- `shared::movement` (`world_step_cost`), `shared::pathfinding` (`find_world_path`), `server` (`roads`: the road thread, saves, `connections::send_roads`, `ServerWorld::road_steps`), `shared::protocol` (`ServerMessage::Road`), client `common` (`KnownRoads`), `world_map` (drawing, clearing woods), `minimap` (lines on the maps).

## Notes

- Which towns: two within `RANGE` are joined unless a third town is nearer both than they are to each other (a relative neighbourhood graph), so no long roads run beside shorter ones. A link only depends on towns within `2 × RANGE + MARGIN` of the area, which is why an area's answer doesn't depend on how it was cut up (tested).
- The route is A* with the walking rules' costs in a box `MARGIN` round both towns, at high tide and with nothing frozen, so the tide and weather never cut a road. Larger roads count slopes for less (`slope_percent`; owner: highways 15% more likely to change elevation; 20% made them too straight), so they run straighter. Roads never step onto water. Town `b` is taken the nearer way round the planet.
- Overlapping roads aren't merged: each is traced alone and they share points where the land leads them the same way. Where they overlap, the larger is drawn on top (owner's rule) and counts for speed.
- On the test world an area 12 chunks across has about a dozen roads, worked out in about 0.3 s together on the test profile.
- Speed: a road's step costs `world_step_cost` divided by its speed, slope included. The server only knows roads its road thread has found, and a client only those it was sent: if the client hasn't been sent one yet it charges itself the full cost and is merely slower; if it thinks a step cheaper than the server does, the server's energy check makes it wait, so nothing is trusted.
