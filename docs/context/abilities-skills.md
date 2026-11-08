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
| **Body** | Strength: force, damage, carrying | Agility: precision, balance, speed in a fight, quiet movement | Toughness: health, injury, poison, disease, endurance |
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
| Athletics | climbing, swimming, running, jumping, dodging, weight training |
| Stealth | sneaking, hiding, locks, traps |
| Survival | observation (noticing traps and the like, rather than a bare Perception check), tracking, hunting, fishing |
| Gathering | foraging, farming, mining, prospecting, herbalism |
| Crafting | building, smithing, leatherwork, tailoring, cooking, alchemy |
| Social | persuading, intimidating, trading, animals, leading |
| Lore | researching, creatures, history, languages, appraising |

Under melee, the third level is the type of weapon and the fourth the weapon itself: `combat/melee/longsword/longsword`, `combat/melee/shortsword/khopesh`. Fighting with no weapon is `combat/melee/unarmed/<kind>`, practising the current form's kind while shapeshifted (owner, 2026-11-06). Skills are learned by doing: a character knows a node once it has practised it (an axe in hand near a tree shows what can be done to it), and only known ones are shown. Defending is `combat/melee/defence/<weapon type>` (one level less deep: no single weapons), with `defence/unarmed` and the shields beside the weapon types (`combat/melee/defence/buckler`). Under melee because attack and defence overlap (the owner has trained with swords), so each benefits the other; shields are there so all defending practises Defence and a single passive defence ability can draw on it. Shields can be specialised, the weapon types under Defence can't. Ranged has no defence: dodging stays in athletics (owner, 2026-11-06). Attack draws on the node of the weapon in hand (or its type's, or `unarmed/<kind>`), and dual wielding is `combat/melee/dual`, beside the weapon types (built: `shared/combat.md`).

Later, not planned yet: investing in businesses, becoming a mayor.

- **Each node names the attributes it leans on**, and its children inherit them unless they say otherwise (melee: Strength and Agility; rapier: mostly Agility). Combinations are written once per branch, not per check.
- **Practice flows up, halving each level:** using a scimitar gives 1 to scimitar, ½ to 1-handed, ¼ to melee, ⅛ to combat. *To balance.*
- **A node's value is a root of its practice:** the cube root, so 100% takes 1,000,000 (built; owner, 2026-11-07: square roots, 100% at 10,000, grew far too fast), and every node has diminishing returns.
- **One practice per skill every 5 seconds** (built: `skill::Windows`), the largest in that time counting, so a quick weapon learns no faster than a slow one. At 8 a blow, nonstop fighting reaches 10% in about 10 minutes, 25% in under 3 hours, 50% in about 22 and 100% in about 174 (owner, 2026-11-07).
- **Familiar foes teach less** (built: `skill::taught`, `sheet::Foes`): characters count the 5-second blocks they spend fighting each kind of creature (a block counts for hitting one or being hit by one). Familiarity is the square root of those blocks, as a percentage; while it is no higher than the practised skill's own percentage, practice is in full, and past that it halves for every 10 points over, attacking and defending alike, so farming one kind stalls (about 20% after 1,000 kills of a 20-second fight). Kills are remembered too (`Foe::defeated`), for later: a mob remembers who harmed it and when, and a kill goes to everyone who harmed it in the last 2 minutes and is still on its level (so not if they died or left); the list is forgotten when it gives up fighting (at rest, settling or going home) (owner, 2026-11-07). Heal casts have no foe and are never reduced.
- **A check adds the values along the path.** Every level counts, like octaves of noise, so a lifelong brawler picking up a sword for the first time still beats someone who has never fought.
- **Learning speed** is practice × a root of the attribute against the kind's median. With square roots in both places, playing four times as long matches being twice as smart: first choices count, months of play catch up. *To balance.*

Worked example, 1,000 practice each, cube roots:

| Character | Scimitar | Another 1-handed weapon |
|---|---|---|
| All scimitar | 10.0 + 7.9 + 6.3 + 5.0 = 29.2 | 0 + 7.9 + 6.3 + 5.0 = 19.2 |
| 100 each with ten 1-handed weapons | 23.9 | 23.9 |
| All unarmed (under melee), picking up a sword | | 0 + 0 + 6.3 + 5.0 = 11.3 |
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
- Spells are abilities drawing on magic nodes. The first, Heal, is built (`shared/magic.md`): 6 Mana into Health by `magic/divine/healing`. Mana always refills; only Health waits for the end of a fight (owner, 2026-11-07).
- **Shapeshifting** (owner, 2026-11-06): a physical change takes the form's Body row and body, and keeps the creature's own Mind and Spirit (built as `Sheet::shapeshift`, for now only from the debug Change Creature). Senses that come with a kind (a wolf's nose, an owl's eyes) are to be abilities linked to the kind, and so come with a form: decided when abilities are added. A disguise is an illusion: looks only, every rule keeping the real kind.

## Combat

The owner's design (2026-11-06/07), the first active ability being Attack. Built (Combat: the Attack ability, 4 PRs): the rules in `shared::combat` (`shared/combat.md`), holding weapons, Attack on the server for players and mobs (`server/server.md`), and showing fights (client `abilities`, `entities`, `vitals`). Not built yet: Parry and Block, interruption by being hit, armour figures (armour counts as 0) and armour piercing, mana refunds on fizzling (no spells yet), saving mobs' Health.

- **Energy alone:** abilities cost energy (`shared::energy`), with no cooldowns until an ability needs one; mana limits magic.
- **Wind-up and recovery:** an ability's cost is split: the wind-up is waited before its effect lands, the recovery after. Steps stay act-first. The server says exactly when (`lands_at`, `ready_at`), so the client shows progress (a sweep over the action bar slot, a bar over the token for long ones), and others' wind-ups are sent so players can react.
- **Cancelling:** anything with a wind-up can be cancelled until it lands, never after, leaving only a small wait. **Fizzling:** when it lands the ability checks its target is still valid (in reach, in sight); if not it fizzles, the energy spent and mana partly refunded (mana is charged on landing). **Interruption** by being hit is to be an opposed check, not automatic.
- **Attacks:** about one a second with a sword, two with bare hands, one every two seconds with a polearm, by handling weight (length counts), quickened by agility, a little cheaper with skill. The body row is melee's three sides (owner): Toughness for Health, Agility for speed, Strength for harm. Polearms and spears reach two cells.
- **Harm:** no hit or miss roll. A blow (weapon, strength, luck; quality counts most, then metal, then weight), then the attacker's power against the defender's passive Defence as a share (equals take half), then armour's share (never nothing). Heavier weapons are to pierce armour, their mass lowering the armour figure they are checked against (owner, not built). Health and Mana are kept in thousandths. Two equal median creatures fight for well under half a minute.
- **Defence:** passive and free, always at work; Parry (and Block with a shield) are to be active abilities opening a short window.
- **Dual wielding** (`combat/melee/dual`, beside the weapon types, specialisable): hits take turns, the bigger weapon always the main hand; the off hand does half plus half the skill's percentage, and the skill takes up to a fifth off each attack's cost, so a master is about a quarter quicker. Untrained, it is worse than one weapon.
- **Automatic abilities** fire whenever they can (Attack: a target, in reach, an enemy), so they can be left on; not every ability may be automatic. Being attacked with no target in sight makes the attacker the target.
- **Death,** for now: a player at 0 Health is put out of the poi (or the wilderness) onto the world map, told "You have been killed, try again!", at full Health with everything they carried.

## Specialising

- A deliberate choice, never automatic. Any level except the top, one per group of siblings (a sword under melee and a bow under ranged can both be specialised), as many as can be earned. Permanent.
- **False cap:** every node is capped at 100%. A specialised one may reach 120%, and each of its siblings is then capped at 100% minus the overshoot (101% leaves them at 99%). The cap limits the value used; practice underneath is kept.
- **Earning one:** reach about 80% in the node, find the way in for that kind of skill (a trainer for a weapon, a book for a spell, a special item for archery), choose to try, which gives a quest, and finish it (usually including reaching 100%). Some skills simply offer the quest, which can be refused, on reaching 100%. Needs quests, trainers and books, so it comes last.
- Unused, the part above 100% falls about a point per 24 hours of playtime (through the rust clock below), and siblings' caps rise with it.

## Rust and opposition

Built (2026-11-08) for disuse under Melee, opposition and dying: `shared/skill.md`. Rate and floor are one setting for now (`Rust`); siblings in use slowing rust or raising the floor, and specialisations falling, are not built.


- **Opposition links** go one way, between any two nodes (never a node and its own ancestor or descendant), and hit only the node they name, not its children (owner, 2026-11-08): heavy armour wears down sneaking, not the other way round; a mirror link is added only if wanted. Melee and ranged oppose each other; combat opposes magic one way.
- **Rust** takes a node back towards a floor (around half the best it reached), and relearning up to that best is faster than learning was (twice as fast). Rate and floor are set on a node and passed down to everything under it unless it says otherwise; siblings in use slow it or raise the floor. Disuse: weapon skills (all of Melee) after 8 hours unused, then a step an hour, each 1% of what is above the floor. *To balance.*
- **Dying** costs every skill the steps 8 hours (Normal) or 24 hours (Hard) of disuse would take, whatever its clock: mild, a "doh" rather than a ragequit (owner, 2026-11-08).
- **The rust clock:** each node keeps the playtime before which it can't rust again. Opposition (about an hour), plain disuse (per node) and an unused specialisation (about 24 hours) each take one step only when the clock has passed, then move it on. Several links don't stack, and loss is always slower than gain.
- **Practising a node pushes its own clock forward**, so using both sides of a link keeps both: opposition only wears down what has been set aside.
- Playtime is counted per character in ticks and saved with it, so time offline never counts, and only up to 5 minutes after the player last did something (built: server `connections/playtime.rs`).
- Separate from rust: worn gear may also make a check harder at the time (clanking armour while sneaking). That is a rule about the gear.

## Movement skills (owner, 2026-11-08)

Speeds are ratios to an average walker (85 m a minute), with 100% at world-record level; each effect grows in a straight line with the node's own value. Practice comes from moving, at most once per skill every 5 seconds. Built: Climbing and Running on the world map, Climbing in the wilderness, Weight Training, and Swimming by wading shallows and rivers, and from 50% swimming deep water and the sea near the coast, and Jumping (`shared/movement.md`). Planned, one PR each, in order:

- Climbing: the slope's cost down to a tenth at 100%; only steps changing height by more than 25 m teach it. In the wilderness (built), slopes from the world points' heights, blended in 16ths, cost and teach as the world map does for the same height.
- Running: world map only, walking at 0% to a marathon record's pace (about 350 m a minute, 4.1×) at 100%.
- Weight Training (a new Athletics leaf, Strength and Toughness): capacity × (1 + value), so 100% doubles it; practised per distance while carrying more than half of it.
- Swimming: wading shallows and rivers costs 1.7× walking at 0% (50 m a minute), 0.71× at 100% (a record's 100 m in 50 s); every wilderness cell teaches it, world map river points a little. Then deep water by skill: wilderness and site water, shallow coastal sea on the world map, never deep ocean.
- Jumping (built): practised per jump, slowly (about 50% after 3,000), making jumps quicker, to 70% of their distance at 100%; later perhaps jumping up to the edge of a hole above (a grappling hook would rival it).
- Sneaking: practised only in the wilderness, while averaging under 1 cell every 2 seconds over a minute (to tweak), and seen by something territorial or timid that isn't hostile. A Sneak toggle ability holds the pace under that; the benefit (harder to spot) comes with mob senses.
- Pioneering (under Survival), a child per terrain that slows travel: grass and scrub 1.2×, forest 1.3×, jungle 1.5×, marsh and snow 1.8×, sand 2×; 100% takes off two thirds of the extra.
- Riding is likely before long; navigation waits for sailing.

## Building it (later tasks, in order)

1. Attributes: the nine, kinds' medians, ranges and bonuses, mobs rolled the same way, and the triangles in a character creation screen (built, `client/character_create.md`; one character per account for now, with Character Selection later before it).
2. The tree, practice, checks and rust clock (the tree's nodes, practice flowing up and from use, values and rust are built, and shown on the Skills page, `client/skills_page.md`; checks are not), with a tool printing the curves (a character after so many hours of a way of playing) and those curves on a players' page.
3. Abilities and opposition (begun: `shared::ability`, active and passive, the action bar, and Attack, which draws on the node of the weapon held and practises it with every blow; see Combat for what is still to come; opposition is built with rust).
4. Specialising, once quests, trainers and books exist.
