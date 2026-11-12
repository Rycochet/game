# Free positions, smooth ground, sizes, reach, 3D creatures, facing: plan

Owner's request (2026-11-10), answers agreed 2026-11-10/11. For sites, the wilderness and the Hall; the world map stays on grid points (it is the quick travel).

## Foundations

- Positions are whole numbers, **240 units a cell** (divides by wall shapes' 40, the wilderness's 16ths, sizes' quarters).
- A creature's footprint is a circle by `Size::quarters`: Medium (today's base, 2/3 cell across) radius 80; Tiny 20, Small 60, Large 120, Huge 180, Giant 240. Drawn base = blocking circle.
- Clients send **move segments** (point to point) instead of one-cell `Step`s on levels. Shared rule: a moving circle against blockers; client slides and predicts, server checks. Energy by distance, slopes extra. A jump is a segment over one jumpable cell.
- Routes: A* on cells with a room map per size, then straightened to any angle. Mobs keep cell brains (distance maps, scent, guard spots), bodies on positions.

## Owner's decisions

1. Blockers: whole wall cells as now, plus thin ones (door frames, natural windows' columns, fences, tree trunks, corner squeezes). Playtest will tell.
2. Squeezing: through a gap down to half its own width; speed divided by (wanted / available)², so at most 4× slower. Route costs per size use it. Large gets through a door slowly; Huge needs 0.75 cell (depends on door frame width, check); Giant never.
3. Field of view is a **game rule**: a cone ahead plus a small circle all round; mobs notice by theirs; server sends by it. Shared, so a future ability can draw an enemy's arc. Reverses "nothing in the game faces a way" (2026-11-09): owner wants a wider audience. Record in decisions.md.
4. Players pass through players; non-enemies can be squeezed past slowly ("excuse me"); enemies block.
5. A site's first level has the same slopes as its wilderness cutout: one shared rule, tested to agree (owner thinks it may already; make explicit).
6. Turning: free and at once. To the target when using an ability; towards the point where the FAB menu opens (right click / long press: the FAB menu chat's work; if not merged, add a "face this point" message for it to call); by moving (turn camera, nudge forward); **Turn Left / Turn Right** bindings, empty by default, apart from the camera's (gamepad later). Mobs turn to noises and blows.
7. Movement pad rows become: Turn Left · Step Up · Turn Right / Step Left · gap · Step Right / Down · Step Down · Up. Turn buttons are hold buttons with curved arrows drawn in code (`step_pad/turn_left`, `turn_right`), same as the bindings: a tap 45°, held about half a turn a second; character only, not the camera; greyed on the world map.
8. Cones: width and near circle per kind from the research (about 120° for people, wider for prey; near circle 1 cell), dark sight inside the cone. `survival/observation` widens the cone and lengthens sight a little, practised by seeing with it. First guesses.
9. World map: models turn to face their way; sight all round, no cone.
10. Kick shoves ½ cell; Heal's range and "beside a shop's door" become distances.
11. Open door leaves don't block; only closed doors.
12. Height doesn't block sight (stays the owner's TODO: seeing someone from the waist up is too complicated for now).
13. Old saves: cell positions load as cell centres; bags on the ground too. No wipe.
14. Network: movement sent as stretches ("from here to there, from this tick, at this speed"), only when the stretch changes; one message per client per tick with every creature's changes (check whether already so); headings from the stretch, sent alone only for turning in place. Mobs can't be worked out on the client (secret seeds).
15. Later, not here: animation (models ready for it), gamepad analogue movement, drawing enemies' cones.

## PRs

1. **Smooth 3D ground** (built: `shared::relief`, `common::DrawnGround`, `dungeon_map` draping). Shared height at every grid corner: wilderness and cutouts from the world map heights with flats kept level; buildings flat; caves a gentle undulation from a position hash, damped to near flat inside caves with a `RoomId`, stairs cells and pool surfaces level. Dungeon view draws triangulated slopes instead of raised blocks and edge faces; walls, liquid, stairs, tokens, camera, clicks follow. Independent.
2. **Shared foundations, no behaviour change** (built: `shared::spot`, `shared::clearance`, `pathfinding::find_sized_path`; see `shared/clearance.md`): `Spot` (units), whole-number distance, `Size::radius`, blockers from a `Level`, `segment_is_legal`, `slide`, room maps per size, any-angle straightening. Golden tests.
3. **Positions in units end to end** (built: `ClientMessage::Move`, `ServerMessage::Moves`, `movement::move_on_level`, `protocol::moves`; see decisions.md): protocol (segments, stretches, one message a tick), server visits, saves (old cells as centres), stairs and shops by distance, dev teleports. The client still sends centre-to-centre moves, so play is the same. Left for PR 4 and 5: slopes from `relief` in costs, swimming and the mobs' World Gate rule for free segments, exits, borders and World Gates by distance.
4. **Free movement on the client** (built: `common::SpotPosition`, `common::ViewTurn`, `player`'s `Walking`, `Moved`, stride and level travel; see decisions.md): keys, pad, touch at any angle with sliding; click travel on any-angle routes; prediction and corrections on positions. Left for 5: relief slopes in costs, swimming on free segments, mobs' World Gate rule.
5. **Sizes in play** (built: `clearance::crowd`, `movement::mob_step_sized`, sized `DistanceMap`s, mobs' `spot`, `Token::base`, `common::PlayerSize`; see decisions.md): bases to scale, creatures blocking (decision 4), squeezing (decision 2, charged on energy), mobs' bodies on positions, routes per size, Kick's half-cell shove. Left over: relief slopes in move costs, swimming on free segments.
6. **Bags at any position** (built: `shared::ground::in_reach`, `storage::SpotMap`, `GroundItems { bags, items }`; see decisions.md): dropped at the feet, joining a bag within reach; pick-up by reach; saved (old cells as middles). Also built: good swimmers swim water on free segments (`clearance::check_segment_by`). Left over, split off: relief slopes in move costs and level routes (the server has no `Relief` yet), swimmers' routes across water.
7. **Melee range by distance** (built: `combat::Body`, `in_reach`, `within`, `Population::relief` on the server, mobs' `close_in`; see decisions.md): attacker's radius + weapon length + arm's reach by size + target's radius, in 3D, with a clear line; mobs close to reach; client's automatic use; kick shove, Heal range (decision 10).
8. **3D creature models, both maps:** `item_art` body and worn builders as scene meshes, cached by look, rest pose with each vertex tagged by body part (for animation later), black outlines, x-ray; replacing flat pictures for the player, mobs, other players; world map too. Measure drawing cost in a busy town. Independent. Built: `item_art::CreatureModels`, `common::TokenModel`, `view3d`'s `model` and `hull`; models turn to the camera until PR 9 sets the model anchor's heading.
9. **Facing and field of view** (built: `shared::facing`, `Client::heading`, `Mob::heading`, `ServerMessage::Turned`, `ClientMessage::Face`, `common::Facing`, `PlayerCone`; see decisions.md): headings from movement or target, sent, models turning smoothly on both maps; the cone as a rule (server sight, mobs' notice, Observation); drawn with the sight mask; turning (decision 6), the pad's new layout (decision 7), Turn Left / Right bindings; wiki controls page.
10. **Slopes in move costs and routes, swimmers' routes** (built: `movement::LevelWay`, `energy::level_move_energy`, `sneak_move_energy`, `RoomMap::new_by`; see decisions.md): every move on a level, players' on both sides and mobs', priced on the smooth ground sampled along it (slopes less by Climbing, wading, terrain, jumps); Climbing practised by the same heights; sized routes weigh slopes and let good swimmers swim; the server's ground agrees with the client's but at cells the client hasn't seen (harmless, `shared/relief.md`).

The plan is finished: every PR above is built.

Order (owner, 2026-11-12): independent, so non-blocking children: 1 and 8 (need nothing), 6 (once 3 is merged), 7 (once 5 is merged). Blocking, in order: 2 → 3 → 4 → 5. Last: 9 (needs 5 and 8, and every parked PR answered and merged). One child per PR, all started on 2026-11-12 and waiting for the parent's "Start".

Each PR updates its context files and decisions.md. Replaces recorded decisions: raised-block wilderness, camera-facing flat icons, "playable kinds all stand in one cell".

Note: the HTML interface chat is rewriting the interface (step pad, key bindings screens): re-check PR 9's pad and bindings work against whatever has merged by then. The movement pad's buttons are markup since #423: add the turn buttons that way.
