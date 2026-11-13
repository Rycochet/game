# Quests (plan)

The plan for the quest system, in 12 PRs (0 to 11), planned with the owner on 2026-11-11 and 2026-11-12. PR 0 (coins), PR 1 (this note), PR 2 (`shared::quest`, see `shared/quest.md`), PR 3 (the server, `server/server.md`'s `quests`), PR 4 (the talk panel and markers: `client/talk.md`, `client/quests.md`), PR 5 (the Quest Log and the O key: `client/quest_log.md`), PR 6 (the tracked quest and the maps' dots: `client/tracked_quest.md`, `client/minimap.md`, `server/server.md`'s `quests/dots.rs`) and PR 7 (7a quest bosses and "Slay" top-ups: `server/server.md`'s `quests/bosses.rs` and `entities/quest_mobs.rs`, `client/quests.md`; 7b quest items in four parts) are built; the rest is not. **Owner** marks the owner's decisions; **Claude** marks Claude's proposals the owner accepted as they were. Quests' text is a plan of its own: `story-text.md`.

## Goal

Quests from talking to the people of a town, sending a character to nearby places or into the wilderness: slaying, exploring, finding, delivering, gathering, escorting. Generated in game, the same for everyone, each with a name, steps, a reason told in flavour text and a reward, chained across towns and leading, in cities, to artifacts and the planet's set item.

## How it works

- **Generated, deterministic, by the server.** A town's quests come from its poi's map seed (`Seed::derive("quests")`, drawing nothing from the site generator's sequences), identical for everyone (**owner**). Only the server works them out, as the pois seed is secret (as with place names and shop stock); clients are sent offers, their log, markers and progress, and the server checks everything.
- **`shared::quest`**, plain Rust: the model, the generator, the step graph and poi hooks (built: `shared/quest.md`, with the figures Claude chose: slots, radii, chain lengths, rewards).
- **The owner's own notes (Questing):** quests come from townspeople and shopkeepers, more in bigger towns; targets are a nearby poi or actions in the wilderness; **a poi may serve several quests, but they must not overlap or interfere**; the whole quest, text and targets come from its seed, placed by its town.
- **Not interfering (Claude):** each poi has fixed hooks (a boss place, an item place, counted kills per mob kind); each hook belongs to one quest only, by a hash, so two towns' quests never claim the same boss or item spot. Counted kills may be shared (top-ups per quester, below).
- **Text is not `shared::quest`'s (owner, agreed with the Flavour text chat):** quest names, step words, descriptions, reasons and NPC names come from `shared::story` (Flavour text PR 3) and its names (PR 2). The server sends a language-neutral `Story`; the client writes it. Townsfolk's gender is `shared::gender::Gender`. Details: `story-text.md`.

## Givers and talking (owner)

- Givers are town buildings: shop and house doors to start with. No one inside yet: the door is the person. Clicking a door opens a **talk panel** ("Mara Tull, baker") with Quests and Trade tabs (shops already open trade by their door, near or far).
- Each building has quest slots by `hash(town seed, building id, slot)`; bigger towns have more givers and slots.
- A quest is offered; the player chooses whether to accept.
- **Markers over doors:** "!" a quest is available, "?" done but not turned in, coloured by kind; a **grey "?"** for a quest taken from them and not finished yet (owner, 2026-11-13). A giver with quests in several states shows one marker, the first of: "?" to hand in, grey "?", "!", grey "!" (owner, 2026-11-13). A **grey "!"** for quests the character can't take yet; talking to that giver gives a **hint in their words, never numbers**: standing ("Folk round here don't know you yet"), skill ("I'd want someone good with animals"), chain ("The smith in Little Fen spoke of you..."). Some requirements are hidden (a skill level, a moon in the sky) and show only as the hint.
- If the quest's dungeon is under the giver's own house (cellar, well, loft), the talk panel offers "Enter Cellar" (or its word), and accepting moves the player straight in.

## Generation

1. Givers and slots, as above.
2. Nearby pois: `PoiMap::in_area` within a radius by town size (about the road `RANGE`, 6 chunks, for a town; less for a hamlet; dailies nearer), nearer ones likelier.
3. A template whose needs the area meets, by poi kind (cave: clear it; crypt: fetch from it; farm: deliver; herbalist: a remedy; wilderness: a wolf pack near town...).
4. Roles: giver, target pois, creature kind, item, a named person or pet, each by its own hash.
5. The story: a reason (**owner**: rumour, family, neighbour feud, lost pet, debt or trade, threat to the town, grief, and Learning) and beats, through `shared::story`.

## Shape

- `Quest`: id, kind (Normal, Daily, Artifact), giver, reason, story, requirements, steps, reward.
- `Step`: a goal, an optional count, the steps it comes after, and alternatives.
- `Requirement`: standing in a town (a tier), another quest done, a skill known, an alignment band; some hidden.
- Every quest has a display name, every step a couple of words with an optional count ("Slay kobolds 2/6"), and a longer description with its reason (**owner**).
- **Steps form a graph:** in series, in parallel or both (**owner**). A final "Return to <giver>" step; while it is the one left, the door shows "?".
- **Goals to start:** Slay (kind, count) at a poi, Explore (reach a poi's deepest level), Find (a quest item), Deliver (to a building or a neighbouring town), Visit or Scout, Gather (herbs), Escort.
- **Alternatives:** a step may be done more than one way, so future abilities never lock anyone out (taming the dog, or carrying it and getting bitten).
- Accepting reveals every target poi, known and named (`world_map::learn_place_name`). Pois with an active step have a marker on the world map and minimap (**owner**).

## Kinds (owner)

| Kind | Colour | Rules |
| ---- | ------ | ----- |
| Normal | yellow | once per character |
| Daily | blue | the giver offers the next one 24 hours (real time) after the last was turned in; an accepted daily is kept until turned in (players don't always control their time); the one on offer is the current UTC day's, `hash(town, giver, day)`, the same for everyone |
| Artifact | orange | cities only, Honoured standing or more; a long chain of whole quests across the city's road neighbours, taking a long time |

- The artifact line ends in an artifact or the planet's set item, by the city's seed. Which set item a planet has comes from the planet seed (decided 2026-11-02), so players can later find out what they aim for. A character who already has the planet's set item gets an artifact instead (one set item per planet per character).
- The final unique or set item quest is a large quest dungeon, not a poi.

## Standing, chains and alignment

- **Standing** per character per town, tiers Stranger, Known, Trusted, Honoured, Champion (Claude), from quests and helpful acts: killing hostile mobs within the town's radius, trading at its shops, giving to its beggars (`mob::Memory` records gifts) (Claude). Built in PR 3 (Claude's figures, to balance): Known from 15 points, Trusted 30, Honoured 75, Champion 150; a normal quest handed in 15, a daily 5, a hostile kill within a town's reach 1, kills alone up to 15 (Known); trading and gifts are left for PR 10.
- **Chains in a town:** a shared seed and cast, N+1 needs N; "I heard you helped X" is a quest done for another giver (**owner**).
- **Across towns (owner):** commoner from smaller to larger, and with no recursive generation, only town size by step: a quest may only require one from a **smaller** town joined by road (ties by chunk order), so City → Town → Village → Hamlet, at most four deep, and each town's quests come from its own seed alone. The smaller town has one **feeder** quest, fixed by its own seed, which may name the larger town (its name only, not its quests).
- **From the plot research (owner):** dark quests (murder, revenge, theft, framing, silencing) only for characters with a bad alignment, and a quest with an alignment requirement is **hidden entirely** (no grey "!", no hint); twists and moral quandaries only in later chain steps, and finishing them may move alignment; Learning is an eighth reason, and training a skill can be a direct reward; the text names the real reward; checkable season and weather beats only in higher-reward quests.
- **Alignment** is `shared::alignment` (`alignment.md`): requirements go by `Alignment::band()`, deeds are applied with `Alignment::after` (Rescued, Escorted, Helped, BetrayedQuest), and it is never sent to clients. PR 2 leaves room for both; PR 10 applies them.

## Limits, failing, abandoning (owner)

- Quest lists are **per planet**: the cap (about 20, Claude) counts each planet separately, and every accepted quest counts, dailies included.
- Quests can fail: time limits, a quest item destroyed, an escort dead, the giver's own condition. Failed or abandoned, progress resets and every quest item vanishes wherever it is (carried, in a bag, on the ground), so it can't be restarted almost done. It can be taken again after at least an hour.
- **Rewards are all or nothing:** if the player can't take all of it (no room, say), it stays with the giver and the quest stays unfinished ("?", progress kept) until they can; the giver says why.

## Quest items and mobs

- **Items are per character; mobs are seen by everyone (owner).**
- Every quest item carries a tag naming its quest (`Item::quest`, built in 7b1), shown as "Quest" in the future item details tooltip (**owner**). Worthless otherwise: no shop buys them, they can't be given or traded, and dropping one destroys it after asking (Claude; built in 7b1). They vanish when their quest fails, is given up or is handed in.
- **Deliveries (owner, 2026-11-13):** the giver hands over a parcel on accepting (refused if it doesn't fit), drawn as a cube in brown paper tied with string; arriving in the town with it delivers it (built in 7b1).
- **Unique quest items (owner):** in their site from the start, seen by anyone without one (each character finds their own copy, so one player's pick-up takes nothing from another), with a red "!" for those without the quest. Picking one up marks the giver's town and door, and the giver offers the quest with "I've already found it", that step done.
- **Quest drops (owner,** such as "toenail clippings from a zombie"**):** dropped only for characters with that step, from that kind, until the count; nobody else's loot changes; no red "!". Built in 7b3 as a Collect step (`Hook::Collect(kind)`, worded "Gather kobold scales 2/5"), each part natural to the creature (owner: only where its body has it), going straight into the quester's pack; Slay top-ups bring its creatures too.
- **Lone quest bosses (owner):** a step naming one particular mob, part of its site from the start and seen by everyone. A red "!" over it for anyone without the quest who hasn't killed it, the quest's colour for those with the step. Killed without the quest, the kill is remembered on the character, the giver's town and door are marked (the town learned and named), and the quest is offered with "I've already dealt with it", that step done. If standing or a chain doesn't allow the quest yet, the kill is still remembered and the giver's hint shows they know (Claude). Kept until turned in; failing or abandoning wipes it (Claude). Never for counted kills or ordinary mobs.
- **Quest mobs coming back (Claude, owner: good):** if someone with the step, or someone who hasn't killed it yet, enters and it isn't alive, it appears; everyone on the step who hurt it gets the kill; others get ordinary loot only; it goes when nobody is near.
- **"Slay X" (owner):** counts the site's ordinary mobs; when a character with the step enters, extras of that kind spawn until there are enough for what they still need (seen by all, anyone may kill them), unless the step says otherwise ("one a night").

## Escorts (owner)

- The escorted creature is a shared mob inside sites; if you lose it, an arrow on the minimap points to it.
- It follows you onto the world map, where nobody else sees it (drawn for you only, as a small token behind yours: Claude).
- World map speed is the slower of the two: your skill bonuses don't count unless it has them too. Roads speed both.

## Quest dungeons (owner)

- Some quests make a dungeon of their own from the quest's seed (the building and cave generators), sized by the giver's town and the quest's difficulty (a village cellar of rats: one small level; a city's job: several). Commoner for dailies; normal quests mostly use real pois.
- **One copy per character** for now (sharing with a party later: `todo-party-quest-instances`), made on entering.
- Way in: a cellar, well or loft under the giver's house through "Enter Cellar"; larger ones by a hidden entrance near the town that only the quester sees, marked like any quest poi (Claude).
- Leaving is allowed any time. **About an hour after leaving it resets** (a fresh copy from the same seed). **Logging out inside keeps it:** the server records that the character is inside and saves the copy, with no timer running while offline.
- **A reset wipes the progress made inside it:** its kills, and quest items found there vanish wherever they are (keys too). Steps done elsewhere stay.
- **Stages behind locked secret doors:** a quest dungeon may come in stages separated by locked secret doors. The key is a quest item (a stage boss's drop, or given by the giver with the door's whereabouts); until you go through, the door has the quest's marker and a minimap dot. Lockpicking may be another way later. Doors have no locks yet: PR 8 adds locked doors and keys (`ItemKind::Key` exists).

## The Hall, the Quest Log, tracking (owner)

- A "!" over every Hall gate leading to a planet with unfinished quests (a planet has up to 16 World Gates); "?" when one there is ready to turn in (Claude, not objected to).
- **Quest Log** (built, PR 5): key **L**, its own moveable panel. Left: a planet dropdown at the top (only planets with quest history, the one the player is on first and also called "Here"; each named by its own 7-rune code, its first World Gate's code (Claude), drawn from rune strokes like the gates, never a font; the server learns a planet's code while the character is there and keeps it in their book, as only it has the seed), then the list grouped Active, Completed, Failed / Abandoned, newest first, each with its giver's marker as a badge (coloured "?" ready, grey "?" under way, the kind's "!" once over: Claude). Right: the details (name, giver and town, where it stands, description, steps with counts and state, reward, target pois still to do; clicking one opens the full map there, from the world map only), Track and Abandon (pressed twice: Claude) for quests under way on the planet the player is on. Another planet's log is made again by the server, opening that world if it must. Dailies' history is one line per giver, with a count and when last done (Claude).
- **Open/Close Doors on O** (built, PR 5; L was Close Doors): closes the open doors next to you, or else opens the closed one you face, else one beside you (Claude). Locking with a key now happens only alongside closing an open door (keys come in PR 8).
- **The tracked quest,** one per planet: chosen by a Track button in the log (the newest accepted if none is chosen: Claude), saved on the character (built in PR 5: `QuestAsk::Track`, `QuestBook::tracked`, the client's `QuestBook::followed`; pressing Track on the one chosen stops choosing it). Shown **under the minimap**: marker and name, the next objectives (steps whose earlier steps are done) with counts, then "Return to X" with "?"; clicking opens the log there.
- **Minimap dots** (built in PR 6; a boss's level and the boss itself once in sight in PR 7a, an item's in 7b): a coloured dot for **every objective you can do now**, behaving like the player's (a dot when on the map, an arrow at the edge when not), on the full map too. On the world map it marks the target poi; inside the site, the stairs towards the objective's level, then on that level the boss or item once seen; for Return, the town, then the giver's door.

## Coins (owner)

Built (PR 0, see `shared/item.md` and decisions.md): Copper Coin 1 g, Silver Coin 3 g, Gold Coin 5 g, 100 to one of the next, turning into the next kind past 99; mobs carry one amount in copper as the fewest coins. Quest rewards pay the same way: `shared::item::coin_stacks` and `coin_items`, then give each stack. Not in this plan: the coin pouch (the owner's: an item for more coin stacks), shop prices, moneylenders, changing coins.

## Abilities to plan for

Future skill-tree nodes quests will use through alternatives and skill requirements (hidden ones as hints): animal handling and taming (lost pets, livestock), tracking (scent trails exist), persuading and intimidating, lockpicking and disarming traps, herbalism (cures with Heal), deciphering (runes), climbing and swimming. Checks stay ratio contests (`abilities-skills.md`).

## PRs

| PR | What | Needs | Order |
| -- | ---- | ----- | ----- |
| 0 | **Coin items** (built, #447): Copper, Silver, Gold kinds turning into the next, mob spawns by copper amount, saved coins as copper | | independent of 1, 2 |
| 1 | **This design note** (built), decisions.md entries, CLAUDE.md row | | |
| 2 | **`shared::quest`** (built): model, generator (givers, slots, nearby pois, templates, roles), step graph, poi hooks, room for alignment requirements and changes; names and text through `shared::story`; golden tests and a print-to-text test | 1, Flavour text PR 3 | |
| 3 | **Server** (built): offers by door, accept, abandon, fail (an hour before retaking), turn in (all-or-nothing coin rewards), progress events (kills, visits, exploring, deliveries by arriving), per-planet lists and cap, standing from quests and hostile kills, character saves, protocol messages, `server.quests` / `server.quest` | 0, 2 | |
| 4 | **Client** (built): talk panel (markup) folding in the shop panel, door markers ("!", "?", grey "!", colours), poi markers (world map and maps), Hall gate markers (`QuestNews::Planets`) | 3 | |
| 5 | **Quest Log** (built): panel (L) with the planet dropdown and history, Track, grey "?" markers; Close Doors to O as Open/Close Doors | 4 | independent |
| 6 | **Tracked quest** (built): under the minimap, objective dots and arrows on both maps (`QuestNews::Dots` for levels) | 4 | independent |
| 7a | **Quest mobs** (built): lone bosses on their site's deepest level with red "!" or the quest's colour, kills remembered and "already dealt with it", coming back while wanted, slay top-ups, the boss's dots | 4 | independent |
| 7b | Quest items, in four PRs (owner's answers, 2026-11-13): 7b1 the tag and its rules, parcels handed over for deliveries (built); 7b2 unique items found in sites (rings and necklaces as their kinds, the rest one shared picture) with their dots (built); 7b3 a Collect step and drops natural to a creature (organs, nails, only with the right body part), listed for models (built); 7b4 quest bosses by quality tier, stronger and glowing in its colour, and named on hover (built) | 4 | independent |
| 8 | Quest dungeons per character: made by town and difficulty, Enter Cellar, hidden entrances, reset an hour after leaving, logging out inside, progress wiped on reset, locked secret doors and keys | 7a, 7b | |
| 9 | Escorts | 7a | |
| 10 | Chains, smaller-town feeders, standing tiers, hints and hidden requirements; alignment requirements (hidden entirely) and changes | 4 | independent |
| 11 | Dailies, the artifact line and set items (large final dungeon) | 8, 10 | |

New screens are markup (`client/dom.md`). Each PR updates its context files and decisions.md, and this table with what is built.
