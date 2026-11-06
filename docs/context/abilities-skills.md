# Attributes, skills and abilities (plan; attributes and the tree's data built)

The owner's design for what a creature is like and what it is good at, the same for players and mobs: a player is steered by a person, a mob by rules, and both have the same sheet. Worked out in conversation on 2026-11-05. Numbers marked *to balance* are first guesses, to be settled with a curves tool (below) rather than here. Why: see `decisions.md`.

## Words

- **Attributes:** what a body, mind and spirit are (in code: `sheet::Attributes`, the nine below).
- **Skills:** what has been learned, as a tree (the nodes and each creature's practice are built: `shared/skill.md`).
- **Abilities:** named actions that draw on skills and attributes (today: `shared::ability`: Attack, which everyone knows, and things a kind does by nature such as Kick; active ones go on the action bar, passive ones such as Night Vision just work). Things that aren't actions, such as NightVision, move to a separate list of senses or traits.

## Attributes

Nine, in a grid. Each row is one side of a creature, each column one way of being strong:

| | Power | Finesse | Resistance |
|---|---|---|---|
| **Body** | Strength: force, damage, carrying | Agility: precision, balance, quiet movement | Toughness: health, injury, poison, disease, endurance |
| **Mind** | Intellect: reasoning, learning, lore, arcane magic | Perception: noticing, reactions, aim | Will: resisting things done to the mind (fear, confusion, sleep), concentration |
| **Spirit** | Presence: commanding, persuading, intimidating | Empathy: reading people and creatures, territory, divine magic | Aura: resisting things done to the self (magic, curses, possession) |

Built (2026-11-06), summarised in `docs/context/shared/creature.md`. The old four (strength, dexterity, intelligence, constitution) became Strength, Agility, Intellect and Toughness; medians and bonuses in `creature/guesses.rs`, `peoples.rs` and `bodies.rs` are still first guesses.

- **Scale:** integers, with a kind's median at 100, leaving room for small differences and slow growth (`u8` won't do). *To balance.*
- **Each kind has a median and a range** for every attribute. Mobs are rolled within it from the seeded RNG; NPCs get more spread, as they live longer and should be more interesting. The range is a share of the median either side (`SPREAD`), the same for every kind so far.
- **Kind bonuses** (owner, 2026-11-06): a kind may add a bonus to some, all or none of the nine, on top of what the triangles give, for players and mobs alike. The peoples share a person's medians and differ only by these. Every kind has the same points in all (`BONUS_POINTS`): what it sets comes first, and the rest is shared evenly among the attributes it doesn't set. Placeholder values for now; the real ones come later.
- **Starting values** come from the triangles (below). **Growth** comes from use: each use of a skill gives a little practice to the attributes its node leans on, far slower than the skill itself, and slower still the further above the starting value. **Temporary changes** (injury, fatigue, hunger, potions, curses, blessings) sit on top and are never saved into the base. **Permanent changes** are rare and earned (rituals, artifacts, perhaps age).

### Choosing them: three triangles

- Each row is a triangle with its attributes at the corners. A dot inside it splits the row between its three (barycentric weights). A corner gives that attribute the kind's maximum and the other two the minimum; the centre gives an even share.
- The three triangles sit round a shared centre and turn together. There are 4 points: each row gets 1, plus a share of the last 1 by how high it sits (rows below the middle get none). Body at the top gives 2, 1, 1; two rows equally high give 1.5, 1.5, 1. At most two rows share the extra point, so every character leans somewhere. How a row's points widen its range is *to balance*.
- The triangles stay one size. Behind each is a radar chart of its three attributes, so turning them shows at once what grows and shrinks. The chart has two layers: the centremost, in its own colour, is the kind's bonus, and outside it what the triangle adds. The bonus layer is the hull of three small circles of one radius, one on each axis at its bonus (a bonus of 0 at the chart's centre), so it is a round-ended triangle that never vanishes, and only real bonuses push a corner out (owner, 2026-11-06).
- The client sends the dots and the turn as integers; the server checks them and works out the attributes. Mouse placement means two players copying a build will still differ a little, which is welcome.
- Mobs use the same four values, set near a role's target and jittered by the RNG; a kind may fix the turn.
- Works by mouse, touch (drag), keys (nudge) and later gamepad.

## Skills

A strict tree, 3–4 levels for most branches; weapons one level more than magic, as attacks happen far more often than casting. Draft of the top levels (the built draft, with deeper levels, is in `shared::skill`'s `branches/`):

| Branch | Children (draft) |
|---|---|
| Combat | melee (unarmed by kind of creature, then each type of weapon with each weapon under it, and defence: each weapon type, unarmed and the shields), ranged (thrown, bows, crossbows, slings), anticipation (reading opponents), armour (light, heavy) |
| Magic | arcane (leans on Intellect), divine (Empathy), wild |
| Athletics | climbing, swimming, running, jumping, dodging |
| Stealth | sneaking, hiding, locks, traps |
| Survival | observation (noticing traps and the like, rather than a bare Perception check), tracking, hunting, fishing |
| Gathering | foraging, farming, mining, prospecting, herbalism |
| Crafting | building, smithing, leatherwork, tailoring, cooking, alchemy |
| Social | persuading, intimidating, trading, animals, leading |
| Lore | researching, creatures, history, languages, appraising |

Under melee, the third level is the type of weapon and the fourth the weapon itself: `combat/melee/longsword/longsword`, `combat/melee/shortsword/khopesh`. Fighting with no weapon is `combat/melee/unarmed/<kind>`, practising the current form's kind while shapeshifted (owner, 2026-11-06). Skills are learned by doing: a character knows a node once it has practised it (an axe in hand near a tree shows what can be done to it), and only known ones are shown. Defending is `combat/melee/defence/<weapon type>` (one level less deep: no single weapons), with `defence/unarmed` and the shields beside the weapon types (`combat/melee/defence/buckler`). Under melee because attack and defence overlap (the owner has trained with swords), so each benefits the other; shields are there so all defending practises Defence and a single passive defence ability can draw on it. Shields can be specialised, the weapon types under Defence can't. Ranged has no defence: dodging stays in athletics (owner, 2026-11-06). The Attack ability will hook onto the node of the weapon in hand, after the tree exists.

Later, not planned yet: investing in businesses, becoming a mayor.

- **Each node names the attributes it leans on**, and its children inherit them unless they say otherwise (melee: Strength and Agility; rapier: mostly Agility). Combinations are written once per branch, not per check.
- **Practice flows up, halving each level:** using a scimitar gives 1 to scimitar, ½ to 1-handed, ¼ to melee, ⅛ to combat. *To balance.*
- **A node's value is a root of its practice** (square root as a start), so every node has diminishing returns.
- **A check adds the values along the path.** Every level counts, like octaves of noise, so a lifelong brawler picking up a sword for the first time still beats someone who has never fought.
- **Learning speed** is practice × a root of the attribute against the kind's median. With square roots in both places, playing four times as long matches being twice as smart: first choices count, months of play catch up. *To balance.*

Worked example, 1,000 uses each, square roots:

| Character | Scimitar | Another 1-handed weapon |
|---|---|---|
| All scimitar | 31.6 + 22.4 + 15.8 + 11.2 = 81 | 0 + 22.4 + 15.8 + 11.2 = 49 |
| 100 each with ten 1-handed weapons | 59 | 59 |
| All unarmed (under melee), picking up a sword | | 0 + 0 + 22.4 + 15.8 = 38 |
| Never fought | | 0, plus a small floor |

## Checks

- **Power** = attribute factor × skill value: doubling either doubles power, so twice as experienced is about as good as twice as able.
- **Ratio contest:** success when a roll from 0 to a + b − 1 is under a, so the chance is a ÷ (a + b). Integer and deterministic. It is never impossible: ten times weaker still gives about 9%, so there are lucky shots and no places a character can't risk going.
- Health and damage should grow modestly over a career, so a lucky hit still matters. Health and Mana are built (`shared/vitals.md`): from Toughness and Will, a point a day of playtime and practice from harm taken and mana spent; their practice may move into the tree when it exists.
- An untrained node counts from its parents and never below a small floor.

## Abilities

An ability draws on several nodes with weights, and on attributes: Throw Spear is half melee/polearm/spear, half ranged/thrown. Using it gives practice to each node in the same shares. Abilities have no practice of their own. This keeps the tree strict while actions cross branches.

- Dodge: athletics/dodging with combat/anticipation against a blade, or with survival/observation against a trap.
- Some come with a kind (Kick), others are learned.
- Spells are abilities drawing on magic nodes.
- **Shapeshifting** (owner, 2026-11-06): a physical change takes the form's Body row and body, and keeps the creature's own Mind and Spirit (built as `Sheet::shapeshift`, for now only from the debug Change Creature). Senses that come with a kind (a wolf's nose, an owl's eyes) are to be abilities linked to the kind, and so come with a form: decided when abilities are added. A disguise is an illusion: looks only, every rule keeping the real kind.

## Specialising

- A deliberate choice, never automatic. Any level except the top, one per group of siblings (a sword under melee and a bow under ranged can both be specialised), as many as can be earned. Permanent.
- **False cap:** every node is capped at 100%. A specialised one may reach 120%, and each of its siblings is then capped at 100% minus the overshoot (101% leaves them at 99%). The cap limits the value used; practice underneath is kept.
- **Earning one:** reach about 80% in the node, find the way in for that kind of skill (a trainer for a weapon, a book for a spell, a special item for archery), choose to try, which gives a quest, and finish it (usually including reaching 100%). Some skills simply offer the quest, which can be refused, on reaching 100%. Needs quests, trainers and books, so it comes last.
- Unused, the part above 100% falls about a point per 24 hours of playtime (through the rust clock below), and siblings' caps rise with it.

## Rust and opposition

- **Opposition links** go one way, between any two nodes: heavy armour wears down sneaking, not the other way round; a mirror link is added only if wanted. Melee and ranged oppose each other; combat opposes magic one way.
- **Rust** takes a node back towards a floor (around half the best it reached), and relearning up to that best is faster than learning was. Rate and floor are set per node; siblings in use slow it or raise the floor.
- **The rust clock:** each node keeps the playtime before which it can't rust again. Opposition (about an hour), plain disuse (per node) and an unused specialisation (about 24 hours) each take one step only when the clock has passed, then move it on. Several links don't stack, and loss is always slower than gain.
- **Practising a node pushes its own clock forward**, so using both sides of a link keeps both: opposition only wears down what has been set aside.
- Playtime is counted per character in ticks and saved with it, so time offline never counts, and only up to 5 minutes after the player last did something (built: server `connections/playtime.rs`).
- Separate from rust: worn gear may also make a check harder at the time (clanking armour while sneaking). That is a rule about the gear.

## Building it (later tasks, in order)

1. Attributes: the nine, kinds' medians, ranges and bonuses, mobs rolled the same way, and the triangles in a character creation screen (built, `client/character_create.md`; one character per account for now, with Character Selection later before it).
2. The tree, practice, checks and rust clock (the tree's nodes, practice flowing up and values are built, and shown on the Skills page, `client/skills_page.md`; practice from use, checks and rust are not), with a tool printing the curves (a character after so many hours of a way of playing) and those curves on a players' page.
3. Abilities and opposition (begun: `shared::ability`, active and passive, the action bar; abilities don't hook onto skills yet: Attack will use the node of the weapon held, `combat/melee/<type>/<weapon>`, once combat comes).
4. Specialising, once quests, trainers and books exist.
