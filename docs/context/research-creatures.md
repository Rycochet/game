# Research: creatures

What the creature kinds, their bodies and their models will be built from, researched and reviewed by the owner on 2026-11-04. Not kept up to date with the code: the code wins. The review page, with every kind's card, checked quotes and SRD data, is https://claude.ai/artifact/Vmfhg3XApniUsDKzaGf1MH; the scripts and raw data are in the owner's research tools folder.

## Method

Monsters from RogueBasin's list, NetHack 3.6 (`src/monst.c`), Angband (`monster.txt`), Dungeon Crawl Stone Soup (`dat/mons`), Brogue CE (`monsterCatalog`) and the D&D System Reference Document 5.1 (588 monsters, CC-BY 4.0, through the MIT 5e-bits database), grouped by head noun into base kinds. Wikipedia's lists of legendary creatures (about 1,900) were ranked by Google Books Ngram and picked by hand. Generic livestock and common wild animals were added, never breeds. One Wikipedia article per kind was read by two local models (llama3.1 8B for all, qwen3 8B again for real animals) with quotes checked word for word; the measurements were then checked against the estimates, which were corrected where the quotes showed them wrong (bison, elk, cave bear, gnome). Senses and behaviour are judged by hand, as the models' choices were guesses. Descriptions are written fresh.

## Owner's decisions

- **Names:** nothing trademarked or owned by one game: "halfling" yes, "hobbit" no. D&D creatures in the open SRD stay under their names (marked SRD below), with the CC-BY 4.0 attribution in `docs/credits.md` once one is in the game. The rest are left out with a stand-in (below).
- **Lists may be long:** ranked, not cut. Livestock and wild animals are generic kinds; "giant X" and "dire X" are size variants of the base animal.
- **Undead and constructs** are variants of almost any kind, by its skeleton (mobs only). Lich, wraith, golem and the like have rows of their own. Slimes and oozes are in.
- **Folk and religious figures stay.** Gods and religion are a later task, which may use variations of this list.
- **Story only:** creatures too big for any map (below) are kept apart, not dropped.
- **Sizes:** cells are about 1.5 m. Size bands go by the largest dimension: Tiny under 0.75 m, Small 0.75-1.5, Medium 1.5-2.1 (humans), Large 2.1-3.2, Huge 3.2-4.5, Giant 4.5-7, then specials by their own footprint. The `Size` enum will change so that clothing sizes follow these bands.
- **Cells:** Tiny shares a cell; up to Large takes 1x1; Huge and Giant 2x2; specials their footprint. Long bodies (over about 2 m long, not upright: horse, centaur, cattle, big cats) are marked 1x2 for the owner to think over, as moving them is still to be worked out.
- **Players** may only pick kinds of 1x1 or smaller, so the centaur and giant stop being playable.
- **Armour fits by body:** snouts and horns refuse closed helmets, hooves and digitigrade feet refuse boots, wing-arms refuse sleeves and arm armour, back wings and shells need torso pieces made for them, a tail goes under faulds.
- **Shapeshifting:** when a shapeshifter changes to a form whose body doesn't take something it is wearing, that item drops to the ground; what still fits stays on.
- **Morphing (owner's question, answered yes):** the frame becomes a general body layout (a spine that tilts, limb slots with roles, head, tail, wings, parts as numbers), so a later task can blend one form into another: arms become forelegs, legs hind legs, the spine levels out, and parts with no partner grow from or shrink to a point.
- **Models:** the human alone first, worked out from proportions, for the owner to tweak its stance; then halfling, dwarf, elf, orc, goblin, kobold, lizardfolk, minotaur and birdfolk; other body plans after that.

## Reading the tables

Heights and lengths in metres, weights in kg: estimates for the game, narrowed where real ranges are wide (humans 1.55-1.90 m), checked against the quotes on the review page. Size: T, S, M, L, H, G, or G+ with a footprint. Sight none, poor, normal or keen; dark is how well it sees without light (none, low, dark, heat); hearing and smell none to keen. Games is how many of the six lists have the kind.

## People

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| human (person, man, commoner) | Adaptable, short-lived folk found everywhere, from farms to cities. | biped | M | 1×1 | 1.55-1.90 |  | 50-100 | skin, hair | skin: pale, fair, olive, tan, brown, dark brown; hair: black, brown, auburn, red, blond, grey |
| halfling | Small, sturdy country folk who love comfort, food and quiet, and move without a sound. | biped | S | 1×1 | 0.90-1.20 |  | 18-35 | skin, curly hair, hairy feet | skin as human; hair brown, black, auburn, sandy |
| dwarf (duergar) | Short, broad people of the mountains: miners, smiths and stubborn keepers of grudges. | biped | S | 1×1 | 1.20-1.45 |  | 60-90 | skin, thick hair and beards | skin: ruddy, tan, brown, grey-brown; hair: black, brown, red, grey, white |
| elf | Tall, slender, long-lived people of woods and old places, with pointed ears and keen senses. | biped | M | 1×1 | 1.55-1.85 |  | 45-70 | skin, fine hair | skin: pale, fair, golden, tan, bark brown; hair: blond, silver, black, copper, auburn |
| dark elf (drow, svartalf) | Elves of the deep places, pale-haired and dark-skinned, who shun the sun and scheme in the dark. | biped | M | 1×1 | 1.50-1.80 |  | 40-65 | skin, fine hair | skin: ash grey, slate, dusky blue, black; hair: white, silver, pale grey |
| half-elf | Children of elves and humans, at home in both worlds and fully trusted by neither. | biped | M | 1×1 | 1.55-1.85 |  | 50-85 | skin, hair | as human and elf |
| half-orc | Children of orcs and humans: strong, tusked, and judged by their looks. | biped | M | 1×1 | 1.65-2.00 |  | 70-120 | skin, coarse hair | skin: grey-green, olive, ash, tan; hair black, dark brown |
| gnome | Small, big-nosed tinkerers and burrowers, curious and fond of jokes and gems. | biped | S | 1×1 | 0.90-1.15 |  | 18-30 | skin, hair, beards | skin: ruddy, tan, wood brown; hair: white, grey, brown, red |
| goblin | Small, sharp-eared, spiteful raiders that swarm from caves and ruins in gangs. | biped | S | 1×1 | 0.90-1.20 |  | 18-30 | skin, sparse hair | skin: green, yellow-green, grey, ochre; hair black |
| hobgoblin | Larger, disciplined goblin kin who march in ranks and keep strict order. | biped | M | 1×1 | 1.60-1.90 |  | 70-100 | skin, coarse hair | skin: red-orange, dark red, grey-brown; hair dark |
| orc (orcish) | Tusked, heavy-set warriors who live by raiding and hate bright light. | biped | M | 1×1 | 1.65-2.00 |  | 70-130 | skin, coarse hair | skin: grey-green, olive, grey, dark brown; hair black |
| bugbear | Big, hairy goblin kin that move with surprising stealth and strike from ambush. | biped | M | 1×1 | 1.80-2.10 |  | 110-150 | coarse fur | fur: yellow-brown, brick red, brown |
| kobold | Small, scaly trap-makers who dig warrens and mines and outnumber whatever they fight. | biped-dg | S | 1×1 | 0.80-1.10 |  | 18-30 | scales | red-brown, rust, black, ochre, grey |
| gnoll | Tall hyena-headed raiders that roam in packs and eat what they kill. | biped-dg | L | 1×1 | 2.00-2.40 |  | 110-160 | fur, mane | fur: tan, sandy, grey with brown spots |
| ogre | Huge, fat, dim brutes that eat people and take what they want with a club. | biped | L | 1×1 | 2.40-3.00 |  | 250-450 | skin, coarse hair | skin: dull yellow, brown, grey, grey-green, warty |
| troll | Long-armed, warty monsters of bridges, hills and forests that heal fast and fear the sun. | biped | L | 1×1 | 2.40-3.20 |  | 250-500 | warty hide, moss-like hair | grey, green, mossy green, stone grey, brown |
| lizardfolk (lizardman) | Scaly, tailed hunters of swamps and rivers, practical and hard to read. | biped-dg | M | 1×1 | 1.70-2.10 | tail 1.0 | 90-130 | scales | green, grey-green, brown, olive, banded |
| ratfolk (ratman) | Rat-headed people of sewers and undercities, quick, wary and many. | biped-dg | S | 1×1 | 1.10-1.40 | tail 0.8 | 25-45 | fur | brown, grey, black, white, piebald |
| catfolk | Cat-headed people, graceful and curious, who roam far from home. | biped-dg | M | 1×1 | 1.50-1.90 | tail 0.9 | 45-80 | fur | tawny, ginger, black, grey, spotted, striped |
| dogfolk (cynocephalus) | Dog-headed people of medieval travel tales, loyal to their packs. | biped-dg | M | 1×1 | 1.50-1.90 |  | 50-90 | fur | brown, black, tan, grey, white, patched |
| birdfolk | Winged people whose arms are wings, nesting on cliffs and peaks. | biped-winged | M | 1×1 | 1.40-1.70 | wingspan 4.5 | 25-45 | feathers | brown, grey, black, white, red, blue, barred |
| frogfolk | Squat, wide-mouthed people of bogs and fens who sing at night. | biped-dg | S | 1×1 | 1.10-1.50 |  | 30-55 | smooth wet skin | green, olive, brown, mottled, bright warning colours |
| troglodyte | Grey, foul-smelling cave-dwellers that hunt in the dark below. | biped | M | 1×1 | 1.50-1.80 |  | 70-100 | scaly hide | grey, grey-brown, ochre |
| merfolk (merman, mermaid, merrow) | People with a fish's tail instead of legs, living in the sea. | serpent-torso | M | 1×1 | 1.6-2.0 long |  | 70-120 | skin above, scales below | skin as human or sea green, blue; tail silver, blue, green; hair green, blue, black, gold |
| centaur | Horse-bodied people with a human upper body, proud and wild. | taur | L | 1×2 | 2.10-2.40 | 2.2 | 450-650 | skin above, horse coat below | skin as human; coat bay, chestnut, black, grey, dun |
| satyr (faun) | Goat-legged, horned revellers of the woods who love wine and music. | biped-dg | M | 1×1 | 1.40-1.70 |  | 45-70 | skin, goat legs and fur | skin tan, ruddy; fur brown, black, grey |
| minotaur | Bull-headed giants of labyrinths who never lose their way. | biped-dg | L | 1×1 | 2.20-2.70 |  | 300-450 | fur on head and shoulders, skin | brown, black, red-brown |
| harpy | Bird-winged women with taloned feet who snatch food and people. | biped-winged | M | 1×1 | 1.40-1.70 | wingspan 3.5 | 30-50 | skin, feathers | skin pale, grey; feathers grey, brown, black |
| naga | Serpents with human heads and upper bodies, guardians of treasure and holy places. | serpent-torso | L | 1×1 | head to tail 4-6 |  | 150-300 | scales, skin | green, gold, blue, black, banded |
| lamia | A snake-tailed woman who preys on children and travellers. | serpent-torso | L | 1×1 | head to tail 3-4 |  | 100-200 | skin, scales | skin as human; scales green, gold, mottled |
| tengu | Bird-like mountain spirits of Japan, fierce swordsmen and tricksters. | biped-winged | M | 1×1 | 1.40-1.80 | wingspan 3.5 | 40-70 | feathers, skin | red skin, long nose; or crow black feathers |
| oni | Horned, tusked ogres of Japan with red or blue skin and iron clubs. | biped | L | 1×1 | 2.20-3.00 |  | 200-400 | skin, wild hair | red, blue, green, black, yellow skin |
| yeti (abominable snowman) | A huge shaggy ape-man of the high snows. | biped | L | 1×1 | 2.20-3.00 |  | 200-400 | thick fur | white, grey, reddish brown |
| sasquatch (bigfoot) | A tall, hairy forest giant that leaves footprints and is seldom seen. | biped | L | 1×1 | 2.10-3.00 |  | 250-400 | thick fur | brown, black, reddish brown |
| neanderthal (caveman) | Stocky, heavy-browed people of the ice age, close cousins of humans. | biped | M | 1×1 | 1.50-1.70 |  | 60-85 | skin, hair | pale to brown skin; red, brown, black hair |
| scorpion man (girtablilu) | A guardian with a man's upper body on a scorpion's. | taur | L | 1×2 | 2.2-2.6 | 2.5 | 250-400 | skin, chitin | bronze skin, black, red-brown |
| gorgon (snake-haired) (medusa) | Women with snakes for hair whose gaze turns the living to stone. | biped | M | 1×1 | 1.60-1.90 |  | 50-80 | skin, snakes for hair; scales below in some | skin pale green, bronze; snakes green, black |
| grimlock (SRD) | Blind grey cave people who hunt by sound and smell. | biped | M | 1×1 | 1.5-1.8 |  | 60-80 | grey skin | grey, white |
| sahuagin (SRD) | Fierce, shark-loving fish-people of the sea. | biped | M | 1×1 | 1.8-2.0 |  | 80-130 | scales, fins | green, black, grey |
| wild man (woodwose, almas) | Hairy men of the woods from medieval art, half human and half beast. | biped | M | 1×1 | 1.60-2.00 |  | 60-100 | shaggy hair all over | brown, black, grey hair |
| garuda | Eagle-headed, winged warrior people and enemies of serpents. | biped-winged | M | 1×1 | 1.70-2.20 | wingspan 5 | 60-100 | feathers, skin | gold, red, white feathers |
| medusa | The one mortal gorgon, whose severed head still turns people to stone. | biped | M | 1×1 | 1.60-1.90 |  | 50-80 | as gorgon | as gorgon |
| baba yaga | A terrible old witch of the Slavic woods who lives in a hut on chicken legs. | biped | M | 1×1 | 1.40-1.70 |  | 40-60 | skin, hair | pale, wrinkled; grey hair |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| human | varies | day | family | normal | none | normal | poor |  | walk, swim, climb | speaks | common | 4 |
| halfling | friendly | day | family | normal | none | keen | normal |  | walk, swim, climb | speaks | common | 4 |
| dwarf | neutral | any | family | normal | dark | normal | normal | feel stone | walk | speaks | common | 5 |
| elf | neutral | any | family | keen | low | keen | normal |  | walk, climb, swim | speaks | common | 4 |
| dark elf | hostile | night | family | keen | dark | keen | normal | light hurts the eyes | walk, climb | speaks | uncommon | 1 |
| half-elf | varies | day | family | normal | low | normal | poor |  | walk, swim, climb | speaks | uncommon |  |
| half-orc | varies | any | family | normal | low | normal | normal |  | walk, swim, climb | speaks | uncommon | 1 |
| gnome | friendly | any | family | normal | low | keen | normal | feel stone | walk, burrow (slow) | speaks | common | 3 |
| goblin | hostile | night | pack | normal | dark | keen | normal |  | walk, climb | cunning | common | 5 |
| hobgoblin | hostile | night | pack | normal | dark | normal | normal |  | walk | speaks | common | 4 |
| orc | hostile | night | pack | normal | dark | normal | keen | light dazzles | walk | speaks | common | 5 |
| bugbear | hostile | night | pack | normal | dark | keen | keen |  | walk, climb | cunning | uncommon | 2 |
| kobold | hostile | night | colony | normal | dark | normal | keen | light dazzles | walk, climb | cunning | common | 6 |
| gnoll | hostile | night | pack | normal | dark | keen | keen |  | walk, run | cunning | common | 2 |
| ogre | hostile | any | family | normal | low | normal | keen |  | walk | cunning | common | 6 |
| troll | hostile | night | solitary | normal | dark | normal | keen | sunlight turns to stone (Norse) | walk, swim | cunning | common | 6 |
| lizardfolk | territorial | day | family | normal | none | normal | normal | hold breath long | walk, swim | speaks | common | 2 |
| ratfolk | varies | night | colony | poor | dark | keen | keen |  | walk, climb, swim | speaks | uncommon |  |
| catfolk | neutral | dawn_and_dusk | family | keen | low | keen | normal |  | walk, run, climb | speaks | uncommon | 1 |
| dogfolk | neutral | day | pack | normal | low | keen | keen |  | walk, run | speaks | uncommon |  |
| birdfolk | neutral | day | flock | keen | none | normal | poor |  | walk, fly | speaks | uncommon |  |
| frogfolk | territorial | night | colony | normal | low | normal | poor | feel vibration in water | walk, hop, swim | speaks | uncommon |  |
| troglodyte | hostile | night | colony | poor | dark | normal | keen | stench | walk, climb | cunning | uncommon |  |
| merfolk | varies | any | family | normal | low | keen | poor |  | swim | speaks | common | 2 |
| centaur | neutral | day | herd | keen | none | normal | normal |  | walk, run | speaks | uncommon | 5 |
| satyr | friendly | night | family | normal | low | keen | keen |  | walk, run, climb | speaks | common | 3 |
| minotaur | hostile | any | solitary | normal | low | normal | keen | never lost in mazes (games) | walk, run | cunning | uncommon | 5 |
| harpy | hostile | day | flock | keen | none | normal | normal | foul stench | walk, fly | cunning | common | 4 |
| naga | varies | any | family | normal | low | normal | keen | taste air | slither, swim | speaks | uncommon | 6 |
| lamia | hostile | night | solitary | normal | low | normal | keen |  | slither | speaks | rare | 1 |
| tengu | varies | day | flock | keen | none | normal | normal |  | walk, fly | speaks | uncommon | 3 |
| oni | hostile | night | family | normal | low | normal | normal |  | walk | speaks | uncommon | 2 |
| yeti | territorial | any | solitary | normal | low | keen | keen | resists cold | walk, climb | animal | rare | 3 |
| sasquatch | timid | night | solitary | normal | low | keen | keen | strong smell | walk, climb | animal | rare | 2 |
| neanderthal | varies | day | family | normal | none | normal | normal |  | walk, swim, climb | speaks | uncommon | 1 |
| scorpion man | territorial | day | family | normal | dark | normal | normal |  | walk | speaks | rare |  |
| gorgon (snake-haired) | hostile | any | solitary | normal | low | normal | normal | gaze turns to stone | walk | speaks | rare | 3 |
| grimlock | hostile | any | pack | none | none | keen | keen | blindsight (hearing and smell) | walk, climb | cunning | uncommon | 1 |
| sahuagin | hostile | night | pack | normal | dark | normal | keen | smells blood in water | swim, walk | speaks | uncommon | 1 |
| wild man | timid | any | solitary | normal | low | keen | keen |  | walk, climb | animal | uncommon |  |
| garuda | varies | day | solitary | keen | none | normal | poor |  | walk, fly | speaks | rare |  |
| medusa | hostile | any | solitary | normal | low | normal | normal | gaze turns to stone | walk | speaks | unique |  |
| baba yaga | varies | any | solitary | normal | low | normal | keen | smells the Russian blood | walk, fly (mortar) | speaks | unique |  |

## Giant

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| giant | People three times a man's height, living in clans in mountains, hills and cold lands. | biped | G | 2×2 | 4.5-6.0 |  | 1500-5000 | skin, hair | as human, also grey, blue-grey |
| hill giant | Crude, greedy giants of the hills who raid farms for cattle. | biped | G | 2×2 | 4.5-5.0 |  | 1500-2500 | skin, hair | tan, ruddy, brown |
| frost giant (jotunn) | Pale, bearded giants of ice and snow who raid in winter. | biped | G | 2×2 | 5.0-6.0 |  | 2500-4000 | skin, hair, beards | skin pale blue-white; hair white, pale blue, yellow |
| fire giant | Coal-skinned giants with fiery hair, great smiths of volcanic halls. | biped | G | 2×2 | 4.5-5.5 |  | 3000-4500 | skin, hair | skin coal black, dark grey; hair flame red, orange |
| stone giant | Grey, reclusive giants who live in caves and throw boulders. | biped | G | 2×2 | 5.0-6.0 |  | 3000-5000 | stone-like skin | grey, granite, sandstone |
| storm giant | The largest giants, rulers of clouds and coasts, slow to anger. | biped | G+ 2x2 | 2×2 | 7.0-8.0 |  | 6000-10000 | skin, hair | skin pale violet, blue-grey, bronze; hair silver, dark blue |
| cyclops | One-eyed giants who herd sheep and eat travellers who come into their caves. | biped | G | 2×2 | 4.0-6.0 |  | 1500-4000 | skin, hair | tan, ruddy, brown |
| ettin (two-headed giant) | Two-headed giants whose heads quarrel, but never both sleep at once. | biped | H | 2×2 | 3.5-4.5 |  | 1500-2500 | skin, coarse hair | grey, brown, filthy |
| titan | Primeval godlike giants, older than the gods who overthrew them. | biped | G+ 2x2 | 2×2 | 8.0-12.0 |  | 10000-30000 | skin, hair | bronze, golden, pale |
| nephilim | Giant children of fallen angels and mortal women. | biped | G | 2×2 | 3.5-4.5 |  | 600-1200 | skin, hair | as human |
| grendel | A monster of the fens that raided a king's hall by night. | biped | H | 2×2 | 3-4 | - | 800-1500 | skin, hair | grey, dark |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| giant | varies | day | family | normal | none | normal | normal |  | walk | speaks | rare | 5 |
| hill giant | hostile | day | family | normal | none | normal | normal |  | walk | cunning | rare |  |
| frost giant | hostile | day | family | normal | none | normal | normal | resists cold | walk | speaks | rare |  |
| fire giant | hostile | day | family | normal | low | normal | normal | resists fire | walk | speaks | rare |  |
| stone giant | neutral | any | family | normal | dark | normal | normal |  | walk, climb | speaks | rare |  |
| storm giant | neutral | day | solitary | keen | none | keen | normal |  | walk, swim | speaks | very rare |  |
| cyclops | hostile | day | solitary | poor | none | normal | keen | one eye: no depth | walk | cunning | rare | 4 |
| ettin | hostile | any | solitary | keen | low | normal | normal | two heads: hard to surprise | walk | cunning | rare | 4 |
| titan | varies | day | solitary | keen | none | keen | normal |  | walk | speaks | very rare | 3 |
| nephilim | hostile | day | family | normal | none | normal | normal |  | walk | speaks | rare |  |
| grendel | hostile | night | solitary | normal | dark | keen | keen | weapons can't cut it | walk, swim | cunning | unique |  |

## Fae

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fairy (fae) | Small winged folk of the hidden world, beautiful, fickle and dangerous to cross. | biped-winged | T | <1 | 0.15-0.60 | wingspan 0.3-0.8 | 0.2-3 | skin, hair, insect wings | skin pale, green, blue; hair any bright colour |
| pixie | Tiny, mischievous fairies of Cornwall who lead travellers astray. | biped-winged | T | <1 | 0.15-0.40 | wingspan 0.3 | 0.2-1 | skin, insect wings | skin green, pale; hair green, red |
| sprite | Tiny winged spirits of air, water and flowers. | biped-winged | T | <1 | 0.20-0.50 | wingspan 0.4 | 0.3-2 | skin, insect wings | skin green, blue, pale |
| brownie (hob, domovoi, tomte) | A small shaggy house spirit who works at night, and leaves if thanked with clothes. | biped | T | <1 | 0.50-0.90 |  | 6-15 | skin, shaggy brown hair | brown, ruddy |
| leprechaun | A small Irish shoemaker who hides a crock of gold and grants wishes when caught. | biped | T | <1 | 0.60-0.90 |  | 8-15 | skin, red beard | ruddy; red, ginger hair |
| redcap | A murderous little goblin of ruined towers who dyes his cap in his victims' blood. | biped | S | 1×1 | 0.90-1.20 |  | 20-35 | skin, wild hair | pale grey, weathered; cap dyed red with blood |
| boggart (bogle) | A house spirit turned bad, that breaks things and torments a household. | biped | S | 1×1 | 0.80-1.20 |  | 15-30 | skin, coarse hair | brown, grey, dirty |
| gremlin | A small imp that sabotages machines and devices. | biped | T | <1 | 0.30-0.60 |  | 2-6 | skin, sparse hair | grey, green-grey, brown |
| spriggan | Ugly fairy guards of old stones and treasure, able to swell to a giant's size. | biped | S | 1×1 | 0.60-1.20 (can grow to giant) |  | 10-30 | skin, bark-like | brown, grey, mossy |
| changeling | A fairy child left in the place of a stolen human baby. | biped | S | 1×1 | 0.80-1.20 |  | 10-25 | skin, hair | pale, sallow, wizened child |
| dryad | A tree spirit in the form of a woman, bound to one tree. | biped | M | 1×1 | 1.50-1.80 |  | 40-60 | skin like bark, leaf hair | skin brown, green, silver-grey; hair green, autumn colours |
| nymph | Beautiful spirits of rivers, springs, trees and mountains. | biped | M | 1×1 | 1.50-1.75 |  | 40-60 | skin, long hair | skin fair to tan; hair any |
| siren | Bird-women whose song lures sailors onto the rocks. | biped-winged | M | 1×1 | 1.50-1.70 | wingspan 3 | 30-50 | skin, feathers | skin pale; feathers white, grey, brown |
| hag (sea hag, night hag) | An ugly old witch who curses, bargains and eats people, often in a coven of three. | biped | M | 1×1 | 1.50-1.80 |  | 45-80 | skin, wild hair | green, grey, blue (sea hag), pale, warty |
| kitsune (fox spirit) | A fox spirit that gains tails and powers with age and can take a woman's shape. | quad | S | 1×1 | length 0.6-0.9 + tails |  | 5-10 | fur | red, gold, white, silver; up to nine tails |
| tanuki | A jolly raccoon-dog spirit that shapeshifts to trick people. | quad | T | <1 | length 0.5-0.7 |  | 4-10 | fur | brown, grey, black mask |
| nekomata | A cat that has lived too long, with a forked tail and a taste for the dead. | quad | S | 1×1 | length 1.0-1.5 |  | 10-30 | fur | any cat colour, two tails |
| jorogumo | A spider spirit that takes the form of a beautiful woman to lure men. | octopod | L | 1×1 | legs 3-4 across |  | 150-300 | chitin, woman's form | black, yellow, red banded |
| selkie | Seals that shed their skins to walk on land as people. | quad | M | 1×1 | 1.50-1.80 / seal 1.5-2 long |  | 60-100 | seal fur or skin | grey, brown, spotted |
| kelpie (each-uisge) | A water horse that lets travellers ride, then drags them into the water. | quad | L | 1×2 | 1.5-1.7 at shoulder | 2.4 | 400-600 | wet coat, weed in mane | black, grey, white |
| trow | Shy, grey little folk of Orkney and Shetland who live in mounds. | biped | S | 1×1 | 0.6-1.2 |  | 10-30 | skin, hair | grey, pale |
| knocker (tommyknocker) | Small mine spirits whose knocks warn of a cave-in or show ore. | biped | T | <1 | 0.5-0.7 |  | 5-12 | skin, beard | grey, weathered |
| pooka (puca) | A shapeshifting Irish spirit, often a black horse, that takes riders on wild rides. | quad | M | 1×1 | varies |  | varies | fur | black |
| nixie (nix, nokken) | A water spirit of Germanic rivers that drowns the careless. | biped | M | 1×1 | 1.40-1.80 |  | 40-70 | skin, wet hair | pale, green, grey |
| naiad (nereid) | A nymph of springs and rivers (nereids are the sea's). | biped | M | 1×1 | 1.50-1.75 |  | 40-60 | skin, wet hair | skin pale, blue-green; hair green, dark blue |
| bakeneko | A cat that grows old, walks on two legs and shapeshifts. | quad | T | <1 | length 0.5-0.8 |  | 4-8 | fur | any cat colour |
| rokurokubi | A woman whose neck stretches at night. | biped | M | 1×1 | 1.5-1.7 (neck stretches) |  | 45-60 | skin, hair | as human |
| tikbalang | A tall horse-headed trickster that leads travellers astray. | biped-dg | L | 1×1 | 2.2-2.7 |  | 150-250 | coarse hair | dark brown, black |
| bogeyman | The shadowy figure that takes children who misbehave. | biped | M | 1×1 | varies |  | varies | shadow, rags | dark |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fairy | varies | any | family | keen | low | keen | normal | senses magic | walk, fly | speaks | common |  |
| pixie | varies | night | family | keen | low | keen | normal |  | walk, fly | speaks | common | 2 |
| sprite | varies | day | family | keen | low | keen | normal |  | walk, fly | speaks | common | 1 |
| brownie | friendly | night | solitary | normal | dark | keen | normal |  | walk | speaks | uncommon |  |
| leprechaun | varies | day | solitary | normal | low | keen | normal |  | walk | speaks | uncommon | 1 |
| redcap | hostile | night | solitary | normal | dark | normal | keen |  | walk, run | cunning | uncommon |  |
| boggart | hostile | night | solitary | normal | dark | normal | normal |  | walk | cunning | uncommon | 1 |
| gremlin | hostile | night | swarm | normal | dark | keen | normal | senses machines | walk, climb | cunning | uncommon | 1 |
| spriggan | hostile | night | family | normal | dark | normal | normal |  | walk | cunning | uncommon | 1 |
| changeling | varies | any | solitary | normal | low | normal | normal |  | walk | speaks | rare |  |
| dryad | timid | day | solitary | normal | none | normal | normal | feels her tree | walk | speaks | uncommon | 3 |
| nymph | timid | day | family | normal | none | normal | normal |  | walk, swim | speaks | uncommon | 3 |
| siren | hostile | day | flock | keen | none | keen | normal | song lures | walk, fly, swim | speaks | uncommon | 1 |
| hag | hostile | night | family | normal | dark | normal | keen |  | walk, swim | speaks | uncommon | 1 |
| kitsune | varies | night | solitary | keen | dark | keen | keen |  | walk, run | speaks | rare |  |
| tanuki | friendly | night | family | normal | low | normal | keen |  | walk | speaks | uncommon |  |
| nekomata | hostile | night | solitary | keen | dark | keen | normal |  | walk, climb | cunning | rare | 1 |
| jorogumo | hostile | night | solitary | normal | dark | normal | normal | feels web | walk, climb | speaks | rare | 1 |
| selkie | timid | any | family | normal | low | keen | normal |  | swim, walk | speaks | uncommon |  |
| kelpie | hostile | any | solitary | normal | low | normal | keen |  | walk, run, swim | cunning | uncommon |  |
| trow | timid | night | family | normal | dark | normal | normal | sunlight traps them | walk | speaks | uncommon |  |
| knocker | varies | any | family | normal | dark | keen | normal | knocks warn of cave-ins | walk | speaks | uncommon |  |
| pooka | varies | night | solitary | normal | dark | normal | keen |  | walk, run | speaks | rare |  |
| nixie | hostile | any | solitary | normal | low | normal | normal |  | swim, walk | speaks | uncommon |  |
| naiad | timid | day | family | normal | low | normal | normal |  | walk, swim | speaks | uncommon |  |
| bakeneko | varies | night | solitary | keen | dark | keen | normal |  | walk, climb | speaks | uncommon |  |
| rokurokubi | varies | night | solitary | normal | low | normal | normal |  | walk | speaks | rare |  |
| tikbalang | hostile | night | solitary | normal | dark | normal | keen |  | walk, run | cunning | rare |  |
| bogeyman | hostile | night | solitary | normal | dark | keen | keen |  | walk, climb | cunning | rare |  |

## Spirit

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| will-o'-wisp (wisp, ignis fatuus) | Floating lights over marshes that lead travellers astray. | floating | T | <1 | 0.2-0.5 glow |  | 0 | light | pale blue, green, white, yellow |
| genie (djinn, jinn, djinni) | A spirit of smokeless fire that can be bound to serve, and grants wishes. | floating | L | 1×1 | 2.5-4.0 |  | 100-300 | skin, smoke below the waist | blue, grey, green, gold |
| efreet (ifrit, efreeti) | A powerful, cruel fire spirit, the strongest of the jinn. | floating | L | 1×1 | 3.0-4.0 |  | 300-600 | skin like coals, smoke | red, black, orange |
| elemental (fire, water, earth) | A living mass of fire, water, earth or air. | amorphous | L | 1×1 | 2.0-5.0 |  | varies | its element | fire: red, orange; water: blue, clear; earth: brown, grey; air: pale, nearly clear |
| sylph | An airy spirit, graceful and nearly invisible. | floating | M | 1×1 | 1.5-1.8 |  | 10-30 | misty skin | pale, white, sky blue |
| undine | A water spirit in the shape of a woman. | amorphous | M | 1×1 | 1.5-1.8 |  | 50-70 | water | clear, blue, green |
| salamander (fire spirit) | A lizard that lives in fire and is not burned. | serpent | M | 1×1 | length 1.5-3 |  | 50-150 | flame-wreathed scales | red, orange, black |
| vortex (whirlwind) | A whirling storm of wind, dust, fire or ice. | amorphous | M | 1×1 | 1.5-3.0 |  | 0 | wind and grit | grey, dusty, fiery, icy |
| mephit (SRD) | A small elemental imp of dust, ice, magma or steam. | biped-winged | S | 1×1 | 1.0-1.2 | wingspan 1.5 | 10-30 | its element | dust grey, ice blue, magma red, steam white |
| azer (SRD) | A dwarf-like fire spirit with flames for hair. | biped | M | 1×1 | 1.2-1.5 | - | 100-150 | brass skin, fire hair | bronze, brass, flame |
| magmin (SRD) | A little creature of living magma that sets fires. | biped | S | 1×1 | 0.9-1.2 | - | 20-40 | magma | black crust, glowing orange |
| invisible stalker (SRD) | An invisible air spirit bound to hunt a target. | amorphous | L | 1×1 | 2.4 |  | - | invisible air | invisible |
| kodama | Tree spirits of old forests, whose echo answers in the hills. | floating | T | <1 | 0.3-0.6 |  | 1-5 | pale glow | white, pale green |
| yuki-onna | A pale snow woman who freezes travellers with her breath. | biped | M | 1×1 | 1.6-1.8 |  | 0-50 | pale skin, long black hair | white, blue-white |
| tsukumogami (animated object) | An old tool or object that has come to life after a hundred years. | amorphous | T | <1 | varies |  | varies | the object | as the object |
| familiar | A spirit in animal form that serves and spies for a witch or wizard. | quad | T | <1 | varies |  | varies | as its animal | as its animal |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| will-o'-wisp | hostile | night | solitary | normal | dark | normal | none | senses life | float | cunning | uncommon | 3 |
| genie | varies | any | solitary | keen | dark | keen | normal |  | float, fly | speaks | rare | 4 |
| efreet | hostile | any | solitary | normal | dark | normal | normal | resists fire | float, fly | speaks | rare | 3 |
| elemental | hostile | any | solitary | normal | dark | normal | none | feel their element | varies by element | mindless | uncommon | 5 |
| sylph | timid | day | solitary | keen | none | keen | none |  | fly | speaks | rare | 1 |
| undine | timid | any | solitary | normal | low | normal | none |  | swim | speaks | rare | 1 |
| salamander (fire spirit) | hostile | any | solitary | normal | dark | normal | none | resists fire | walk, slither | cunning | uncommon | 6 |
| vortex | hostile | any | solitary | none | none | none | none | feel air | fly | mindless | uncommon | 4 |
| mephit | hostile | any | family | normal | dark | normal | normal |  | walk, fly | speaks | common | 1 |
| azer | neutral | any | family | normal | dark | normal | normal | resists fire | walk | speaks | rare | 1 |
| magmin | hostile | any | solitary | normal | dark | normal | none | burns | walk | cunning | uncommon | 1 |
| invisible stalker | hostile | any | solitary | normal | dark | normal | none | tracks anything | fly | speaks | rare | 4 |
| kodama | timid | night | colony | normal | dark | normal | none | feels the forest | float | animal | uncommon |  |
| yuki-onna | hostile | night | solitary | normal | dark | normal | none | resists cold | float, walk | speaks | rare |  |
| tsukumogami | varies | night | solitary | normal | dark | normal | none |  | walk, hop | cunning | uncommon |  |
| familiar | friendly | any | solitary | keen | dark | keen | keen | shares senses with its master | as its animal | cunning | uncommon |  |

## Celestial

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| angel (planetar, solar) | A winged messenger and warrior of heaven. | biped-winged | M | 1×1 | 1.8-2.4 | wingspan 4-6 | 80-150 | skin, feathers | skin golden, pale, bronze, dark; wings white, gold |
| cherub | A heavenly guardian with four faces and four wings (not a winged baby). | biped-winged | M | 1×1 | 1.8-2.4 | wingspan 4 | 100-200 | feathers | gold, white |
| seraph | A burning, six-winged angel of the highest order. | biped-winged | L | 1×1 | 2.5-3.0 | wingspan 6 | 150-250 | fiery feathers | flame red, gold, white |
| deva | A shining divine being of Indian traditions. | biped-winged | M | 1×1 | 1.8-2.2 | wingspan 4 | 80-150 | skin, feathers | skin blue, gold, pale; wings white |
| archon | A ruler of the heavenly spheres, guardian of the way between them. | biped-winged | M | 1×1 | 1.8-2.4 |  | 80-150 | light, armour | white, gold |
| valkyrie | A warrior maiden who chooses the slain and carries them to the hall of heroes. | biped | M | 1×1 | 1.7-2.0 |  | 60-90 | skin, hair | fair; blond, red, dark hair |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| angel | varies | day | family | keen | dark | keen | normal | senses lies and evil | walk, fly | speaks | rare | 3 |
| cherub | neutral | any | family | keen | dark | keen | normal |  | fly | speaks | rare | 1 |
| seraph | neutral | any | family | keen | dark | keen | normal |  | fly | speaks | very rare | 1 |
| deva | friendly | day | family | keen | dark | keen | normal |  | walk, fly | speaks | rare | 2 |
| archon | neutral | day | family | keen | dark | keen | normal |  | walk, fly | speaks | rare | 1 |
| valkyrie | neutral | day | family | keen | none | keen | normal |  | walk, ride, fly (winged horse) | speaks | rare | 1 |

## Fiend

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| rakshasa | Shapeshifting demons of India who eat people and wear their faces. | biped | M | 1×1 | 1.70-2.00 |  | 70-110 | fur, skin | tiger orange with black stripes, black |
| demon | A fiend of chaos and destruction from the lower planes. | biped-dg | L | 1×1 | 2.0-3.5 |  | 150-500 | hide, scales | red, black, grey, dark green, purple |
| devil | A fiend of order and bargains, bound by the letter of every contract. | biped-dg | L | 1×1 | 2.0-3.0 |  | 150-400 | hide | red, black, brass, bone white |
| imp | A tiny winged devil, often a witch's servant. | biped-winged | T | <1 | 0.4-0.7 | wingspan 0.8 | 2-5 | leathery skin | red, black, green |
| succubus (incubus) | A demon who seduces mortals in their sleep (the incubus is the male form). | biped-winged | M | 1×1 | 1.6-1.9 | wingspan 3 | 50-70 | skin | pale, red, purple |
| lemure | The restless dead of Rome; in games, shapeless blobs of tormented flesh. | amorphous | M | 1×1 | 1.2-1.6 |  | 50-80 | melted flesh | pink-red, grey |
| hellhound (hell hound) | A black, fire-breathing hound from the underworld. | quad | M | 1×1 | shoulder 0.9-1.2 | 1.5-2.0 | 70-150 | fur | black, rust red, smouldering |
| quasit (SRD) | A tiny shapeshifting demon that serves as a familiar. | biped-winged | T | <1 | 0.5 | - | 3-5 | skin | green, grey |
| balor (SRD) | A huge winged demon wreathed in flame, with a whip and a sword. | biped-winged | H | 2×2 | 3.5-4 | wingspan 8 | 2000-4500 | hide, flames | red, black, burning |
| marilith (SRD) | A six-armed snake-tailed demon general. | serpent-torso | L | 1×1 | head to tail 5 |  | 500-1000 | skin, scales | green-grey skin, scales |
| vrock (SRD) | A vulture-like demon that screeches and spreads spores. | biped-winged | L | 1×1 | 2.4 | wingspan 5 | 300-500 | feathers | grey, black, vulture-like |
| hezrou (SRD) | A toad-like demon with a crippling stench. | biped | L | 1×1 | 2.4 | - | 600-900 | warty hide | brown-green, grey |
| glabrezu (SRD) | A towering demon with pincers, which grants wishes that betray. | biped | H | 2×2 | 4.5 | - | 2000-3000 | hide | red, brown, purple |
| nalfeshnee (SRD) | A boar-like demon judge with tiny wings. | biped-winged | H | 2×2 | 3.5-4 | - | 2000-4000 | hide, tiny wings | ochre, brown |
| pit fiend (SRD) | The greatest of the devils, a winged general of hell. | biped-winged | L | 1×1 | 3.0-3.5 | wingspan 6 | 500-1000 | scales | red, black |
| nightmare (horse) | A black horse with burning mane and hooves that carries fiends. | quad | L | 1×2 | shoulder 1.6-1.8 | 2.5 | 500-700 | coat, burning mane and hooves | black |
| fury (erinyes, erinys) | One of three winged avengers who pursue the guilty. | biped-winged | M | 1×1 | 1.6-1.9 | wingspan 3.5 | 50-80 | skin, snakes in hair | black, grey, pale |
| dretch (SRD) | The lowest, weakest kind of demon. | biped | S | 1×1 | 1.0-1.2 | - | 20-40 | flabby skin | grey, pink |
| kasha | A cat-like demon that steals corpses from funerals. | quad | L | 1×1 | 1.0-1.4 at shoulder | 2.0 | 150-300 | fur, flames | black, orange flames |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| rakshasa | hostile | night | solitary | normal | low | keen | keen | sees through disguises | walk | speaks | rare | 2 |
| demon | hostile | any | varies | normal | dark | keen | keen | resists fire | walk, fly (some) | speaks | uncommon | 5 |
| devil | hostile | any | family | normal | dark | keen | normal | resists fire | walk, fly (some) | speaks | uncommon | 3 |
| imp | hostile | any | solitary | normal | dark | keen | normal |  | walk, fly | speaks | common | 6 |
| succubus | hostile | night | solitary | normal | dark | normal | normal | charms | walk, fly | speaks | rare | 2 |
| lemure | hostile | any | swarm | poor | dark | poor | poor |  | walk (slow) | mindless | common | 4 |
| hellhound | hostile | night | pack | normal | dark | keen | keen | breathes fire | walk, run | animal | uncommon | 5 |
| quasit | hostile | any | solitary | normal | dark | normal | normal | invisible at will | walk, fly | speaks | uncommon | 3 |
| balor | hostile | any | solitary | keen | dark | keen | keen | fire aura | walk, fly | speaks | very rare | 1 |
| marilith | hostile | any | solitary | normal | dark | normal | normal | six arms | slither | speaks | very rare | 3 |
| vrock | hostile | any | flock | normal | dark | normal | keen | spores, screech | walk, fly | speaks | rare | 3 |
| hezrou | hostile | any | pack | normal | dark | normal | keen | stench | walk | speaks | rare | 3 |
| glabrezu | hostile | any | solitary | keen | dark | normal | normal | four arms: two with pincers | walk | speaks | rare | 2 |
| nalfeshnee | hostile | any | solitary | normal | dark | normal | keen |  | walk, fly | speaks | very rare | 3 |
| pit fiend | hostile | any | solitary | keen | dark | normal | normal | fire aura | walk, fly | speaks | very rare |  |
| nightmare (horse) | hostile | night | solitary | normal | dark | normal | keen | resists fire | walk, run, fly | cunning | rare | 1 |
| fury | hostile | any | family | keen | dark | keen | keen | track the guilty | walk, fly | speaks | rare | 4 |
| dretch | hostile | any | swarm | poor | dark | poor | keen | stench | walk | animal | common | 1 |
| kasha | hostile | night | solitary | keen | dark | keen | keen |  | walk, run, fly | cunning | rare |  |

## Undead

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| banshee | A wailing woman spirit whose cry foretells a death in the family. | floating | M | 1×1 | 1.5-1.8 |  | 0 | ghostly, long hair | pale, grey, white; hair silver, red |
| skeleton | The bones of the dead, raised and moving on their own. | biped | M | 1×1 | as living body |  | 10-20% of living | bone | white, yellow, brown, stained |
| zombie | A rotting corpse raised to shamble after the living. | biped | M | 1×1 | as living body |  | as living | rotting skin | grey, green, blue-grey, rotting |
| ghoul | A grey flesh-eater that haunts graveyards and eats the dead. | biped | M | 1×1 | 1.5-1.9 |  | 40-70 | grey skin | grey, grey-green, pale |
| ghast | A stronger, fouler ghoul whose stench sickens. | biped | M | 1×1 | 1.5-1.9 |  | 40-70 | grey skin | grey-black |
| mummy | A preserved, wrapped corpse risen to guard a tomb. | biped | M | 1×1 | as living body |  | as living, dried | wrappings | tan, brown, grey linen; dark skin |
| ghost (spirit, phantom) | The spirit of a dead person, bound to where it died. | floating | M | 1×1 | as living |  | 0 | pale shape | pale grey, white, faint blue |
| poltergeist | An unseen spirit that throws things and makes noise. | floating | - | 1×1 | unseen |  | 0 | none | unseen |
| spectre (specter) | A terrifying ghost that drains the life of those it touches. | floating | M | 1×1 | as living |  | 0 | dark shape | black, grey |
| wraith | A shadowy undead in tattered robes that hates the living. | floating | M | 1×1 | 1.6-2.2 |  | 0 | tattered shroud | black, grey |
| wight (barrow-wight) | An undead barrow-dweller, cold and cunning, that drains life. | biped | M | 1×1 | as living |  | as living | withered skin | grey, white |
| shade (shadow) | A shadow of the dead, dim and thin. | floating | M | 1×1 | as living |  | 0 | shadow | black, grey |
| vampire | An undead that drinks the blood of the living and fears the sun. | biped | M | 1×1 | as living |  | as living | pale skin | pale, ashen, ruddy after feeding |
| lich | A wizard who made itself undead to live forever, keeping its soul hidden away. | biped | M | 1×1 | as living |  | dried | dried skin over bone | grey, brown, bone |
| revenant | A corpse that returns to take vengeance on its killer. | biped | M | 1×1 | as living |  | as living | rotting skin | grey, pale |
| draugr (draug) | A Norse undead in its barrow, swollen and strong, guarding treasure. | biped | L | 1×1 | as living, swollen |  | much heavier | swollen, dark skin | black, blue-black, corpse pale |
| jiangshi (hopping vampire) | A stiff Chinese hopping corpse that sucks the life breath of the living. | biped | M | 1×1 | as living |  | as living | stiff, grey-green skin | grey-green, pale |
| dullahan (headless horseman) | A headless rider who carries his own head and names the dying. | biped | M | 1×1 | as living (head under arm) |  | as living | pale skin, black cloak | pale, black |
| death knight | A fallen knight raised as an undead warrior. | biped | M | 1×1 | as living |  | as living | armour over dried flesh | dark steel, bone |
| bone dragon (dracolich) | A dragon's skeleton raised as undead. | quad-winged | G+ | 2×2+ | as dragon |  | much lighter | bone | bone white, yellow |
| rusalka | The ghost of a drowned girl that lures men into rivers. | biped | M | 1×1 | 1.5-1.75 |  | 40-60 | pale skin, green wet hair | pale, green hair |
| white lady | A pale female ghost seen in castles and on lonely roads. | floating | M | 1×1 | 1.5-1.75 |  | 0 | pale gown | white |
| gashadokuro (giant skeleton) | A giant made of the bones of the unburied dead. | biped | G | 2×2 | 15 (folklore) |  | 0 | bone | bone white |
| strigoi (moroi) | A Romanian vampire, a troubled soul risen from the grave. | biped | M | 1×1 | as living |  | as living | pale skin, red hair | pale |
| aswang | A Philippine shapeshifter that eats the dead and unborn. | biped | M | 1×1 | 1.5-1.75 |  | 40-60 | skin | as human by day |
| manananggal | A creature that leaves its lower half behind and flies on bat wings. | biped-winged | M | 1×1 | 1.5-1.75 (upper half) | wingspan 3 | 20-40 | skin, bat wings | pale, grey |
| adze | A vampire that flies as a firefly and drinks blood. | swarm | T | <1 | firefly 0.01 / human 1.5 |  | varies | glow / skin | glowing |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| banshee | hostile | night | solitary | normal | dark | normal | none | wail foretells death | float | speaks | rare | 1 |
| skeleton | hostile | any | swarm | normal | dark | normal | none | senses life | as living, no swim | mindless | common | 4 |
| zombie | hostile | any | swarm | poor | low | normal | normal | smells the living | walk (slow) | mindless | common | 4 |
| ghoul | hostile | night | pack | normal | dark | normal | keen | paralysing touch (games) | walk, run, climb, burrow | cunning | common | 5 |
| ghast | hostile | night | pack | normal | dark | normal | keen | stench | walk, run | cunning | uncommon | 3 |
| mummy | hostile | any | solitary | normal | dark | normal | poor | fire hurts | walk (slow) | cunning | uncommon | 4 |
| ghost | varies | night | solitary | normal | dark | normal | none | passes through walls | float | speaks | common | 6 |
| poltergeist | hostile | night | solitary | normal | dark | normal | none | moves objects | float | cunning | uncommon | 2 |
| spectre | hostile | night | solitary | normal | dark | normal | none | drains life | float | cunning | uncommon | 2 |
| wraith | hostile | night | solitary | normal | dark | normal | none | drains life, hates light | float | cunning | uncommon | 6 |
| wight | hostile | night | family | normal | dark | normal | none | hates sunlight | walk | speaks | uncommon | 4 |
| shade | hostile | any | solitary | normal | dark | normal | none | hides in darkness | float | cunning | uncommon | 4 |
| vampire | hostile | night | family | keen | dark | keen | keen | burned by sunlight | walk, climb, fly (as bat or mist) | speaks | rare | 6 |
| lich | hostile | any | solitary | normal | dark | normal | none | keeps its soul in a phylactery | walk, float | speaks | very rare | 6 |
| revenant | hostile | any | solitary | normal | dark | normal | normal | hunts its killer | walk | speaks | rare | 2 |
| draugr | hostile | night | solitary | normal | dark | normal | normal | grows larger at will | walk, swim (through rock) | speaks | rare | 1 |
| jiangshi | hostile | night | solitary | poor | dark | normal | keen | senses breath | hop | mindless | uncommon | 1 |
| dullahan | hostile | night | solitary | keen | dark | normal | none | headless rider | walk, ride | speaks | rare |  |
| death knight | hostile | any | solitary | normal | dark | normal | none |  | walk, ride | speaks | very rare |  |
| bone dragon | hostile | any | solitary | normal | dark | normal | none |  | walk, fly | cunning | very rare | 1 |
| rusalka | hostile | night | family | normal | low | normal | normal |  | swim, walk | speaks | uncommon |  |
| white lady | varies | night | solitary | normal | dark | normal | none |  | float | speaks | uncommon |  |
| gashadokuro | hostile | night | solitary | normal | dark | normal | none | invisible until it strikes | walk | mindless | very rare |  |
| strigoi | hostile | night | solitary | keen | dark | normal | keen |  | walk, fly | speaks | uncommon |  |
| aswang | hostile | night | solitary | normal | dark | keen | keen | shapeshifts | walk, fly | speaks | rare |  |
| manananggal | hostile | night | solitary | normal | dark | keen | keen | leaves its legs behind | fly | speaks | rare |  |
| adze | hostile | night | solitary | normal | dark | normal | keen | shapeshifts | fly, walk | speaks | rare |  |

## Construct

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| gargoyle | Winged stone beasts that sit still as statues by day and hunt at night. | biped-winged | M | 1×1 | 1.50-1.90 | wingspan 3 | 200-350 | stone hide | stone grey, granite, sandstone, moss-stained |
| golem | A body of clay, stone or metal animated by magic to obey its maker. | biped | L | 1×1 | 2.0-3.0 |  | 300-2500 | its material | clay brown, flesh, stone grey, iron dark, bronze |
| homunculus | A tiny creature made by an alchemist as a servant. | biped-winged | T | <1 | 0.3-0.5 | wingspan 0.6 | 1-3 | leathery skin | grey, green-grey, flesh |
| animated armour (animated armor, flying sword, rug of smothering) | A suit of armour that walks and fights with nobody inside. | biped | M | 1×1 | as worn |  | armour's weight | armour, empty | steel, bronze, any armour |
| automaton (clockwork) | A clockwork figure that moves by itself. | biped | M | 1×1 | 1.5-2.0 |  | 150-400 | metal, gears | brass, bronze, iron |
| living statue | A statue that comes to life. | biped | M | 1×1 | as carved |  | 400-1500 | stone or metal | marble white, grey, bronze |
| scarecrow | A scarecrow that comes to life at night. | biped | M | 1×1 | 1.6-2.0 |  | 5-15 | straw, sackcloth | straw yellow, brown, faded cloth |
| mannequin | A tailor's dummy that moves when nobody is looking. | biped | M | 1×1 | 1.6-1.8 |  | 10-30 | painted wood | pale wood, painted |
| shield guardian (SRD) | A construct made to protect whoever wears its amulet. | biped | L | 1×1 | 2.5-3.0 |  | 500-1000 | stone, metal | grey, bronze |
| talos (bronze giant) | A giant of bronze that guarded Crete by throwing rocks. | biped | G+ 2x2 | 2×2 | 10-30 |  | 50000+ | bronze | bronze |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| gargoyle | territorial | night | solitary | normal | dark | normal | poor | still as stone | walk, fly, climb | cunning | uncommon | 4 |
| golem | neutral | any | solitary | normal | dark | normal | none | mindless obedience | walk (slow) | mindless | uncommon | 6 |
| homunculus | friendly | any | solitary | normal | dark | normal | normal | shares senses with maker | walk, fly | speaks | uncommon | 3 |
| animated armour | hostile | any | solitary | normal | dark | normal | none | senses life | walk | mindless | uncommon | 2 |
| automaton | neutral | any | solitary | normal | dark | normal | none |  | walk | mindless | rare | 1 |
| living statue | hostile | any | solitary | normal | dark | normal | none |  | walk (slow) | mindless | uncommon | 1 |
| scarecrow | hostile | night | solitary | normal | dark | normal | none | fire burns | walk | mindless | uncommon |  |
| mannequin | hostile | night | solitary | normal | dark | normal | none |  | walk | mindless | rare | 1 |
| shield guardian | neutral | any | solitary | normal | dark | normal | none |  | walk | mindless | rare |  |
| talos | hostile | day | solitary | keen | none | normal | none | one vein of ichor at the ankle | walk | cunning | unique |  |

## Ooze

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| slime | A small blob of goo that creeps, splits and dissolves things. | amorphous | T | <1 | 0.2-1.0 across |  | 1-50 | slime | green, blue, clear, yellow, red |
| ooze | A spreading puddle of acid that eats through flesh and wood. | amorphous | M | 1×1 | 1-2 across |  | 50-300 | ooze | grey, ochre, black |
| jelly | A wobbling mass that dissolves what it engulfs. | amorphous | S | 1×1 | 0.5-1.5 across |  | 20-200 | jelly | ochre, pink, blue, black |
| pudding | A great blob of black ooze that dissolves metal. | amorphous | L | 1×1 | 1.5-3 across |  | 200-1000 | ooze | black, brown, white |
| gelatinous cube (SRD) | A transparent cube of jelly that fills corridors and digests anything in its path. | amorphous | M | 1×1 | 3x3x3 m fills a corridor (G+ 2x2) |  | 1500-3000 | clear jelly | clear, faint green |
| amoeba (plasmodium) | A giant single cell that engulfs prey. | amorphous | T | <1 | 0.1-1 (giant) |  | varies | membrane | clear, grey |
| gibbering mouther (SRD) | A mass of mouths and eyes that babbles maddeningly. | amorphous | M | 1×1 | 1-1.5 across |  | 100-200 | flesh | pink, grey |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| slime | hostile | any | colony | none | none | none | none | feel vibration | creep, splits | mindless | common | 3 |
| ooze | hostile | any | solitary | none | none | none | none | feel vibration; acid | creep, climb walls | mindless | common | 4 |
| jelly | hostile | any | solitary | none | none | none | none | feel vibration | creep, splits | mindless | common | 6 |
| pudding | hostile | any | solitary | none | none | none | none | feel vibration; dissolves metal | creep, climb | mindless | uncommon | 3 |
| gelatinous cube | hostile | any | solitary | none | none | none | none | feel vibration | creep | mindless | uncommon | 3 |
| amoeba | hostile | any | solitary | none | none | none | none | feel touch | creep | mindless | uncommon | 1 |
| gibbering mouther | hostile | any | solitary | normal | dark | normal | normal | maddening noise | creep, swim | animal | rare | 1 |

## Plant

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| mold (mould) | A patch of mould that releases spores, cold or poison when touched. | plant | T | <1 | 1 cell patch |  | - | mould | yellow, brown, green, red, white |
| fungus (mushroom folk, shrieker) | Mushrooms and fungi: some scream, some poison, some walk. | plant | S | 1×1 | 0.3-1.5 |  | 5-40 | cap and stalk | white, brown, red, purple, glowing |
| treant (SRD) (tree spirit, awakened tree, awakened shrub) | A walking tree, guardian of the forest. | plant | H | 2×2 | 4-6 |  | 2000-5000 | bark, leaves | bark brown, grey; leaves green, autumn |
| mandrake | A root shaped like a person, whose scream kills when pulled up. | plant | T | <1 | 0.2-0.4 |  | 0.5-2 | root, leaves | pale brown root, green leaves |
| carnivorous plant (man-eating plant) | A plant that catches and eats animals, and sometimes people. | plant | L | 1×1 | 1.5-4 |  | 50-500 | stems, leaves | green, red, purple |
| shambling mound (SRD) | A heap of rotting plants that walks and smothers prey. | plant | L | 1×1 | 2.0-2.7 |  | 200-400 | rotting plants | green, brown, black |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| mold | neutral | any | colony | none | none | none | none | spores when touched | none | mindless | common | 3 |
| fungus | neutral | any | colony | none | none | none | none | senses vibration; shriekers scream | none or walk (mushroom folk) | mindless | common | 3 |
| treant | neutral | day | solitary | normal | none | normal | normal | feels the forest | walk (slow) | speaks | rare | 3 |
| mandrake | timid | any | solitary | none | none | keen | none | scream kills | none / walk | animal | uncommon |  |
| carnivorous plant | hostile | any | solitary | none | none | none | none | feel touch, warmth | none | mindless | uncommon | 2 |
| shambling mound | hostile | any | solitary | none | none | normal | normal | feel vibration | walk, swim | mindless | uncommon | 1 |

## Shapeshifter

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| doppelganger | A creature that copies the look of anyone it meets. | biped | M | 1×1 | 1.5-1.9 |  | 50-80 | grey skin (true form) | grey, pale; copies anyone |
| lycanthrope (werewolf, wererat, werebear) | A person who turns into a beast: wolf, rat, bear, boar or tiger. | biped-dg | M | 1×1 | as person; hybrid larger |  | as person + 20% | fur in beast form | as its animal |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| doppelganger | hostile | any | solitary | normal | dark | normal | normal | reads surface thoughts | walk | speaks | rare | 4 |
| lycanthrope | hostile | night | pack | normal | low | keen | keen | silver hurts | walk, run | speaks | uncommon | 4 |

## Dragon

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| dragon | A great winged reptile, clever and ancient, that breathes fire or worse and sleeps on a hoard. | quad-winged | G+ 3x6 (adult) | 3×6 | 3-5 at shoulder | 12-20 (young 3-6; ancient 25-30) | adult 5000-20000 | scales | red, green, black, white, blue, gold, bronze, copper: no fixed meaning (the colour-by-nature set is D&D) |
| wyvern | A two-legged dragon with wings for arms and a venomous sting in its tail. | biped-winged | H | 2×2 | 1.5-2 at shoulder | 6-9 (wingspan 10-14) | 1000-2000 | scales | green, brown, grey, red |
| drake | A wingless dragon that runs on four legs. | quad | L | 1×2 | 1.0-1.5 at shoulder | 3-5 | 300-800 | scales | red, green, brown, grey |
| wyrm (lindworm) | A huge legless or two-legged serpent dragon. | serpent | H | 2×2 | 1 high | 8-15 | 1000-4000 | scales | green, grey, black, gold |
| pseudodragon (SRD) (faerie dragon) | A cat-sized dragon, clever and often a wizard's companion. | quad-winged | T | <1 | 0.2-0.3 | 0.5-0.8 + tail | 2-4 | scales | red-brown, copper |
| hydra | A many-headed serpent whose heads grow back when cut off. | many-headed | H | 2×2 | 2-3 at shoulder | 6-10 | 2000-5000 | scales | green, brown, black, marsh colours |
| sea serpent | An enormous serpent of the deep seas that wrecks ships. | serpent | G+ 2x10 | 2×10 | 1-2 thick | 20-50 | 10000+ | scales | green, blue-black, grey |
| amphisbaena | A serpent with a head at each end. | serpent | S | 1×1 | 0.2 thick | 1-2 | 10-30 | scales | green, black, red |
| mushussu (sirrush) | A scaly dragon with lion forelegs, eagle talons and a scorpion's sting. | quad | L | 1×2 | 1.5 at shoulder | 3-4 | 300-600 | scales | turquoise, gold, brown |
| grootslang | A great serpent with an elephant's head that guards diamonds. | serpent | G | 2×2 | 2-3 | 10-15 | 5000+ | hide | grey |
| fafnir | A dwarf who became a dragon to guard a cursed treasure. | quad | G+ 3x8 | 3×8 | 3 | 15-20 | 10000+ | scales | green, gold, dark |
| ladon | A hundred-headed serpent that guards the golden apples. | many-headed | G+ 3x8 | 3×8 | 2-3 | 15 | 5000+ | scales | gold, green |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| dragon | varies | any | solitary | keen | dark | keen | keen | senses hidden things | walk, fly, swim (some), burrow (some) | speaks | very rare | 6 |
| wyvern | hostile | day | solitary | keen | low | normal | normal |  | walk, fly | animal | rare | 3 |
| drake | hostile | day | solitary | normal | low | normal | keen | breath (some) | walk, run | animal | uncommon | 2 |
| wyrm | hostile | any | solitary | normal | low | normal | keen |  | slither, swim | cunning | rare | 3 |
| pseudodragon | friendly | any | solitary | keen | dark | keen | normal | telepathy | walk, fly | cunning | uncommon | 2 |
| hydra | hostile | any | solitary | keen | dark | normal | keen | several heads: hard to surprise | walk, swim | animal | rare | 4 |
| sea serpent | hostile | any | solitary | normal | dark | normal | normal |  | swim | animal | rare | 2 |
| amphisbaena | hostile | any | solitary | normal | dark | normal | keen |  | slither | animal | rare |  |
| mushussu | neutral | day | solitary | normal | none | normal | normal |  | walk | animal | rare |  |
| grootslang | hostile | any | solitary | normal | dark | normal | normal | hoards diamonds | slither | cunning | rare |  |
| fafnir | hostile | any | solitary | keen | dark | keen | keen | poison breath | walk, slither | speaks | unique |  |
| ladon | territorial | any | solitary | keen | dark | keen | normal | 100 heads, never sleeps | slither | speaks | unique |  |

## Beast

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| basilisk | The king of serpents, whose gaze or breath kills. | quad | S | 1×1 | 0.3-0.5 high | 1-2 | 10-40 | scales | green, brown, yellow; a crown-like crest |
| cockatrice | A rooster with a serpent's tail whose touch turns flesh to stone. | bird | S | 1×1 | 0.4-0.7 | 1.0 | 5-15 | feathers, scales | rooster colours with scaled tail |
| chimera | A fire-breathing beast with the heads of a lion, a goat and a snake. | quad-winged | L | 1×2 | 1.2-1.6 at shoulder | 3-4 | 300-500 | fur, scales | lion tawny, goat grey, snake green |
| manticore | A lion with a man's face, rows of teeth and a tail that shoots spines. | quad-winged | L | 1×2 | 1.3-1.6 at shoulder | 3-4 | 300-500 | fur, leather wings | red, tawny |
| griffin (gryphon, griffon) | An eagle-headed, winged lion that guards treasure. | quad-winged | L | 1×2 | 1.4-1.8 at shoulder | 3-4 (wingspan 7-8) | 300-500 | feathers front, fur back | gold, tawny, brown, white head |
| hippogriff | The offspring of a griffin and a mare: eagle in front, horse behind. | quad-winged | L | 1×2 | 1.5-1.7 at shoulder | 3 (wingspan 7) | 350-500 | feathers front, horse coat back | any horse colour, brown feathers |
| pegasus | A winged horse. | quad-winged | L | 1×2 | 1.5-1.7 at shoulder | 2.5 (wingspan 6) | 400-550 | coat, feathers | white, grey, rarely black |
| unicorn | A horse-like beast with a single healing horn, shy of all but the pure. | quad | L | 1×2 | 1.5-1.7 at shoulder | 2.5 | 400-550 | coat | white, silver, rarely black |
| kirin (qilin) | A gentle, scaled, hoofed beast whose appearance marks a wise ruler. | quad | L | 1×2 | 1.5-1.8 at shoulder | 2.5 | 400-600 | scales, mane | gold, multicoloured scales, flame-like mane |
| phoenix (firebird) | A fiery bird that dies in flames and is born again from its ashes. | bird | M | 1×1 | 1.0-1.5 | wingspan 3-5 | 10-20 | burning feathers | red, gold, orange, purple |
| roc | A bird so large it carries off elephants. | bird | G+ 4x4 | 4×4 | 6-10 | wingspan 15-30 | 2000-10000 | feathers | brown, gold, white |
| thunderbird | A huge spirit bird whose wingbeats are thunder. | bird | G | 2×2 | 3-5 | wingspan 10-15 | 500-2000 | feathers | black, grey, white with lightning |
| sphinx (androsphinx, gynosphinx) | A winged lion with a human head who poses riddles. | quad-winged | L | 1×2 | 1.5-2.5 at shoulder | 3-5 (wingspan 6) | 500-1500 | fur, feathers, skin | tawny lion, gold, human face |
| kraken | A gigantic squid-like monster that drags ships under. | cephalopod | G+ 6x6 | 6×6 | - | 30-60 with arms | 20000+ | smooth skin | red, grey, dark purple |
| catoblepas | A heavy beast whose head hangs low, with a gaze that kills. | quad | L | 1×2 | 1.5 at shoulder | 3 | 500-900 | hide, mane | black, dark brown |
| peryton | A deer with a bird's wings that casts the shadow of a man. | quad-winged | M | 1×1 | 1.2 at shoulder | 2 (wingspan 4) | 50-120 | feathers, stag head | green, blue-green, brown |
| leucrotta | A swift beast with a badger's head and a horse's legs that mimics voices. | quad | L | 1×2 | 1.3 at shoulder | 2.5 | 300-500 | fur | tawny, grey, badger-striped head |
| barghest (black dog) | A huge black dog of the moors, an omen of death. | quad | M | 1×1 | 0.9-1.2 at shoulder | 1.6-2.0 | 60-120 | shaggy fur | black |
| cerberus (three-headed dog) | The three-headed hound that guards the gates of the underworld. | quad | H | 2×2 | 2-3 at shoulder | 4-5 | 1500-2500 | fur, snakes in mane | black |
| wendigo | A gaunt, starving spirit of winter that hungers for human flesh. | biped | L | 1×1 | 2.5-4.5 |  | 50-150 (gaunt) | grey skin, bones showing | grey, ash, antlers in modern art |
| bunyip | A water monster of Australian billabongs. | quad | L | 1×2 | 1.2 at shoulder | 3 | 300-600 | fur, flippers | black, dark brown |
| tarrasque | A six-legged, shelled dragon of southern France, tamed by a saint. | quad | G+ 4x8 | 4×8 | 6-15 | 15-25 | 50000+ | shell, scales | green, brown, dark |
| jackalope (wolpertinger) | A rabbit with antlers, a joke of the American west. | quad | T | <1 | 0.3 at shoulder | 0.5 | 2-5 | fur, antlers | brown, grey |
| feathered serpent (couatl) | A great serpent clothed in feathers, a sky and wind spirit. | serpent | H | 2×2 | 1 thick | 8-15 | 1000-2000 | feathers, scales | green, turquoise, gold, red |
| owlbear (SRD) | A bear with an owl's head, beak and feathers. | biped | L | 1×1 | 2.2-2.6 | - | 400-700 | feathers on head, fur | brown fur, grey-brown feathers |
| rust monster (SRD) | An insect-like beast that rusts and eats metal. | hexapod | M | 1×1 | 1.0 at shoulder | 1.5-1.8 + tail | 100-200 | shell, hide | rusty red, brown |
| bulette (SRD) | A huge armoured beast that burrows and bursts up from below. | quad | H | 2×2 | 2.5-3 | 3-4 | 2000-4000 | armour plates | blue-grey, brown |
| ankheg (SRD) | A giant burrowing insect that sprays acid. | hexapod | L | 1×2 | 1.0-1.5 | 3 | 300-500 | chitin | brown, yellow |
| otyugh (SRD) | A tentacled refuse-eater that lives in filth. | amorphous | L | 1×1 | 1.5-2 across |  | 500-800 | hide | grey, brown, filth-caked |
| roper (SRD) | A cave creature disguised as a stalagmite that lashes out with sticky strands. | amorphous | L | 1×1 | 2.5-3 tall |  | 1000-2000 | stony hide | stone grey, brown |
| darkmantle (SRD) | A cave creature that hangs from the ceiling and drops onto prey. | cephalopod | T | <1 | 0.5-1 across |  | 5-15 | skin | black, grey |
| stirge (SRD) | A small bird-like bloodsucker with a long proboscis. | bird | T | <1 | 0.3 long | wingspan 0.6 | 0.5-1 | feathers, leather | rust red, brown |
| remorhaz (SRD) | A long arctic burrowing beast whose back glows with heat. | myriapod | G | 2×2 | 2 | 8-12 | 3000-6000 | chitin, heated back | blue-white, glowing red back |
| behir (SRD) | A many-legged lightning-breathing serpent. | myriapod | G | 2×2 | 2 | 12 | 3000-5000 | scales | blue, deep blue |
| chuul (SRD) | A lobster-like predator with paralysing mouth tentacles. | hexapod | L | 1×2 | 1.5 | 3 | 300-600 | shell | green-brown, yellow |
| aboleth (SRD) | An ancient, tentacled fish that enslaves minds. | fish | H | 2×2 | 1.5 | 6 | 2000-3000 | slimy skin | blue-green, grey, pink belly |
| xorn (SRD) | A three-armed creature that swims through stone and eats gems. | amorphous | M | 1×1 | 1.5 | 1.5 across | 250-400 | stony hide | grey, brown, speckled |
| ettercap | A spider-like humanoid that keeps giant spiders and sets webs. | biped | M | 1×1 | 1.5-1.8 | - | 60-90 | chitin, hair | black, brown, purple, mottled |
| drider (SRD) | A creature with a person's upper body on a giant spider's. | taur | L | 1×2 | 2.4-3 tall | 2.5 | 300-500 | skin, chitin | dark elf skin, black spider |
| cloaker (SRD) | A flying creature that looks like a black cloak and wraps its prey. | floating | L | 1×1 | wingspan 2.5-3 |  | 50-100 | leathery | black, grey |
| grick (SRD) | A worm-like predator with a beak ringed by tentacles. | serpent | M | 1×1 | 2.5 long |  | 100-200 | rubbery skin | grey, dark green |
| baku | A tapir-like beast that eats bad dreams. | quad | L | 1×2 | 1.4 at shoulder | 2.5 | 500-800 | hide | grey, black |
| nue | A Japanese chimera with a monkey's face and a snake for a tail. | quad | L | 1×2 | 1.0 at shoulder | 2.5 + snake tail | 200-300 | fur | tiger stripes, tanuki brown |
| strix | An owl that feeds on human flesh and blood. | bird | T | <1 | 0.4-0.6 | wingspan 1.2 | 1-2 | feathers | grey, brown |
| chupacabra | A spiny creature that drains the blood of livestock. | biped-dg | S | 1×1 | 0.9-1.2 | - | 20-40 | scaly skin or hairless | grey, green-grey |
| simurgh (homa, huma) | A huge, wise, benevolent bird of Persian legend. | bird | H | 2×2 | 2-4 | wingspan 8-12 | 300-1000 | feathers | copper, gold, peacock colours |
| sirin (alkonost, gamayun) | A bird with a woman's head whose song brings joy, sorrow or forgetting. | bird | M | 1×1 | head + bird 1.2 | wingspan 2.5 | 15-30 | feathers, woman's face | feathers gold, brown, multicoloured |
| fenghuang | A beautiful bird of five colours that appears in times of peace. | bird | M | 1×1 | 1.5-2.5 | wingspan 3-5 | 15-40 | feathers | five colours: black, white, red, yellow, green |
| lamassu (shedu) | A winged bull or lion with a man's head that guards gateways. | quad-winged | H | 2×2 | 3-4 at shoulder | 4-5 (wingspan 8) | 2000-4000 | hide, feathers, curled beard | stone colours in art: tan, gold |
| ammit | Devourer of the dead: crocodile head, lion front, hippo behind. | quad | L | 1×2 | 1.2 at shoulder | 3 | 500-800 | scales, fur | green crocodile head, tawny lion, grey hippo |
| yali | A lion-like temple guardian with an elephant's trunk. | quad | L | 1×2 | 1.3 at shoulder | 2.5 | 300-500 | fur, scales | gold, tawny, stone colours |
| makara | A sea creature with a land animal's front and a fish's tail. | fish | H | 2×2 | 2 | 5-8 | 2000-4000 | scales | green, blue, gold |
| wani | A sea dragon or crocodile of Japanese legend. | serpent | H | 2×2 | 1-1.5 | 6-12 | 2000-5000 | scales | green, dark |
| nian | A beast that comes at the new year and fears red and loud noises. | quad | L | 1×2 | 1.2-1.5 at shoulder | 2.5 | 300-500 | fur | red, gold |
| enfield | A heraldic beast with a fox's head and an eagle's forelegs. | quad | M | 1×1 | 0.9 at shoulder | 1.5 | 50-80 | fur, feathers | red fox colours |
| yale (centicore) | A goat-like beast whose horns swivel in any direction. | quad | L | 1×2 | 1.4 at shoulder | 2.5 | 300-500 | coat | black, tawny, spotted |
| impundulu (lightning bird) | A lightning bird, servant of witches. | bird | M | 1×1 | 1.0-1.5 | wingspan 2-3 | 10-20 | feathers | black and white |
| mokele-mbembe | A large river beast said to live in the Congo swamps. | quad | H | 2×2 | 2-3 at shoulder | 8-10 | 5000-10000 | hide | grey-brown |
| cerberus (unique) |  | quad |  | 1×1 |  |  |  |  |  |
| fenrir | The monstrous wolf bound by the gods, fated to swallow the sun. | quad | G | 2×2 | 3-6 at shoulder | 8-15 | 10000+ | fur | grey, black |
| sleipnir | An eight-legged grey horse, the fastest of all. | quad | L | 1×2 | 1.8 at shoulder | 2.6 | 700-900 | coat | grey |
| scylla | A sea monster with six dog-like heads on long necks. | cephalopod | G | 2×2 | 6-10 | - | 5000+ | skin, scales | pale, grey, dog heads |
| echidna (myth) | Half woman, half snake: the mother of monsters. | serpent-torso | L | 1×1 | head to tail 5 |  | 300-500 | skin, scales | pale skin, green scales |
| nemean lion | A lion whose golden hide no weapon could pierce. | quad | L | 1×2 | 1.2-1.5 at shoulder | 3-3.5 | 300-450 | golden fur (impenetrable) | gold |
| arachne | A weaver turned into a spider for her pride. | taur | L | 1×2 | 2.4-3 tall | 2.5 | 300-500 | skin, chitin | pale skin, black, purple |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| basilisk | hostile | any | solitary | normal | dark | normal | normal | gaze turns to stone or kills | walk | animal | rare | 3 |
| cockatrice | hostile | day | solitary | normal | none | normal | poor | touch turns to stone | walk, fly (short) | animal | uncommon | 3 |
| chimera | hostile | day | solitary | keen | dark | keen | keen | breathes fire | walk, run, fly | animal | rare | 3 |
| manticore | hostile | any | solitary | keen | dark | keen | keen |  | walk, fly | speaks | rare | 3 |
| griffin | territorial | day | family | keen | none | normal | normal |  | walk, fly | animal | uncommon | 3 |
| hippogriff | timid | day | herd | keen | none | normal | normal |  | walk, run, fly | animal | uncommon | 3 |
| pegasus | timid | day | herd | keen | none | normal | normal |  | walk, run, fly | animal | rare | 2 |
| unicorn | timid | dawn_and_dusk | solitary | keen | low | keen | keen | senses purity | walk, run | cunning | rare | 4 |
| kirin | friendly | day | solitary | keen | none | keen | normal | senses justice | walk, run, fly | speaks | very rare | 1 |
| phoenix | friendly | day | solitary | keen | none | normal | poor | reborn from its ashes | walk, fly | cunning | very rare | 3 |
| roc | territorial | day | solitary | keen | none | normal | poor |  | walk, fly | animal | very rare | 2 |
| thunderbird | neutral | day | solitary | keen | none | keen | poor | lightning from its eyes, thunder from its wings | fly | cunning | very rare |  |
| sphinx | varies | day | solitary | keen | dark | keen | normal | riddles | walk, fly | speaks | very rare | 2 |
| kraken | hostile | any | solitary | keen | dark | normal | normal | feel vibration | swim | cunning | very rare | 4 |
| catoblepas | neutral | any | solitary | poor | none | normal | keen | gaze kills (its head hangs down) | walk (slow) | animal | rare | 2 |
| peryton | hostile | day | flock | keen | none | normal | keen | casts a man's shadow | walk, fly | animal | rare |  |
| leucrotta | hostile | night | solitary | normal | dark | keen | keen | mimics voices | walk, run | cunning | rare | 1 |
| barghest | hostile | night | solitary | normal | dark | keen | keen | omen of death | walk, run | cunning | uncommon |  |
| cerberus | hostile | any | solitary | keen | dark | keen | keen | three heads: hard to surprise | walk, run | cunning | unique | 1 |
| wendigo | hostile | night | solitary | keen | dark | keen | keen | never full | walk, run | cunning | rare | 1 |
| bunyip | hostile | night | solitary | normal | dark | normal | normal |  | walk, swim | animal | rare | 1 |
| tarrasque | hostile | any | solitary | normal | dark | normal | keen |  | walk | animal | unique | 2 |
| jackalope | timid | dawn_and_dusk | solitary | normal | low | keen | normal | mimics voices | hop, run | animal | uncommon |  |
| feathered serpent | friendly | day | solitary | keen | dark | keen | normal |  | fly, slither | speaks | very rare | 2 |
| owlbear | hostile | day | pairs | keen | dark | keen | keen |  | walk, climb | animal | uncommon | 2 |
| rust monster | neutral | any | solitary | poor | dark | normal | keen | smells metal | walk | animal | uncommon |  |
| bulette | hostile | day | solitary | normal | dark | normal | normal | feel vibration (tremorsense) | walk, burrow | animal | rare | 1 |
| ankheg | hostile | any | solitary | normal | dark | normal | normal | feel vibration | walk, burrow | animal | uncommon | 1 |
| otyugh | hostile | any | solitary | normal | dark | normal | keen |  | walk | cunning | uncommon | 1 |
| roper | hostile | any | solitary | normal | dark | normal | normal |  | creep, climb | cunning | rare | 1 |
| darkmantle | hostile | any | colony | none | dark | keen | normal | echolocation | fly, climb | animal | uncommon | 1 |
| stirge | hostile | night | flock | normal | dark | normal | keen | smells blood | fly | animal | common | 1 |
| remorhaz | hostile | any | solitary | normal | dark | normal | normal | feel vibration | walk, burrow | animal | rare | 1 |
| behir | hostile | any | solitary | keen | dark | normal | normal |  | walk, climb | animal | rare | 1 |
| chuul | hostile | any | solitary | normal | dark | normal | keen | senses magic | walk, swim | animal | uncommon | 1 |
| aboleth | hostile | any | solitary | normal | dark | normal | normal | telepathy | swim | speaks | rare | 1 |
| xorn | neutral | any | solitary | normal | dark | normal | keen | smells gems and metal | walk, burrow (through stone) | speaks | rare | 4 |
| ettercap | hostile | night | solitary | normal | dark | normal | normal | feel web | walk, climb | cunning | uncommon | 1 |
| drider | hostile | night | solitary | normal | dark | normal | normal | feel web | walk, climb | speaks | rare | 1 |
| cloaker | hostile | night | solitary | normal | dark | normal | normal | moans | fly, glide | speaks | rare | 1 |
| grick | hostile | any | solitary | none | dark | normal | normal | feel vibration | slither, climb | animal | uncommon | 1 |
| baku | friendly | night | solitary | normal | low | normal | keen | eats dreams | walk | animal | rare |  |
| nue | hostile | night | solitary | normal | dark | normal | normal |  | walk, fly | cunning | rare |  |
| strix | hostile | night | flock | keen | dark | keen | poor |  | fly | animal | uncommon |  |
| chupacabra | hostile | night | solitary | normal | dark | normal | keen |  | walk, hop | animal | uncommon |  |
| simurgh | friendly | day | solitary | keen | none | normal | normal | ancient wisdom | fly | speaks | very rare |  |
| sirin | varies | day | solitary | keen | none | keen | poor | song | walk, fly | speaks | rare |  |
| fenghuang | friendly | day | pairs | keen | none | normal | poor |  | walk, fly | cunning | very rare |  |
| lamassu | friendly | day | solitary | keen | dark | normal | normal |  | walk, fly | speaks | very rare |  |
| ammit | hostile | any | solitary | normal | dark | normal | keen |  | walk, swim | animal | unique |  |
| yali | territorial | day | solitary | keen | low | normal | normal |  | walk, run | animal | rare |  |
| makara | neutral | any | solitary | normal | low | normal | normal |  | swim, walk | animal | rare |  |
| wani | hostile | any | solitary | normal | dark | normal | normal |  | swim | cunning | rare |  |
| nian | hostile | night | solitary | normal | dark | keen | normal | afraid of red, loud noises | walk | animal | rare |  |
| enfield | neutral | day | solitary | keen | low | keen | keen |  | walk, run | animal | rare |  |
| yale | neutral | day | herd | normal | none | normal | normal |  | walk, run | animal | uncommon |  |
| impundulu | hostile | any | solitary | keen | dark | normal | normal | lightning | fly | cunning | rare |  |
| mokele-mbembe | territorial | day | solitary | normal | none | normal | normal |  | walk, swim | animal | rare |  |
| cerberus (unique) |  |  |  |  |  |  |  |  |  |  |  |  |
| fenrir | hostile | any | solitary | keen | dark | keen | keen |  | walk, run | speaks | unique |  |
| sleipnir | friendly | day | solitary | keen | none | normal | normal |  | walk, run, fly | animal | unique |  |
| scylla | hostile | any | solitary | keen | dark | keen | keen | six heads | swim | speaks | unique |  |
| echidna (myth) | hostile | any | solitary | normal | dark | normal | keen |  | slither | speaks | unique |  |
| nemean lion | hostile | day | solitary | keen | low | keen | keen | weapons can't cut its hide | walk, run | animal | unique |  |
| arachne | hostile | night | solitary | normal | dark | normal | normal | feel web | walk, climb | speaks | unique |  |

## Game

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| mimic | A shapeshifter that disguises itself as a chest or door to catch the greedy. | amorphous | M | 1×1 | as its disguise |  | 150-300 | sticky skin | looks like a chest, door, stairs |
| floating eye (eye monster) | A huge floating eyeball whose gaze paralyses. | floating | S | 1×1 | 0.6-1.2 across |  | 10-40 | skin | pink, blue, grey |
| tentacle horror | A writhing mass of tentacles from somewhere it should have stayed. | amorphous | L | 1×1 | 2-3 |  | 300-800 | slick skin | grey, purple, green |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| mimic | hostile | any | solitary | normal | dark | normal | normal | sticky | creep | cunning | uncommon | 4 |
| floating eye | neutral | any | solitary | keen | dark | none | none | gaze paralyses | float | animal | common | 4 |
| tentacle horror | hostile | any | solitary | normal | dark | normal | normal | madness | creep | animal | rare | 3 |

## Livestock

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cattle (cow, bull, ox) | Cows, bulls and oxen: milk, meat, leather and pulling ploughs. | quad | L | 1×2 | 1.2-1.5 at shoulder | 2.0-2.6 | 400-900 | short hair | black, brown, red, white, dun, piebald |
| sheep (ewe, ram) | Wool, milk and meat; flocks need shepherds and dogs. | quad | S | 1×1 | 0.6-0.9 at shoulder | 1.1-1.5 | 45-100 | wool | white, cream, black, brown, grey |
| goat | Hardy climbers kept for milk, meat and hair. | quad | S | 1×1 | 0.6-0.9 at shoulder | 1.0-1.5 | 25-80 | hair | white, black, brown, grey, piebald |
| pig (hog, sow, boar) | Kept for meat, rooting in woods and yards. | quad | M | 1×1 | 0.6-0.9 at shoulder | 1.2-1.8 | 100-300 | sparse bristles | pink, black, red-brown, spotted |
| horse (pony, warhorse) | Riding, pulling and war; ponies are the small ones. | quad | L | 1×2 | 1.4-1.75 at shoulder | 2.2-2.6 | 380-900 | coat, mane, tail | bay, chestnut, black, grey, dun, palomino, roan, piebald |
| donkey (ass) | A patient pack animal, small and hardy. | quad | M | 1×1 | 0.9-1.4 at shoulder | 1.5-2.0 | 180-260 | coat | grey, brown, black, white muzzle |
| mule | The strong, sure-footed offspring of a horse and a donkey. | quad | L | 1×2 | 1.3-1.6 at shoulder | 2.0-2.4 | 300-500 | coat | brown, bay, black, grey |
| camel | Desert beast of burden that goes long without water. | quad | L | 1×2 | 1.8-2.0 at shoulder (2.2-2.5 at hump) | 2.2-3.5 | 400-650 | coarse hair | tan, sandy, brown, white |
| llama (alpaca) | Mountain pack animal kept for wool; alpacas are smaller. | quad | M | 1×1 | 1.0-1.2 at shoulder (1.7-1.8 head) | 1.5-2.0 | 130-200 | wool | white, brown, black, piebald |
| water buffalo | Heavy ox of wet lands, used for ploughing paddies. | quad | L | 1×2 | 1.3-1.8 at shoulder | 2.4-3.0 | 500-1200 | sparse hair | dark grey, black |
| yak | Shaggy ox of the high mountains. | quad | L | 1×2 | 1.4-2.0 at shoulder | 2.5-3.3 | 300-1000 | long shaggy hair | black, brown, white, piebald |
| reindeer | Herded deer of the far north, pulling sledges. | quad | M | 1×1 | 0.9-1.4 at shoulder | 1.6-2.1 | 80-180 | thick fur | grey-brown, white |
| chicken (hen, rooster) | Kept for eggs and meat in every village. | bird | T | <1 | 0.3-0.6 | 0.4-0.7 | 1.5-4.5 | feathers | white, brown, black, red, speckled |
| duck | Kept for eggs and meat on ponds. | bird | T | <1 | 0.3-0.5 | 0.5-0.7 | 1-4 | feathers | white, mallard green and brown, black |
| goose | Kept for meat and down, and noisy guards. | bird | T | <1 | 0.6-0.9 | 0.7-1.0 | 3-9 | feathers | white, grey, brown |
| turkey | Large table bird. | bird | T | <1 | 0.7-1.0 | 1.0-1.2 | 5-15 | feathers | bronze, black, white, red wattle |
| pigeon (dove) | Kept for meat and to carry messages. | bird | T | <1 | 0.25-0.35 | 0.3 | 0.3-0.5 | feathers | grey, white, iridescent neck |
| rabbit | Kept for meat and fur. | quad | T | <1 | 0.2-0.3 | 0.3-0.5 | 1-6 | fur | brown, grey, white, black, spotted |
| dog (mastiff) | Companions, guards, herders and hunters. | quad | S | 1×1 | 0.25-0.85 at shoulder | 0.5-1.2 | 5-80 | fur | black, brown, tan, white, grey, brindle, patched |
| cat | Mousers in every barn and kitchen. | quad | T | <1 | 0.23-0.28 at shoulder | 0.45-0.55 | 3-6 | fur | black, white, ginger, grey, tabby, tortoiseshell |
| honeybee | Kept in hives for honey and wax. | swarm | T | <1 | 0.012-0.02 (swarm 1 cell) |  | 0.0001 | hair, chitin | gold and black bands |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cattle | neutral | day | herd | normal | low | keen | keen | wide field of view | walk, run, swim | animal | common |  |
| sheep | timid | day | flock | normal | low | keen | normal | near all-round view | walk, run | animal | common | 1 |
| goat | neutral | day | herd | normal | low | keen | keen |  | walk, run, climb | animal | common | 1 |
| pig | neutral | day | family | poor | low | keen | keen | roots by smell | walk, run, swim | animal | common | 1 |
| horse | timid | day | herd | normal | low | keen | keen | wide field of view | walk, run, swim | animal | common | 4 |
| donkey | neutral | day | herd | normal | low | keen | keen | long ears | walk, run | animal | common |  |
| mule | neutral | day | herd | normal | low | keen | keen |  | walk, run | animal | common | 1 |
| camel | neutral | day | herd | normal | low | normal | keen | closes nostrils against sand | walk, run | animal | common | 1 |
| llama | neutral | day | herd | normal | low | keen | normal | spits | walk, run | animal | common |  |
| water buffalo | neutral | day | herd | normal | low | normal | keen |  | walk, swim | animal | common |  |
| yak | neutral | day | herd | normal | low | normal | keen | resists cold | walk, climb | animal | common | 2 |
| reindeer | timid | any | herd | normal | dark | keen | keen | sees ultraviolet | walk, run, swim | animal | common |  |
| chicken | timid | day | flock | normal | none | normal | poor |  | walk, fly (short) | animal | common |  |
| duck | timid | day | flock | normal | low | normal | poor |  | walk, swim, fly | animal | common |  |
| goose | territorial | day | flock | keen | low | keen | poor | loud guard | walk, swim, fly | animal | common |  |
| turkey | neutral | day | flock | keen | none | normal | poor |  | walk, run, fly (short) | animal | common |  |
| pigeon | timid | day | flock | keen | none | normal | poor | finds its way home | walk, fly | animal | common |  |
| rabbit | timid | dawn_and_dusk | colony | normal | low | keen | keen | near all-round view | hop, run, burrow | animal | common |  |
| dog | friendly | day | pack | normal | low | keen | keen | tracks by scent | walk, run, swim | animal | common | 4 |
| cat | neutral | dawn_and_dusk | solitary | keen | dark | keen | keen | whiskers feel the way | walk, run, climb | animal | common | 4 |
| honeybee | neutral | day | colony | normal | none | poor | keen | feel vibration; sees ultraviolet | fly | animal | common | 2 |

## Animal

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| wolf | Grey hunters of forests and plains that work in packs. | quad | M | 1×1 | 0.6-0.85 at shoulder | 1.0-1.6 | 25-70 | fur | grey, white, black, brown, tan |
| fox | Small, clever hunter of woods and fields. | quad | T | <1 | 0.35-0.5 at shoulder | 0.45-0.9 | 3-14 | fur | red, grey, white (arctic), silver, black |
| jackal | Small wild dog of dry lands that scavenges. | quad | S | 1×1 | 0.4-0.5 at shoulder | 0.7-1.05 | 6-15 | fur | golden, tan, black-backed |
| coyote | Wild dog of the plains, cunning and adaptable. | quad | S | 1×1 | 0.5-0.66 at shoulder | 0.75-1.0 | 7-20 | fur | grey-brown, tan |
| hyena | Strong-jawed hunter and scavenger of the plains. | quad | M | 1×1 | 0.7-0.9 at shoulder | 0.95-1.6 | 40-86 | fur | tan with spots or stripes |
| wild dog (dingo) | Wild or feral dogs that run in packs. | quad | S | 1×1 | 0.5-0.75 at shoulder | 0.9-1.1 | 15-35 | fur | ginger, mottled black-tan-white |
| bear | Huge, strong forest animal that eats almost anything. | quad | L | 1×2 | 1.0-1.5 at shoulder (2-3 standing) | 1.5-2.8 | 90-600 | fur | black, brown, grey-brown, white (polar) |
| boar (wild pig) | Wild pig with tusks, dangerous when cornered. | quad | M | 1×1 | 0.55-1.1 at shoulder | 0.9-2.0 | 50-200 | bristly fur | dark grey, brown, black; striped young |
| deer (stag, hind) | Shy grazing animal of woods; males carry antlers. | quad | M | 1×1 | 0.8-1.2 at shoulder | 1.5-2.1 | 50-200 | short fur | red-brown, grey-brown, fawn spotted young |
| elk (moose) | Very large deer of northern forests. | quad | L | 1×2 | 0.75-1.5 at shoulder (moose to 2.1) | 2.4-3.1 | 200-700 | fur | brown, dark brown legs |
| bison (aurochs) | Massive wild ox of the plains. | quad | L | 1×2 | 1.5-2.0 at shoulder | 2.1-3.5 | 400-1270 | shaggy fur | dark brown, black |
| wild horse | Wild horses that live in herds. | quad | L | 1×2 | 1.2-1.5 at shoulder | 2.1-2.6 | 200-450 | coat, upright mane | dun, bay, with dark stripe |
| antelope (gazelle) | Swift grazers of the plains, from small gazelles to great elands. | quad | M | 1×1 | 0.6-1.8 at shoulder | 1.0-3.0 | 15-950 | short fur | tan, brown, white belly, striped |
| lion | Great maned cat of the plains that hunts in prides. | quad | L | 1×2 | 1.0-1.25 at shoulder | 1.6-2.5 + tail 0.9 | 120-250 | fur, mane (male) | tawny, gold; mane black to blond |
| tiger | Great striped cat of jungles and forests. | quad | L | 1×2 | 0.8-1.1 at shoulder | 1.5-2.9 + tail 1.0 | 65-300 | fur | orange with black stripes, white belly; white rare |
| leopard (panther, jaguar) | Spotted climbing cat; black ones are called panthers. | quad | M | 1×1 | 0.45-0.8 at shoulder | 0.9-1.9 + tail 0.8 | 20-90 | fur | gold with black rosettes; black ('panther') |
| lynx (wildcat) | Tufted-eared wildcat of northern forests. | quad | S | 1×1 | 0.5-0.7 at shoulder | 0.8-1.3 | 8-30 | fur | grey, tawny, spotted, black ear tufts |
| badger | Burrowing animal of woods, fierce in defence. | quad | T | <1 | 0.25-0.3 at shoulder | 0.6-0.9 | 7-17 | coarse fur | grey, black and white face stripes |
| weasel (stoat, ferret) | Small, fierce hunter; stoats turn white in winter. | quad | T | <1 | 0.05-0.1 | 0.15-0.45 | 0.03-3 | fur | brown above, white or cream below; white in winter (stoat) |
| otter | Playful swimmer of rivers and coasts. | quad | T | <1 | 0.3 | 0.6-1.0 + tail 0.4 | 5-15 | thick fur | dark brown, pale throat |
| beaver | Dam-builder of rivers. | quad | T | <1 | 0.3 | 0.75-0.9 + tail 0.3 | 11-30 | thick fur, flat tail | brown |
| squirrel | Small climber of trees that hoards nuts. | quad | T | <1 | 0.1 | 0.2-0.3 + tail 0.2 | 0.3-0.8 | fur | red, grey, brown, black |
| rat | Common pest of towns and dungeons. | quad | T | <1 | 0.08-0.1 | 0.2-0.25 + tail 0.2 | 0.2-0.5 | fur | brown, grey, black, white |
| mouse | Tiny pest of houses and fields. | quad | T | <1 | 0.03 | 0.07-0.1 + tail 0.08 | 0.02-0.04 | fur | brown, grey, white |
| mole | Burrower that lives almost blind under the ground. | quad | T | <1 | 0.05 | 0.1-0.2 | 0.07-0.13 | velvet fur | black, dark grey |
| hedgehog | Small spiny animal that curls up in a ball. | quad | T | <1 | 0.1 | 0.15-0.3 | 0.4-1.2 | spines | brown, cream tips |
| hare | Fast, long-eared runner of open fields. | quad | T | <1 | 0.3 | 0.5-0.7 | 2.5-5 | fur | brown, grey; white in winter (mountain) |
| bat | Flying animal that hunts at night by echoes. | biped-winged | T | <1 | 0.05-0.4 | wingspan 0.15-1.7 | 0.005-1.6 | fur, leather wings | brown, black, ginger, grey |
| monkey | Clever, chattering climber of warm forests. | biped | S | 1×1 | 0.3-0.8 | 0.3-0.9 + tail | 1-35 | fur | brown, grey, black, golden, white face |
| ape (gorilla) | Large, strong, clever animal like the gorilla. | biped | M | 1×1 | 1.2-1.8 standing | - | 40-200 | fur | black, grey, red-orange (orangutan) |
| baboon | Big monkey of rocky plains with large teeth. | quad | S | 1×1 | 0.5-0.8 at shoulder | 0.5-1.1 + tail | 14-40 | fur | olive, yellow-brown, grey; red rump |
| elephant | The largest land animal, with a trunk and tusks. | quad | G | 2×2 | 2.5-4.0 at shoulder | 5.5-7.5 | 2500-7000 | thick skin | grey, brown (dust) |
| rhinoceros | Huge armoured grazer with horns on its nose. | quad | L | 1×2 | 1.4-1.9 at shoulder | 3.0-4.0 | 800-3600 | thick skin | grey, grey-brown |
| hippopotamus | Huge river animal, more dangerous than it looks. | quad | H | 2×2 | 1.3-1.6 at shoulder | 3.3-5.0 | 1300-3200 | thick skin | grey, pink-brown |
| crocodile (alligator) | Armoured river hunter that ambushes from the water. | quad | L | 1×2 | 0.4-0.6 | 2.5-6.0 | 100-1000 | scales, plates | olive, grey-green, brown |
| lizard | Small scaly animal of warm places. | quad | T | <1 | 0.02-0.1 | 0.05-0.5 | 0.01-1 | scales | green, brown, grey, banded |
| giant lizard (monitor) | Large monitor lizards, up to the size of a man and larger. | quad | L | 1×2 | 0.4-0.6 | 2.0-3.1 | 30-150 | scales | grey-brown, speckled |
| snake (viper, cobra) | Legless reptile, many of them venomous. | serpent | S | 1×1 | 0.05 | 0.3-2.5 | 0.05-10 | scales | green, brown, black, grey, banded, diamond pattern |
| constrictor (python, anaconda) | Large snake that squeezes its prey to death. | serpent | L | 1×1 | 0.2-0.3 thick | 3-7 | 50-150 | scales | green, brown, olive, patterned |
| turtle (tortoise) | Shelled reptile of land and water. | quad | S | 1×1 | 0.1-0.6 | 0.1-1.8 | 0.5-500 | shell | green, brown, olive, patterned |
| frog | Small jumping amphibian of ponds. | quad | T | <1 | 0.02-0.1 | 0.02-0.3 | 0.01-3 | smooth wet skin | green, brown, bright warning colours (poison) |
| toad | Warty, slow cousin of the frog with poisonous skin. | quad | T | <1 | 0.03-0.1 | 0.05-0.25 | 0.05-2 | warty dry skin | brown, olive, grey |
| salamander (newt) | Small, smooth amphibian of damp places. | quad | T | <1 | 0.02 | 0.1-1.8 (giant) | 0.01-50 | smooth wet skin | black and yellow, orange, brown |
| eagle | Great bird of prey with keen eyes. | bird | T | <1 | 0.7-1.0 | wingspan 1.8-2.3 | 3-7 | feathers | brown, golden nape, white head (bald) |
| hawk (falcon) | Fast bird of prey used in hunting. | bird | T | <1 | 0.4-0.6 | wingspan 0.8-1.4 | 0.5-1.5 | feathers | brown, grey, barred |
| owl | Silent night hunter with great eyes. | bird | T | <1 | 0.2-0.7 | wingspan 0.5-2.0 | 0.1-4 | soft feathers | brown, grey, white, barred |
| crow (raven) | Clever black bird; ravens are the largest. | bird | T | <1 | 0.4-0.7 | wingspan 0.9-1.5 | 0.4-1.6 | feathers | black, glossy; grey hooded |
| vulture | Bird that feeds on the dead. | bird | S | 1×1 | 0.6-1.2 | wingspan 1.7-3.0 | 4-15 | feathers, bare head | black, brown, white |
| songbird (sparrow, robin) | Small singing birds of every kind. | bird | T | <1 | 0.1-0.25 | wingspan 0.2-0.4 | 0.01-0.1 | feathers | any |
| seabird (gull) | Gulls and other birds of the shore. | bird | T | <1 | 0.3-0.7 | wingspan 0.8-1.7 | 0.5-2 | feathers | white, grey, black |
| fish (quipper) | Fish of rivers, lakes and seas. | fish | T | <1 | 0.05-2 | 0.05-2 | 0.01-100 | scales | silver, green, brown, bright reef colours |
| eel | Long, slippery fish; some give electric shocks. | fish | S | 1×1 | 0.1 | 0.5-2.5 | 1-25 | slimy skin | brown, olive, grey, moray patterns |
| shark | Hunter of the seas with rows of teeth. | fish | L | 1×2 | 0.5-1 | 1.5-6 | 50-2000 | rough skin | grey, blue-grey, white belly |
| octopus | Eight-armed, clever sea creature that changes colour. | cephalopod | S | 1×1 | - | 0.3-4.5 with arms | 1-50 | soft skin | changes colour: red, brown, white, mottled |
| squid | Ten-armed sea creature; giant ones grow very large. | cephalopod | M | 1×1 | - | 0.5-13 with arms | 1-500 | soft skin | red, white, translucent |
| crab | Shelled, clawed scuttler of shores. | hexapod | T | <1 | 0.05-0.3 | 0.05-0.4 (4 m legspan for giant spider crab) | 0.1-20 | shell | red, brown, green, blue |
| lobster | Large clawed shellfish of the seabed. | hexapod | T | <1 | 0.1 | 0.25-0.6 | 0.5-20 | shell | dark blue-green, red-brown |
| jellyfish | Drifting, stinging creature of the sea. | amorphous | T | <1 | - | 0.02-2 bell, tentacles to 30 | 0.1-200 | jelly | clear, pink, blue, glowing |
| whale | The great animals of the sea. | fish | G+ 2x8 | 2×8 | - | 4-30 | 1000-150000 | skin | grey, black, white, blue |
| dolphin | Clever, playful sea animal. | fish | M | 1×2 | - | 1.5-4 | 50-650 | skin | grey, white belly |
| seal | Sleek swimmer of cold coasts. | fish | M | 1×2 | - | 1.2-5 (elephant seal) | 50-4000 | fur, blubber | grey, brown, spotted |
| walrus | Large tusked seal of the north. | fish | L | 1×2 | - | 2.2-3.6 | 600-1700 | thick skin | brown, pink |
| spider | Eight-legged hunter that spins webs. | octopod | T | <1 | 0.005-0.1 | legspan 0.01-0.3 | 0.001-0.2 | chitin, hair | black, brown, grey, red and black, yellow banded |
| scorpion | Eight-legged hunter of deserts with a venomous sting. | octopod | T | <1 | 0.02 | 0.05-0.2 | 0.005-0.05 | chitin | black, brown, yellow |
| centipede | Many-legged venomous hunter of dark places. | myriapod | T | <1 | 0.01 | 0.03-0.3 | 0.001-0.1 | chitin | red-brown, orange, black, banded |
| beetle | Hard-shelled insect of every kind. | hexapod | T | <1 | 0.005-0.05 | 0.005-0.15 | 0.001-0.1 | chitin shell | black, brown, iridescent green, red |
| ant | Tiny insect living in great colonies. | hexapod | T | <1 | 0.001-0.01 (colony) | 0.002-0.03 | tiny | chitin | black, red, brown |
| wasp (hornet, swarm of insects) | Stinging insect that lives in paper nests. | hexapod | T | <1 | 0.01-0.05 | wingspan 0.06 | tiny | chitin, hair | yellow and black, red-brown |
| fly | Common buzzing insect of filth. | hexapod | T | <1 | 0.005-0.02 | 0.005-0.03 | tiny | chitin | black, grey, metallic green |
| mosquito | Biting insect of swamps that spreads fever. | hexapod | T | <1 | 0.003-0.01 | 0.003-0.015 | tiny | chitin | brown, grey, striped |
| moth (butterfly) | Night-flying insect drawn to light. | hexapod | T | <1 | 0.01-0.1 | wingspan 0.02-0.3 | tiny | scales on wings | brown, grey, white, patterned; butterflies bright |
| mantis | Ambush hunter with grasping forelegs. | hexapod | T | <1 | 0.03-0.15 | 0.03-0.15 | tiny | chitin | green, brown, pink (orchid) |
| cockroach | Hardy scavenging insect. | hexapod | T | <1 | 0.01-0.08 | 0.01-0.08 | tiny | chitin | brown, black |
| termite | Wood-eating insect of great mounds. | hexapod | T | <1 | 0.004-0.02 | 0.004-0.02 | tiny | soft body | pale cream, brown |
| tick (louse, flea) | Small blood-drinker that waits on grass. | hexapod | T | <1 | 0.003-0.01 | 0.003-0.01 | tiny | shell | red-brown, black |
| worm (earthworm) | Soft, legless burrower of the soil. | serpent | T | <1 | 0.01 | 0.1-0.3 (giant: 2-3 m; purple worm 25 m) | 0.01-1 | soft skin | pink, red-brown, grey |
| slug (snail) | Slimy creeper of gardens; snails carry shells. | amorphous | T | <1 | 0.01 | 0.02-0.2 | 0.01-0.1 | slime | brown, grey, orange, black, yellow |
| leech | Bloodsucking worm of ponds. | serpent | T | <1 | 0.005 | 0.01-0.2 | tiny | slimy skin | brown, black, olive, striped |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| wolf | territorial | dawn_and_dusk | pack | keen | dark | keen | keen |  | walk, run, swim | animal | common | 5 |
| fox | timid | night | solitary | keen | dark | keen | keen | hears prey under snow | walk, run, burrow | animal | common | 1 |
| jackal | timid | night | pairs | normal | dark | keen | keen |  | walk, run | animal | common | 5 |
| coyote | timid | night | pairs | normal | dark | keen | keen |  | walk, run, swim | animal | common | 1 |
| hyena | hostile | night | pack | normal | dark | keen | keen | cracks bones | walk, run | animal | common | 1 |
| wild dog | neutral | day | pack | normal | low | keen | keen |  | walk, run | animal | common | 1 |
| bear | territorial | any | solitary | normal | low | keen | keen | smell carries for miles | walk, run, climb, swim | animal | common | 4 |
| boar | territorial | night | family | poor | low | keen | keen |  | walk, run, swim | animal | common | 1 |
| deer | timid | dawn_and_dusk | herd | normal | low | keen | keen | wide field of view | walk, run, swim | animal | common | 1 |
| elk | timid | dawn_and_dusk | herd | normal | low | keen | keen |  | walk, run, swim | animal | common | 1 |
| bison | neutral | day | herd | poor | low | keen | keen |  | walk, run | animal | common |  |
| wild horse | timid | day | herd | normal | low | keen | keen |  | walk, run | animal | common |  |
| antelope | timid | day | herd | keen | low | keen | keen |  | walk, run | animal | common |  |
| lion | hostile | night | family | keen | dark | keen | normal |  | walk, run | animal | uncommon | 2 |
| tiger | hostile | night | solitary | keen | dark | keen | normal |  | walk, run, swim | animal | uncommon | 4 |
| leopard | hostile | night | solitary | keen | dark | keen | normal |  | walk, run, climb | animal | uncommon | 3 |
| lynx | timid | night | solitary | keen | dark | keen | normal |  | walk, run, climb | animal | common | 1 |
| badger | territorial | night | family | poor | low | normal | keen |  | walk, burrow | animal | common | 1 |
| weasel | hostile | any | solitary | normal | low | keen | keen |  | walk, run, climb, swim | animal | common | 1 |
| otter | timid | night | family | normal | low | keen | keen | whiskers feel in water | walk, swim | animal | common |  |
| beaver | timid | night | family | poor | low | keen | keen |  | walk, swim | animal | common |  |
| squirrel | timid | day | solitary | keen | none | keen | keen |  | climb, run, glide (flying squirrels) | animal | common |  |
| rat | neutral | night | colony | poor | dark | keen | keen | whiskers | walk, climb, swim, burrow | animal | common | 6 |
| mouse | timid | night | colony | poor | dark | keen | keen | whiskers | walk, climb, burrow | animal | common | 1 |
| mole | timid | any | solitary | poor | none | keen | keen | feel vibration, touch | burrow | animal | common | 1 |
| hedgehog | timid | night | solitary | poor | low | keen | keen |  | walk, curl up | animal | common |  |
| hare | timid | dawn_and_dusk | solitary | normal | low | keen | keen |  | run, hop | animal | common |  |
| bat | timid | night | colony | poor | dark | keen | normal | echolocation | fly, climb | animal | common | 6 |
| monkey | neutral | day | family | keen | none | normal | normal |  | walk, climb | cunning | common | 3 |
| ape | neutral | day | family | keen | none | normal | normal |  | walk, climb | cunning | uncommon | 2 |
| baboon | territorial | day | pack | keen | none | normal | normal |  | walk, run, climb | cunning | common | 1 |
| elephant | neutral | any | herd | poor | low | keen | keen | feels rumbles through feet | walk, swim | cunning | uncommon | 2 |
| rhinoceros | territorial | any | solitary | poor | low | keen | keen |  | walk, run | animal | uncommon | 1 |
| hippopotamus | hostile | night | herd | normal | low | keen | keen |  | walk, run, swim (walks under water) | animal | uncommon |  |
| crocodile | hostile | any | solitary | keen | dark | keen | normal | feels ripples | walk, swim | animal | common | 3 |
| lizard | timid | day | solitary | keen | none | normal | normal | tastes the air | walk, run, climb | animal | common | 5 |
| giant lizard | hostile | day | solitary | keen | none | normal | keen | tastes the air (forked tongue) | walk, run, swim, climb | animal | uncommon |  |
| snake | territorial | any | solitary | poor | dark (heat pits) | poor | keen | senses heat, feels vibration | slither, swim, climb | animal | common | 5 |
| constrictor | hostile | night | solitary | poor | dark (heat pits) | poor | keen | senses heat | slither, swim, climb | animal | uncommon | 2 |
| turtle | timid | day | solitary | normal | low | poor | normal |  | walk (slow), swim | animal | common | 3 |
| frog | timid | night | colony | keen | low | normal | poor |  | hop, swim | animal | common | 4 |
| toad | timid | night | solitary | normal | low | normal | poor | poison skin | walk, hop, swim | animal | common | 4 |
| salamander (newt) | timid | night | solitary | poor | low | poor | normal |  | walk, swim | animal | common |  |
| eagle | territorial | day | pairs | keen | none | normal | poor | sees far | walk, fly | animal | uncommon | 1 |
| hawk | territorial | day | pairs | keen | none | normal | poor | falcons fastest diving | fly | animal | common | 2 |
| owl | timid | night | solitary | keen | dark | keen | poor | silent flight, finds prey by sound | fly | animal | common | 1 |
| crow | neutral | day | flock | keen | none | normal | poor |  | walk, fly | cunning | common | 4 |
| vulture | timid | day | flock | keen | none | normal | keen (new world) | finds carrion | walk, fly | animal | common | 1 |
| songbird | timid | day | flock | keen | none | keen | poor |  | fly, hop | animal | common | 2 |
| seabird | neutral | day | colony | keen | none | normal | normal |  | fly, swim | animal | common |  |
| fish | timid | any | swarm | normal | low | poor | normal | feels currents (lateral line) | swim | animal | common | 3 |
| eel | timid | night | solitary | poor | low | poor | keen | electric eels shock | swim, slither (short) | animal | common | 3 |
| shark | hostile | any | solitary | normal | low | keen | keen | smells blood, senses electricity | swim | animal | uncommon | 2 |
| octopus | timid | night | solitary | keen | low | poor | normal | tastes by touch | swim, crawl | cunning | common | 2 |
| squid | neutral | night | swarm | keen | dark | poor | poor |  | swim | animal | common |  |
| crab | neutral | any | colony | normal | low | poor | normal |  | walk sideways, swim | animal | common | 2 |
| lobster | neutral | night | solitary | poor | low | poor | keen |  | walk, swim | animal | common |  |
| jellyfish | neutral | any | swarm | none | none | none | none | stings on touch | float in water | mindless | common | 2 |
| whale | neutral | any | family | poor | low | keen | poor | echolocation (toothed whales) | swim | cunning | uncommon | 1 |
| dolphin | friendly | any | family | normal | low | keen | poor | echolocation | swim | cunning | common |  |
| seal | timid | any | colony | normal | low | normal | normal | whiskers feel currents | swim, crawl | animal | common |  |
| walrus | territorial | any | herd | poor | low | normal | normal | whiskers | swim, crawl | animal | uncommon |  |
| spider | timid | night | solitary | normal | dark | poor | poor | feels web and vibration | walk, climb | animal | common | 6 |
| scorpion | hostile | night | solitary | poor | dark | poor | poor | feels vibration | walk, burrow | animal | common | 5 |
| centipede | hostile | night | solitary | poor | dark | poor | normal | feelers | walk, climb, burrow | animal | common | 6 |
| beetle | neutral | any | solitary | normal | low | poor | keen | feelers | walk, fly, burrow | animal | common | 5 |
| ant | territorial | any | colony | poor | low | poor | keen | scent trails | walk, climb, burrow | animal | common | 4 |
| wasp | hostile | day | colony | normal | none | poor | keen |  | fly | animal | common | 2 |
| fly | neutral | day | swarm | keen | none | poor | keen |  | fly, walk | animal | common | 1 |
| mosquito | hostile | dawn_and_dusk | swarm | poor | low | normal | keen | senses breath and heat | fly | animal | common | 1 |
| moth | timid | night | solitary | normal | dark | normal | keen | drawn to light | fly | animal | common | 2 |
| mantis | hostile | day | solitary | keen | none | poor | normal |  | walk, fly | animal | common |  |
| cockroach | timid | night | colony | poor | dark | poor | keen | feelers | run, fly | animal | common | 1 |
| termite | timid | any | colony | none | none | poor | keen | scent | walk, burrow | animal | common | 1 |
| tick | hostile | any | solitary | none | none | poor | keen | senses breath and heat | climb | animal | common | 1 |
| worm | timid | night | colony | none | none | none | normal | feels vibration | burrow | mindless | common | 6 |
| slug | timid | night | solitary | poor | low | none | keen |  | creep | mindless | common | 1 |
| leech | hostile | any | colony | poor | low | poor | keen | senses heat and ripples | swim | mindless | common | 1 |

## Extinct

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| axe beak (terror bird) | A tall flightless bird with a heavy hooked beak. | bird | L | 1×2 | 2.0-3.0 | 2-3 | 100-130 | feathers | unknown: brown guessed |
| mammoth (mastodon) | Shaggy elephant of the ice age. | quad | G | 2×2 | 3.0-4.0 at shoulder | 5-6 | 6000-8000 | shaggy fur | brown, red-brown |
| sabre-toothed cat | Great cat of the ice age with long sabre teeth. | quad | L | 1×1 | 1.0-1.2 at shoulder | 1.8-2.2 | 160-400 | fur | tawny, maybe spotted (unknown) |
| dire wolf | Large wolf of the ice age. | quad | M | 1×1 | 0.8-0.95 at shoulder | 1.5-1.8 | 50-80 | fur | reddish brown (DNA, 2021) |
| cave bear | Huge bear of the ice age. | quad | L | 1×2 | 1.2-1.5 at shoulder (3 standing) | 2.5-3.5 | 225-600 | fur | brown |
| raptor (dinosaur) | Fast, feathered, sickle-clawed hunting dinosaur. | biped-dg | M | 1×1 | 0.5-1.8 at hip | 1.5-6 | 15-500 | feathers | unknown: likely brown, banded |
| tyrannosaur (tyrannosaurus rex) | Huge two-legged hunting dinosaur. | biped-dg | G+ 2x6 | 2×6 | 3.6-4 at hip | 12-13 | 6000-9000 | scales, maybe feathers | unknown: brown, grey guessed |
| triceratops | Three-horned, frilled grazing dinosaur. | quad | G | 2×2 | 2.5-3 at hip | 8-9 | 6000-12000 | scales, frill | unknown |
| plesiosaur (plesiosaurus) | Long-necked swimming reptile of ancient seas. | fish | G | 2×2 | - | 4-15 | 500-5000 | skin | unknown |
| giant sloth | Huge ground sloth of the ice age. | quad | H | 2×2 | 2-3 at shoulder (4-6 standing) | 6 | 3000-4000 | shaggy fur | brown |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| axe beak | hostile | day | solitary | keen | none | normal | normal |  | run | animal | rare |  |
| mammoth | neutral | day | herd | poor | low | keen | keen |  | walk | cunning | rare | 2 |
| sabre-toothed cat | hostile | night | family | keen | dark | keen | normal |  | walk, run | animal | rare |  |
| dire wolf | territorial | dawn_and_dusk | pack | keen | dark | keen | keen |  | walk, run | animal | uncommon |  |
| cave bear | territorial | any | solitary | normal | low | keen | keen |  | walk, climb | animal | uncommon |  |
| raptor (dinosaur) | hostile | day | pack | keen | low | keen | keen |  | run | animal | rare | 1 |
| tyrannosaur | hostile | day | solitary | keen | low | keen | keen |  | walk, run | animal | very rare | 1 |
| triceratops | territorial | day | herd | normal | low | normal | normal |  | walk | animal | rare | 1 |
| plesiosaur | hostile | any | solitary | keen | low | normal | normal |  | swim | animal | rare | 1 |
| giant sloth | neutral | day | solitary | poor | low | normal | keen |  | walk | animal | rare |  |

## Folk

| Name | Description | Body | Size | Cells | Height | Length | Weight | Covering | Colours |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| kappa | Small, beaked river creatures of Japan with a shell and a water-filled dish on their heads. | biped | S | 1×1 | 0.90-1.20 |  | 20-40 | scaly skin, shell | green, blue-green, yellow-green |

| Name | Temper | Active | Social | Sight | Dark | Hearing | Smell | Other senses | Moves | Mind | Rarity | Games |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| kappa | hostile | any | solitary | normal | none | normal | normal | loses strength if head water spills | walk, swim | cunning | uncommon |  |

## Biped proportions

Beside a person (1.00): heads tall as artists measure (this table's person is 7.5; the code's frame is 8), breadth of shoulders, thickness of limbs, length of arms, and hip height over total height. They become each kind's `shared::body::Proportions`.

| Kind | Heads | Build | Shoulders | Limbs | Arms | Legs | Posture | Feet | Hands | Head | Tail | Wings | Armour fit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| human | 7.5 | average | 1.00 | 1.00 | 1.00 | 0.51 | upright | plantigrade | 5 fingers | round, small ears | - | - | all |
| halfling | 5 | slight, round | 0.90 | 0.90 | 0.95 | 0.47 | upright | plantigrade, large hairy | 5 fingers | round, child-like face, slightly pointed ears | - | - | all, but no shoes by custom |
| dwarf | 5 | stocky, barrel chest | 1.25 | 1.30 | 0.95 | 0.44 | upright | plantigrade, broad | 5 fingers, thick | large, big nose, full beard | - | - | all; beard under bevor |
| elf | 8 | slender | 0.90 | 0.80 | 1.03 | 0.54 | upright | plantigrade, narrow | 5 fingers, long | narrow, long pointed ears | - | - | all; helmets need ear room |
| dark elf | 8 | slender | 0.90 | 0.80 | 1.03 | 0.54 | upright | plantigrade, narrow | 5 fingers, long | narrow, long pointed ears | - | - | all; helmets need ear room |
| half-elf | 7.5 | average | 0.97 | 0.92 | 1.00 | 0.52 | upright | plantigrade | 5 fingers | slightly pointed ears | - | - | all |
| half-orc | 7.5 | muscular | 1.10 | 1.15 | 1.03 | 0.51 | upright | plantigrade | 5 fingers | heavy jaw, small tusks | - | - | all |
| gnome | 4.5 | slight | 0.85 | 0.85 | 0.95 | 0.45 | upright | plantigrade | 5 fingers | big head, big nose and ears | - | - | all; tall hats |
| goblin | 5 | wiry | 0.85 | 0.80 | 1.10 | 0.46 | stooped | plantigrade, long toes | 5 fingers, long | big head, long pointed ears, wide mouth | - | - | all; helmets need ear room |
| hobgoblin | 7 | muscular | 1.05 | 1.05 | 1.00 | 0.51 | upright | plantigrade | 5 fingers | flat face, pointed ears | - | - | all |
| orc | 7 | muscular, heavy | 1.15 | 1.20 | 1.08 | 0.49 | slightly stooped | plantigrade | 5 fingers | heavy brow, jaw, tusks | - | - | all; face plates need tusk room |
| bugbear | 7 | heavy, hairy | 1.15 | 1.20 | 1.12 | 0.49 | stooped | plantigrade | 5 fingers | bear-like snout, round ears | - | - | all; open helmets |
| kobold | 4 | slight | 0.85 | 0.80 | 0.95 | 0.48 | leaning forward | digitigrade, clawed | 5 fingers, clawed (3 or 4 in some art) | reptile snout, small horns | yes, long | - | no closed helmets; no boots or greaves made for plantigrade; faulds over tail |
| gnoll | 7 | lean, muscular | 1.10 | 1.05 | 1.10 | 0.53 | leaning forward | digitigrade, clawed | 4 fingers, clawed | hyena snout, round ears, mane | short | - | no closed helmets; no human boots |
| ogre | 6 | fat, heavy | 1.40 | 1.50 | 1.05 | 0.45 | upright, belly | plantigrade, broad | 5 fingers, thick | small head for body, heavy jaw | - | - | all, made larger |
| troll | 7 | gaunt, long-limbed | 1.15 | 1.00 | 1.25 | 0.50 | stooped | plantigrade, long | 4 fingers, long claws | long nose, wide mouth, tusks | - | - | open helmets; long sleeves |
| giant | 8 | heavy | 1.20 | 1.25 | 1.00 | 0.51 | upright | plantigrade | 5 fingers | as human | - | - | all, made larger (already GIANT frame) |
| cyclops | 7.5 | heavy | 1.20 | 1.25 | 1.00 | 0.51 | upright | plantigrade | 5 fingers | one central eye | - | - | helmets with one eye slot |
| ettin | 7 | heavy, broad | 1.50 | 1.30 | 1.00 | 0.48 | upright | plantigrade | 5 fingers | two heads on one neck base | - | - | two helmets; wide neck pieces only |
| lizardfolk | 7 | lean | 1.00 | 0.95 | 1.00 | 0.50 | leaning forward | digitigrade, clawed | 4 or 5 fingers, clawed | snout, frills or crest | yes, long | - | no closed helmets, no plantigrade boots |
| ratfolk | 6 | slight | 0.85 | 0.85 | 0.98 | 0.48 | hunched | digitigrade | 4 fingers | rat snout, round ears, whiskers | yes, long, bare | - | no closed helmets |
| catfolk | 7.5 | lithe | 0.95 | 0.90 | 1.02 | 0.53 | upright | digitigrade, padded | 5 fingers, retractable claws | cat face, tall ears | yes, long | - | helmets need ear room; no plantigrade boots |
| dogfolk | 7 | lean | 1.00 | 1.00 | 1.00 | 0.52 | upright | digitigrade | 5 fingers | dog head, ears up or hanging | yes | - | no closed helmets |
| birdfolk | 7 | light | 0.95 | 0.80 | wings | 0.50 | upright | bird feet, talons | wing tips (some have hands on the wing) | beak, crest | tail feathers | arms are wings | no sleeves, gauntlets or arm armour; torso armour must leave wings free |
| frogfolk | 5 | squat | 1.05 | 0.95 | 1.00 | 0.42 | crouched | webbed, long toes | 4 fingers, webbed | wide frog head, no neck | - | - | no neck armour; no boots |
| troglodyte | 7 | lean | 1.00 | 1.00 | 1.05 | 0.50 | slightly stooped | plantigrade | 5 fingers, clawed | lizard-like head, crest | short | - | open helmets |
| satyr | 7 | lean | 1.00 | 0.95 | 1.00 | 0.52 | upright | digitigrade, hooves | 5 fingers | horns, goat ears, beard | short goat tail | - | no boots or sabatons; open helmets with horn room |
| minotaur | 7 | massive | 1.35 | 1.35 | 1.05 | 0.50 | upright | plantigrade or hooves | 5 fingers | bull head and horns, thick neck | cow tail | - | no closed helmets; horned helmets only; gorgets made larger |
| harpy | 7 | gaunt | 0.90 | 0.75 | wings | 0.48 | crouched | bird talons | wing tips | human, wild hair | tail feathers | arms are wings | as birdfolk |
| tengu | 7 | average | 1.00 | 0.95 | 1.00 | 0.51 | upright | bird talons or human | 5 fingers | crow beak or long nose | tail feathers | back wings | torso armour must leave back wings free |
| oni | 7 | massive | 1.30 | 1.30 | 1.05 | 0.49 | upright | plantigrade | 5 fingers (3 in some art) | one or two horns, tusks, wild hair | - | - | helmets with horn room |
| kappa | 4.5 | squat | 1.00 | 0.90 | 1.00 | 0.45 | crouched | webbed | webbed | beak, dish on head | - | shell on the back | no torso armour over the shell; no helmet (dish) |
| rakshasa | 7.5 | muscular | 1.05 | 1.05 | 1.00 | 0.52 | upright | plantigrade | 5 fingers, palms backwards | tiger head | tiger tail | - | open helmets |
| yeti | 7 | massive, shaggy | 1.35 | 1.40 | 1.15 | 0.47 | stooped | plantigrade, huge | 5 fingers | ape-like | - | - | none: shaggy animal, won't wear armour |
| sasquatch | 7 | massive, shaggy | 1.30 | 1.35 | 1.15 | 0.48 | stooped | plantigrade, huge | 5 fingers | ape-like, crested | - | - | none |
| neanderthal | 7 | stocky | 1.15 | 1.20 | 1.00 | 0.49 | upright | plantigrade | 5 fingers | heavy brow, no chin | - | - | all |
| gargoyle | 7 | muscular | 1.10 | 1.10 | 1.05 | 0.50 | crouched | digitigrade, clawed | 4 fingers, clawed | horns, fangs | yes | back wings | none: stone |
| fairy | 7 | slight | 0.85 | 0.70 | 1.00 | 0.53 | upright | plantigrade | 5 fingers | pointed ears | - | insect wings | torso armour must leave wings free |
| brownie | 4.5 | slight | 0.85 | 0.85 | 1.00 | 0.46 | upright | plantigrade | 5 fingers | shaggy, flat nose | - | - | all, rarely |
| leprechaun | 4.5 | slight | 0.85 | 0.85 | 1.00 | 0.46 | upright | plantigrade | 5 fingers | beard | - | - | all |
| redcap | 5 | wiry | 0.90 | 0.85 | 1.05 | 0.47 | stooped | iron boots | 5 fingers, talons | old, long teeth | - | - | all |
| dryad | 8 | slender | 0.90 | 0.80 | 1.03 | 0.54 | upright | plantigrade, root-like | 5 fingers, twig-like | leaf hair | - | - | none |
| nymph | 7.5 | slender | 0.92 | 0.85 | 1.00 | 0.53 | upright | plantigrade | 5 fingers | as human | - | - | all, rarely |
| hag | 7 | gaunt, bent | 0.95 | 0.80 | 1.05 | 0.49 | bent | plantigrade | 5 fingers, long nails | long nose, warts | - | - | all |
| angel | 8 | ideal | 1.05 | 1.00 | 1.00 | 0.53 | upright | plantigrade | 5 fingers | human | - | back wings | all, if armour leaves wings free |
| demon | 7 | massive | 1.30 | 1.25 | 1.08 | 0.50 | upright | hooves or clawed digitigrade | 4 or 5 fingers, clawed | horns, fangs | yes | some | none (natural armour); weapons yes |
| devil | 7.5 | lean, muscular | 1.10 | 1.05 | 1.02 | 0.52 | upright | hooves or digitigrade | 5 fingers, clawed | horns | yes, barbed | some | some wear armour: open helmets, tail slots |
| imp | 4 | thin | 0.85 | 0.70 | 1.05 | 0.48 | hunched | digitigrade, clawed | 4 fingers, clawed | horns, big ears | barbed | back wings | none |
| ghoul | 7.5 | gaunt | 0.95 | 0.75 | 1.10 | 0.51 | stooped | plantigrade, long nails | 5 fingers, long nails | sunken, long tongue | - | - | all (rags usually) |
| golem | 6 | massive, blocky | 1.40 | 1.50 | 1.10 | 0.47 | upright | plantigrade, blocky | 4 or 5 thick fingers | blocky, no features | - | - | none |
| owlbear | 6 | massive | 1.40 | 1.50 | 1.05 | 0.45 | upright or on all fours | clawed | clawed paws | owl head, beak | stub | - | none |
| gorgon (snake-haired) | 7.5 | average | 1.00 | 0.95 | 1.00 | 0.52 | upright | plantigrade | 5 fingers | snakes for hair | - | - | no helmets |
| grimlock | 7.5 | muscular | 1.05 | 1.05 | 1.00 | 0.51 | upright | plantigrade | 5 fingers | eyeless | - | - | all |
| sahuagin | 7 | muscular | 1.05 | 1.05 | 1.05 | 0.50 | upright | webbed, clawed | webbed, clawed | fish head, fins | yes, finned | - | open helmets; fin slots |
| wild man | 7 | hairy | 1.05 | 1.05 | 1.05 | 0.51 | upright | plantigrade | 5 fingers | shaggy | - | - | all, rarely |
| chupacabra | 5 | thin | 0.85 | 0.70 | 1.05 | 0.50 | hunched | digitigrade | 3 fingers, clawed | spines down the back | - | - | none |
| garuda | 7.5 | muscular | 1.10 | 1.05 | 1.00 | 0.52 | upright | bird talons | 5 fingers | eagle head | tail feathers | back wings | as tengu |
| tikbalang | 6 | gangly | 0.90 | 0.80 | 1.30 | 0.58 | hunched | hooves | 5 fingers, long | horse head | - | - | no helmets |

## Story only

| Name | Description | Size |
| --- | --- | --- |
| leviathan | The great sea monster of the deep, too vast to fight. | 50-100 |
| behemoth | The great beast of the land, too vast to fight. | 10-15 |
| jormungandr | The world serpent, so big it circles the world. | world-girdling |
| charybdis | A whirlpool monster that swallows the sea three times a day. | whirlpool |
| typhon | The father of monsters, with a hundred snake heads. | 20-50 |

## Left out

| Name | Why | Instead |
| --- | --- | --- |
| beholder | D&D's own, not in the SRD | floating eye |
| mind flayer | D&D's own, not in the SRD | tentacle horror |
| displacer beast | D&D's own, not in the SRD | a shadow cat of our own |
| umber hulk | D&D's own, not in the SRD | a burrowing brute of our own |
| carrion crawler | D&D's own, not in the SRD | giant centipede |
| kuo-toa | D&D's own, not in the SRD | fish-folk |
| yuan-ti | D&D's own, not in the SRD | naga, lamia |
| slaad | D&D's own, not in the SRD | frogfolk |
| githyanki | D&D's own, not in the SRD | dark elf or a new astral people |
| draconian | one game's or company's | lizardfolk |
| balrog | Tolkien's | demon (fire) |
| hobbit | Tolkien's | halfling |
| ent | Tolkien's | treant |
| warg | Tolkien's | dire wolf, worg |
| mumak | Tolkien's | elephant |
| quylthulg | one game's or company's | a summoner of our own |
| yeek | one game's or company's | kobold |
