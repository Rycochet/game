# Story text (plan)

The plan for quests' flavour text, in 4 PRs, planned with the owner on 2026-11-12. All four PRs are built: `shared: lang`, `shared: story`'s names, facts, plots and beats, and the English wordings. **Owner** marks the owner's decisions.

## Goal

A quest's structure ("find item:ring at poi") becomes up to about three paragraphs of flavour text: a grandmother's ring lost when she went to a cave west of here, wolves chased the family off, and the giver is getting married. Deterministic, open to other languages' grammar, with generated names, and the same names and facts across a questline's steps. Quests themselves: the Quest system plan (`quests.md`).

## How it works

- **Facts are fixed, wording is per language.** The server decides the plot, the cast (names, gender, relations), the facts and the beats, and sends them as data, a `Story`, with a seed made only for text (never a poi seed or anything secret). **The client writes the text in its language (owner).** The rendering code is shared so tests can print stories; the server never renders.
- **Beats, not sentences.** Each beat ("the lost person vanished heading to the place", "a first attempt failed because of creatures", "the stake: a wedding") has several wordings per language. The wording is `hash(text seed, beat) % wordings in this language`, so languages needn't have the same number. Adding wordings may reword existing quests, as changing place name lists does.
- **Facts are values, never words:** relation Grandmother, trait Curious, direction West. Each language has a phrase per value (`Lang::phrase`), so a fact said twice stays the same.
- **Proper names are never translated; words always are.** People's names are sounds; epithets, titles and descriptions of places go through the language files. Town names' English parts ("Little", "Castle") wait for a later language pass.
- **Fluent** (`fluent-bundle`) for the language files (**owner**: fine for now, can revisit), with per-language Rust rules for what it can't say. See `shared/lang.md`.
- **Gender** is `shared::gender::Gender` (Male, Female, Neutral). Townsfolk roll it from their seed like mobs. Neutral is each language's third pronoun form ("they").
- **One cast per questline:** a later step or chained quest may take a role from an earlier quest ("quest X, role grandmother") instead of rolling a new one. The server can rebuild an earlier cast from that quest's seed; the client is sent the cast resolved.
- **No live generation by a model.** Wordings are written offline in bulk (`research-tools/llm.py`, Ollama as fallback), reviewed by the owner, and the game only picks from fixed lists.
- **Alignment stays hidden.** The server alone decides which beats a story has (dark beats need a bad reputation); nothing the client renders depends on alignment.

## Facts, roles and beats

From the research catalogue (`docs/context/research-story-plots.md` with PR 3), owner-reviewed:

- **Facts** (each an enum implementing `lang::Selector`): Relation, Trait, State, Errand, TimeAgo, Direction (8 ways or Here), DistanceBand (Near, Short, Day, Far), PlaceKind (from `PoiKind`), Creatures (kind and count band), ItemCategory, Occasion, Stake, Reward, Season and Weather, and a Truth per stated fact (True or Lie, and who lied). Checkable facts (direction, distance, place, creatures, item, reward, season) are worked out from the world and must be true; colour facts are rolled from the text seed.
- **Roles:** Giver, Relative, Lost, Pet, Rival, Recipient, Escortee, Witness, Donor, Boss, Creatures, Item, Place, Home.
- **Reasons** (why a giver asks): Rumour, Family, Neighbour feud, Lost pet, Debt or trade, Threat to the town, Grief, and **Learning (owner)**, each suited to some quest goals and place kinds.
- **Beats:** hook and backstory (paragraph 1), complications (2), the ask, always present, and the stake (3); plus dark beats, lies, twists and quandaries. States only move forward.

## Owner's decisions on the research (2026-11-12)

1. Learning is an eighth reason; training a skill can be a quest's direct reward.
2. Dark beats (murder, revenge, theft, framing, silencing) only for characters with a bad reputation; a quest with an alignment requirement is hidden entirely from others (no grey "!"). Alignment is `shared::alignment`, never sent to clients.
3. Twists and moral quandaries only in later chain steps, and they may move alignment.
4. The giver mostly tells what is known before the quest is accepted, but stories may lie: a Truth per fact, fixed at generation, found out at the place, at turn-in or in a later step.
5. Pets are existing creature kinds for now (Horse, Chicken, Cattle, Wolf as a dog); more as creatures are added.
6. The text names the real reward, unless a twist or lie changes it.
7. Season and weather beats that can be checked only in higher-reward quests.

## PRs

| PR | What | Needs | State |
|---|---|---|---|
| 1 | `shared::lang`: `Language`, Fluent bundles built in, English rules, rendering with names, counts and selectors; tests that every message parses and renders with its values. This note. | - | built |
| 2 | Names: family names per household, given names per people (`creature::names`), pet names, named bosses with a translated epithet, descriptions of unnamed places, direction and distance bands from a town, as facts. | Gender (merged) | built |
| 3 | `shared::story`: plots and beats from the catalogue, `Cast` with roles carried over, facts, seeded choices, beats into up to 3 paragraphs, a serialisable `Story` for the protocol; sample stories printed by a test. The Quest system's PR 2 needs this and uses it for givers' names and text. | 1, 2 | built |
| 4 | English wordings: drafted in bulk, reviewed by the owner, then the `.ftl` files and golden tests. Replaces the placeholders in `lang/en/story-plots.ftl`, keeping each beat to the values it is sure to have (`shared/story.md`). | 3 | built |
