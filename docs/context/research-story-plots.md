# Research: story plots for quest text

Research for the Flavour text plan (`docs/context/story-text.md`), done 2026-11-12 in chat `local_c00633de` for parent `local_78a84702`. It is the catalogue `shared::story` (Flavour text PR 3) is built from: why givers ask, the backstory beats and complications, what each beat needs, which quest goals and pois it fits, and how beats make up to three paragraphs. Summaries only: no source text is reproduced. **Owner** marks the owner's decisions, reviewed 2026-11-12.

## Owner's decisions (2026-11-12)

1. **Learning is an eighth reason (owner: yes).** A scholar, priest, herbalist or apprentice wants something studied, read, practised or made. There will be ways to train skills besides practice by use, so training can also be a quest's direct reward.
2. **Dark stories are allowed (owner), but only for characters who already have a bad reputation.** Murder by townsfolk, revenge, theft from a rival, framing someone: beats tagged *Dark* (D1 to D5 below). This calls for a global **alignment** on players (not designed yet: a Quest system matter). A quest with an alignment requirement is **hidden completely** from characters who don't meet it: no grey "!" and no hint, unlike other quests' "not yet".
3. **Twists later in chains (owner: yes)**, and **moral quandaries** are good too: a later step may offer a choice between two outcomes (Q beats below), which can tie into alignment.
4. **The fate is mostly told before accepting (owner)**: the giver says what they know, so the player knows what they are taking on. **But a story may contain lies**: a giver or witness may say something false (L beats below), with the truth found at the place, at turn-in or in a later step.
5. **Pets use the creature kinds we have (owner)** (Horse, Chicken, Cattle, Wolf as a dog), with a note to add pet kinds (Dog, Cat, Goat, Pig, Sheep, Bird, Donkey...) to the Pet values as creature kinds are added.
6. **Show the reward (owner)**, so the player can decide knowingly. The text may name the real reward (from the quest), except where a twist or lie changes it.
7. **Checkable seasons and weather (owner: yes), for higher-reward quests**: a beat tied to the planet's real season or the place's climate (S10, C13) is kept for quests with bigger rewards.

## Sources and what we take from each

### Doran & Parberry 2011

*A prototype quest generator based on a structural analysis of quests from four MMORPGs* (PCG workshop, FDG 2011). Over 750 quests from EverQuest (35%), World of Warcraft (27%), Eve Online (19%) and Vanguard (18%). Every quest has a giver's **motivation**, a **strategy** chosen for it, and the strategy is a short sequence of **actions**; an action can itself expand into a sub-quest, which gives nesting.

Motivations and their share of the quests (their Table 3): Conquest 20.2%, Equipment 18.5%, Knowledge 18.3%, Protection 18.2%, Serenity 13.7%, Reputation 6.5%, Wealth 2.0%, Comfort 1.6%, Ability 1.1%.

Strategies, summarised (labels ours; check against the paper before quoting it):

| Motivation | Strategies (each a few actions) |
|---|---|
| Knowledge | bring an item to be studied; spy on someone or something; question someone; use an item in the field and report |
| Comfort | fetch luxuries; kill pests |
| Reputation | fetch something rare; kill the giver's enemies; visit a dangerous place |
| Serenity | revenge or justice; capture a wrongdoer; check on someone; recover something lost or stolen; rescue someone held |
| Protection | attack what threatens; treat or mend; draw attackers away; build defences; guard someone or something |
| Conquest | attack an enemy; steal from them |
| Wealth | gather raw materials; steal valuables; make valuables |
| Ability | assemble a tool for a skill; fetch training materials; use tools; practise fighting; practise a skill; research a skill |
| Equipment | assemble; deliver supplies; steal supplies; trade for supplies |

Atomic actions (about 20): go to, explore, kill, capture, damage, defend, escort, gather, take, give, exchange, use, repair, experiment, listen, read, report, spy, stealth.

**What we take:** motive first, then strategy, then actions is the same shape as reason → plot → beats here. Their actions are our quest goals (Slay ≈ kill, Explore, Find ≈ take, Deliver ≈ give, Visit/Scout ≈ go to and report, Gather, Escort). Their nesting is our chains. Their finding that games use few motives, unevenly, says a handful of well-written reasons beats many thin ones.

### Propp: the folktale's 31 functions and 7 roles

*Morphology of the Folktale* (1928). Russian wonder tales are one sequence of up to 31 functions, always in the same order, with parts left out. Grouped as we use them:

- **Preparation:** someone leaves home (absentation); a warning given and broken; the villain learns and tricks.
- **Lack or harm:** the villain does harm, or something is lacking (the pivot of every tale).
- **The hero sets out:** the lack is made known and the hero sent (mediation, the dispatcher); the hero agrees and leaves.
- **Donor:** the hero is tested, answers, and gains a helper or a magical means.
- **Getting there:** the hero is led to the place.
- **Struggle:** fight, marking of the hero, victory.
- **Putting right:** the lack is ended (the item found, the captive freed).
- **Return:** coming home, pursuit, rescue from pursuit; arriving unrecognised; a false hero's claim; a hard task, solved; recognition, the false hero exposed, the hero transformed, the villain punished; **wedding** (or reward).

The seven roles: villain, donor, helper, the sought-for person (and their father), dispatcher, hero, false hero.

**What we take:** the roles map almost one to one onto our cast: dispatcher = giver; sought-for = the lost person, pet or item; villain = the creatures, a named boss or (in feuds) a rival; donor = someone who gives a key, a clue or a remedy (the herbalist, an old soldier); false hero = a rival who wants the same thing (complication C7). The order of functions gives the order of beats: absence and lack (paragraph 1), the obstacle (paragraph 2), the mediation and the stake (paragraph 3), and the return functions give the turn-in text. The final function, the wedding, is exactly the grandmother's-ring stake.

### Polti: the 36 dramatic situations

*Les trente-six situations dramatiques* (1895; English 1916, public domain). Each situation is a few roles in a relation. Those that suit a giver's request (numbers are Polti's):

| Use | Situations |
|---|---|
| Fits well | 1 Supplication; 2 Deliverance (rescue); 5 Pursuit; 6 Disaster; 7 Falling prey to misfortune; 9 Daring enterprise; 10 Abduction; 11 The enigma; 12 Obtaining; 14 Rivalry of kinsmen; 17 Fatal imprudence; 21 Self-sacrifice for kin; 34 Remorse; 35 Recovery of a lost one; 36 Loss of loved ones |
| With care (chains, twists) | 24 Rivalry of superior and inferior; 28 Obstacles to love; 30 Ambition; 32 Mistaken jealousy; 33 Erroneous judgement |
| Dark (bad reputation only) | 3 Crime pursued by vengeance; 4 Vengeance for kin; 13 Enmity of kinsmen |
| Left out | adultery, incest, madness, kin-slaying, crimes of love, conflict with a god, sacrificing loved ones, and the rest |

**What we take:** a check that the seven reasons cover the useful situations: supplication is every quest's frame; 35 and 36 are family and grief; 6, 7 and 9 are threat and rumour; 14 and 33 are the neighbour feud and its twist; 17 (someone's own rashness) gives the trait-driven hook ("she always had to see over the next hill").

### Booker: the seven basic plots

*The Seven Basic Plots* (2004): Overcoming the Monster, Rags to Riches, the Quest, Voyage and Return, Comedy, Tragedy, Rebirth. Too broad for one quest, but they fit **questlines**: a chain of steps takes one shape.

| Booker plot | Questline shape here |
|---|---|
| Overcoming the Monster | threat to the town: scout, thin them out, kill the leader |
| The Quest | Artifact line: a long road to one thing |
| Voyage and Return | a relative or pet lost far away, found, brought home |
| Rags to Riches | debt or trade: a failing shop put on its feet |
| Comedy | neighbour feud ending in a misunderstanding cleared up (and perhaps a match between the families) |
| Tragedy | grief: the fate found out is the worst one; the line ends in a burial or a keepsake |
| Rebirth | someone fallen (a drunk uncle, a disgraced guard) redeemed by the last step |

### Folktale indexes: ATU and Thompson

Aarne–Thompson–Uther tale types (Uther 2004) that match our reasons, summarised:

| ATU | Type | Gives us |
|---|---|---|
| 101 | The old dog (a faithful old dog saves a child and is kept on) | lost pet, a pet's worth |
| 300 | The dragon-slayer | threat: a monster demands, a hero kills it, a false claimant |
| 301 | Princesses rescued from underground | Explore a cave, rescue, companions who betray |
| 400 / 425 | The search for a lost wife / husband | family: searching for a spouse |
| 451 | A sister seeks her lost brothers | family search |
| 461 | Three hairs from the devil's beard | fetch from a dangerous being; answers on the way |
| 505 | The grateful dead (the hero pays a dead man's debt or burial; the dead repay) | grief + debt: burying someone properly |
| 551 | The water of life (sons seek a remedy for a sick father) | Gather a remedy |
| 560 | The magic ring stolen and won back | Find a stolen ring |
| 736A | The ring of Polycrates (a lost ring comes back) | a lost ring found in an odd place |
| 780 | The singing bone (a hidden death comes to light) | grief: learning how someone really died |

Thompson's *Motif-Index of Folk-Literature* (1955–58) sorts small motifs by chapter letter. Useful chapters for tagging beats: B (animals: faithful and helpful animals), E (the dead: returning to settle debts, wanting proper burial), H (tests; its quests section includes the quest for a lost person), N (chance and fate: lost things found), P (society: family, neighbours, trades), Q (rewards and punishments), R (captives and rescues), W (traits of character). We use the letters only as tags on beats, to keep the catalogue balanced.

### Game quest typologies

- **Aarseth 2005** ("From Hunt the Wumpus to EverQuest: introduction to quest theory", DiGRA): quests are about *place* (get somewhere), *time* (do it before) or *objective* (do this), and mostly mixtures. Our goals are place (Explore, Visit/Scout), objective (Slay, Find, Gather, Deliver) and both (Escort); time comes from complications (C4).
- **Howard 2008** (*Quests: Design, Theory, and History in Games and Narratives*): quests tie meaning to places; the place itself should carry the story. So beats name the place's kind and its history (H13), not only its direction.
- **Ashmore & Nitsche 2007** ("The quest in a generated world", DiGRA): generated quests feel empty unless they lean on the generated world's real features. So every checkable fact (creatures at the place, direction, distance, what the place is) must be true, and colour facts must never contradict them.
- **Grinblat & Bucklew 2017** (Caves of Qud's generated histories, PCG workshop): generate the *events* first, then explain them backwards with motives. Our order is the same: the quest generator picks goal, poi and creatures; the story explains why.
- **Kybartas & Bidarra 2017** (survey of story generation, IEEE TCIAIG): quests sit between fully authored and fully generated; the safe middle is authored fragments with generated structure and slots, which is this plan.
- Common fetch / kill / escort lists (any MMO wiki) add nothing beyond the above.

## Language-neutral facts

All facts are values; each language has a phrase per value. Checkable facts are worked out from the world and must be true; colour facts are rolled from the text seed.

| Fact | Values (first draft) | Kind |
|---|---|---|
| Relation (to the giver) | Grandmother, Grandfather, Mother, Father, Sister, Brother, Daughter, Son, Aunt, Uncle, Cousin, Spouse, Betrothed, Friend, Neighbour, Apprentice, Master, Partner | colour |
| Trait (a person's) | Adventurous, Curious, Stubborn, Proud, Greedy, Kind, Fearful, Careless, Pious, Brave, HotTempered, Forgetful, Sickly, Old | colour |
| State (a cast member's) | Alive, Missing, Dead, Captive, Sick, Hurt, Gone (left for good) | colour, carried across steps |
| Errand (why someone went to a place) | Treasure, Herbs, Hunting, Graveside, Trade, Shelter, Work, Pilgrimage, Curiosity, Rescue, Eloping | colour, limited by place kind |
| TimeAgo | Days, Weeks, LastSeason, Years, LongAgo (a generation) | colour; LongAgo makes Alive unlikely |
| Direction | N, NE, E, SE, S, SW, W, NW, or Here (in town) | checkable, from the giver's town |
| DistanceBand | Near (in sight of town), Short (a short walk), Day (most of a day), Far (days) | checkable; bands set in Flavour text PR 2 |
| PlaceKind | Cave, Crypt, Grave, Ruin, Tower, Stronghold, Dwelling, Store, Camp, Pyramid, Settlement, Farm, HerbalistHut, Oasis, Wilds | checkable, from `PoiKind` (below) |
| Creatures | a `CreatureKind` and a count band (One, Few, Many) | checkable |
| ItemCategory | Ring, Necklace, Brooch, Locket, Weapon, Tool, Letter, Book, Deed, Box, Instrument, Toy, Remedy, Herb, Goods, Coin | checkable when the quest has an item |
| Occasion | Wedding, Birth, Funeral, Festival, ComingOfAge, NameDay, Harvest, Winter | colour |
| Stake | Memory, Safety, Livelihood, Honour, Promise, Occasion, Child, Love | colour |
| Reward | the quest's real reward (coins, an item, training in a skill, a favour), named in the text (owner) | checkable, from the quest |
| Season, Weather | the planet's season; the place's climate or weather (owner: only in higher-reward quests) | checkable |
| Alignment | a requirement band on the quest (e.g. BadReputation) for Dark beats; Quest system to design | checkable, hides the quest |
| Truth | per stated fact: True or Lie, and who lied (owner: stories may lie) | colour, set at generation |
| Gender | `shared::gender::Gender` (Male, Female, Neutral) | per cast member |

`PoiKind` into `PlaceKind`: Cave (Grassy, Earth, Rock, Ice caves); Crypt (Crypt, MossyCrypt, BloodyCrypt, Mausoleum); Grave (Grave); Ruin (Ruins, Boulders); Tower (StoneTower, WoodenTower); Stronghold (Castle, Fort, ArcheryRange); Dwelling (ThatchedHut, AdobeHouse, StiltHut, GreenStiltHut, the cottages, SnowyCottage, IceShelter); Store (AdobeStore, Warehouse, Barrel, Barrels); Camp (the tents); Pyramid (Stone and Sand pyramids); Settlement (Hamlet, Village, Town, City); Farm; HerbalistHut; Oasis; Wilds (no poi: the wilderness near a point).

Errands each place allows: Treasure (Ruin, Pyramid, Crypt, Cave, Tower), Herbs (Wilds, Cave, Oasis, HerbalistHut), Hunting (Wilds, Camp, Cave), Graveside (Grave, Crypt), Trade (Settlement, Store, Camp, Farm), Shelter (Cave, Tower, Ruin, Dwelling), Work (Farm, Stronghold, Tower, Store), Pilgrimage (Pyramid, Tower, Grave, Oasis), Curiosity (any), Rescue (any), Eloping (Settlement, Dwelling).

## Roles

| Role | Who | Notes |
|---|---|---|
| Giver | the person at the door | always; name, gender, trade from the building |
| Relative | kin or friend of the giver | carries Relation, Trait, State |
| Lost | the person or pet the story is about | usually the Relative or the Pet |
| Pet | an animal of the giver's household | an existing creature kind for now (Horse, Chicken, Cattle, Wolf as a dog); add pet kinds as creatures are added (owner) |
| Rival | a neighbour or other family | named, in the same town or a neighbouring one |
| Recipient | who receives a delivery | in another building or town |
| Escortee | who is escorted | may be the Relative |
| Witness | who saw or told the rumour | a traveller, a child, a hunter; may be unnamed ("a hunter") |
| Donor | who gives a key, clue or remedy | the herbalist, an old soldier |
| Boss | a named creature | named with a translated epithet (Flavour text PR 2) |
| Creatures | the kind at the place | checkable |
| Item | the quest item | category checkable; whose it was is colour |
| Place | the target poi | kind, direction, distance checkable |
| Home | the giver's town | its name |

## Reasons

How each reason maps to Doran & Parberry's motives, Polti's situations and the goals it suits (★ suits well, · possible, blank never):

| Reason | D&P motives | Polti | Slay | Explore | Find | Deliver | Visit/Scout | Gather | Escort |
|---|---|---|---|---|---|---|---|---|---|
| Rumour | Knowledge, Reputation, Wealth | 9, 11, 12 | · | ★ | ★ | | ★ | · | |
| Family | Serenity, Protection | 2, 21, 35 | · | · | ★ | · | · | · | ★ |
| Neighbour feud | Serenity, Reputation, Conquest (softened) | 14, 24, 33 | · | | ★ | · | · | · | |
| Lost pet | Comfort, Serenity | 35 | · | · | | | · | | ★ |
| Debt or trade | Wealth, Equipment, Ability | 1, 30 | · | | · | ★ | · | ★ | · |
| Threat to the town | Protection, Comfort (pests), Conquest | 6, 7, 2 | ★ | · | | · | ★ | · | · |
| Grief | Serenity, Knowledge | 34, 36, 3 | · | · | ★ | · | ★ | · | |
| Learning (owner: added) | Knowledge, Ability | 11 | | · | ★ | · | ★ | ★ | · |

Which place kinds each reason uses well: Rumour (Ruin, Pyramid, Tower, Cave, Crypt, Oasis); Family (any, mostly Dwelling, Farm, Settlement, Cave, Ruin); Neighbour feud (Farm, Dwelling, Store, Settlement, Wilds); Lost pet (Wilds, Cave, Farm, Ruin, Dwelling); Debt or trade (Settlement, Store, Farm, Camp, HerbalistHut); Threat (Cave, Camp, Ruin, Stronghold, Wilds, Farm); Grief (Grave, Crypt, Ruin, Cave, Stronghold, Wilds); Learning (Tower, Pyramid, Ruin, Crypt, HerbalistHut).

## Beats

Each beat sits in one paragraph slot: **H** hook and backstory, **C** complication, **A** the ask, **S** the stake. Roles and facts listed are what the beat needs; checkable facts must hold in the world. Tags are Thompson chapters and Propp functions.

### Hook and backstory (paragraph 1)

| Beat | What happens | Roles | Facts | Reasons | Tags |
|---|---|---|---|---|---|
| H1 RelativeWent | the relative went to the place on an errand and never came back | Relative, Place | Relation, Trait, Errand, TimeAgo, Direction, Distance, State Missing or Dead | Family, Grief | Propp absentation; H |
| H2 HeirloomLost | something of the relative's was lost at the place | Relative, Item, Place | Relation, ItemCategory, TimeAgo, Direction, Distance | Family, Grief | N |
| H3 DrivenOff | the family lived or worked at the place until creatures drove them off | Relative?, Creatures, Place (Farm, Dwelling, Settlement, Store) | Creatures, TimeAgo | Family, Threat, Debt | Propp villainy |
| H4 PetStrayed | the pet ran off towards the place, chasing something or frightened | Pet, Place | Trait (pet), Direction, Distance, TimeAgo Days or Weeks | Lost pet | B |
| H5 RumourHeard | someone told of something at the place: lights, a beast, treasure, a voice | Witness, Place | Errand Treasure or Curiosity, Direction, Distance | Rumour, Threat | Propp reconnaissance |
| H6 NeighbourWrong | the rival moved a boundary, kept something lent, spoiled a crop, or blamed the giver | Rival, Item? | Relation Neighbour, Trait (rival) | Neighbour feud | P |
| H7 DebtOwed | the giver owes, or is owed, by someone in another town | Recipient or Rival, Home | Distance, Direction | Debt or trade | P, E (debts of the dead, with Grief) |
| H8 OrderPending | a commission or order needs goods or materials | Recipient? | ItemCategory Goods, Herb or Tool | Debt or trade, Learning | |
| H9 ThreatSeen | creatures were seen near town, took livestock or hurt someone | Creatures, Witness?, Place | Creatures, Direction, Distance, TimeAgo Days | Threat | Propp villainy |
| H10 DiedThere | the relative died at the place (creatures, accident, sickness, age) and was never brought home | Relative, Place | Relation, State Dead, TimeAgo, Errand | Grief | E |
| H11 DyingWish | the relative asked something of the giver before dying | Relative | Relation, State Dead | Grief, Family | E, M (promises) |
| H12 GiverWasThere | the giver went there long ago and fled | Place | Trait (giver) Fearful or Old, TimeAgo Years | Rumour, Grief, Threat | |
| H13 PlaceHistory | what the place was: a family's home, a shrine, a fort, a mine | Place | PlaceKind, TimeAgo LongAgo | any | Howard: place carries story |
| H14 BossNamed | a named creature leads them | Boss, Creatures | Creatures | Threat, Rumour, Grief | Propp villain |
| H15 SickAtHome | someone at home is ill and needs a remedy | Relative | Relation, State Sick | Family, Debt (Learning) | ATU 551 |
| H16 Eloped | the relative went off with someone the family disapproved of | Relative | Relation, Errand Eloping | Family | Polti 28 (care) |

### Complications (paragraph 2)

| Beat | What happens | Roles | Facts | Reasons |
|---|---|---|---|---|
| C1 FirstTryFailed | someone went before and fled, came back hurt, or didn't come back | Relative or Witness or Giver | State Hurt or Missing | any |
| C2 CreaturesMovedIn | since then, creatures have taken the place | Creatures, Place | Creatures (checkable) | Family, Grief, Rumour, Debt |
| C3 GiverCannot | the giver is too old, hurt, needed at the shop, or afraid | Giver | Trait Old, Fearful, or State Hurt | any |
| C4 TimePressure | the occasion is soon, or the debt falls due | | Occasion or Stake | Family, Debt, Threat |
| C5 KeepItQuiet | the giver asks that the rival or family not be told | Rival or Relative | Relation | Feud, Family, Debt |
| C6 NobodyBelieves | the town laughs at the rumour; only the giver believes it | Witness? | | Rumour, Threat |
| C7 RivalSeeksIt | someone else is after the same thing (Propp's false hero) | Rival | Trait Greedy or Proud | Rumour, Feud, Grief |
| C8 Accused | the giver's family is blamed for the loss | Rival | | Feud |
| C9 FateUnknown | nobody knows whether the relative lives | Relative | State Missing | Family, Lost pet |
| C10 CannotPay | the giver can't afford a merchant's price or a fine | Recipient or Rival | Reward | Debt |
| C11 Guilt | the giver caused it: an argument, a gate left open | Giver, Relative or Pet | Trait Careless or HotTempered | Family, Lost pet, Grief |
| C12 WrongPlaceSearched | they searched the wrong place; the giver now knows better | Witness | Direction | Family, Lost pet, Rumour |
| C13 WeatherBars | the weather or season makes the way hard (snow, floods, heat) and it must be done before it worsens (higher-reward quests only; owner) | Place | Season, Weather (checkable) | any |

### Dark beats (owner: only for characters with a bad reputation; the quest is hidden from everyone else)

| Beat | What happens | Roles | Facts | Reasons | Tags |
|---|---|---|---|---|---|
| D1 Murdered | the relative was killed by a person, not a creature | Relative, Rival | Relation, State Dead | Grief, Feud | Polti 3, 4 |
| D2 Revenge | the giver wants the killer or wrongdoer harmed | Rival | Trait HotTempered | Grief, Feud | Polti 3 |
| D3 StealFrom | take something from the rival's house or store | Rival, Item, Place (Dwelling, Store, Farm) | ItemCategory | Feud, Debt | D&P Conquest, Wealth |
| D4 Frame | plant something to have someone blamed | Rival, Item | | Feud | |
| D5 Silence | make a witness keep quiet | Witness | | Feud, Debt | |

### Lies, twists and quandaries (later chain steps; owner)

| Beat | What happens | Roles | Facts | Where |
|---|---|---|---|---|
| L1 GiverLies | something the giver said is false (the relative didn't go for herbs; the ring wasn't hers) | Giver | Truth Lie on one fact | told in any step, found out at the place or later |
| L2 WitnessLies | the rumour or account was false or mistaken | Witness | Truth Lie | Rumour, Feud |
| L3 TruthFound | the lie comes out: at the place, at turn-in, or in the next step | the liar | the false fact and the true one | turn-in or chain step |
| T1 NotGuilty | the rival wasn't the wrongdoer after all | Rival | | chain step 2+ (Polti 32, 33) |
| T2 LeftOnPurpose | the lost relative left by choice | Relative | State Gone | chain step 2+ |
| T3 BossWasOneOfUs | the named creature was once someone from town | Boss | | chain step 2+ |
| Q1 WhoToHelp | two people want opposite things; the step is done for one of them | Giver, Rival | | chain step 2+, may move alignment |
| Q2 KeepOrReturn | keep the find, or give it to its first owner | Item, Rival or Relative | | chain step 2+, may move alignment |
| Q3 SpareOrSlay | the boss or a wrongdoer asks to be spared | Boss or Rival | | chain step 2+, may move alignment |

### The ask (paragraph 3, always present)

| Beat | Ask | Goal | Roles |
|---|---|---|---|
| A1 Retrieve | bring the item back | Find | Item, Place |
| A2 Clear | kill the creatures (or the boss) | Slay | Creatures or Boss, Place |
| A3 LearnFate | go and see what became of them (the giver mostly says what they believe; owner) | Explore or Visit | Lost, Place |
| A4 Take | take this to the recipient | Deliver | Item, Recipient |
| A5 Bring | see the person or pet safely there or home | Escort | Escortee or Pet, Place |
| A6 Collect | gather a number of herbs or materials | Gather | Item (Herb, Goods), Place? |
| A7 Look | look at the place and tell the giver what is there | Visit/Scout | Place |
| A8 Lay to rest | bring back remains or a keepsake for burial | Find | Relative (Dead), Item |

### The stake (paragraph 3, 0 to 2)

| Beat | Stake | Facts | Reasons |
|---|---|---|---|
| S1 Occasion | it is wanted for a wedding, birth, festival | Occasion | Family, Debt |
| S2 Memory | to remember them by; something to hold | Stake Memory | Grief, Family |
| S3 Safety | before they come to town, or take a child | Stake Safety | Threat, Lost pet |
| S4 Honour | to prove the rival wrong | Stake Honour | Feud |
| S5 Livelihood | or the farm, shop or trade is lost | Stake Livelihood | Debt, Threat |
| S6 Child | a child misses the pet or the parent | Stake Child, Relation | Lost pet, Family |
| S7 Promise | to keep a promise to the dead | Stake Promise | Grief |
| S8 Reward | the real reward, named so the player can decide (owner) | Reward | any |
| S10 Season | before the season turns: winter, harvest, the rains (higher-reward quests only; owner) | Season | Debt, Threat, Family |
| S9 Warning | be careful: a word about the creatures or the giver's fear | Creatures, Trait | any with creatures |

## Rules for putting beats together

- **One reason, one plot.** The quest generator chooses goal, place, creatures and item; the story picks a reason that suits that goal and place kind (tables above), weighted, by `hash(text seed, "reason")`. A chain keeps its reason unless a step says otherwise.
- **Three paragraphs at most.** Paragraph 1: one or two H beats. Paragraph 2: zero to two C beats. Paragraph 3: exactly one A beat and zero to two S beats. Short quests and dailies merge 2 into 3 or drop to one paragraph (H + A). Only the ask is required.
- **Needs before beats.** A beat is only chosen when its roles are filled and its checkable facts hold: C2 only if the place really has creatures; H4 only with a Pet; A5 only for Escort. The server picks; the client only words.
- **No contradictions.** A cast member's State is set once (H1 Missing, H10 Dead) and only moves forward (Missing → Dead or Alive-found, never Dead → Alive) across a questline. TimeAgo LongAgo excludes State Alive for anyone who went there. Errand must be one the place kind allows.
- **Not the same beat twice** in a quest, and avoid the same H beat in two of a giver's quests in a row.
- **Twists and quandaries** only as planned later steps of a chain (owner): T and Q beats chosen when the chain is generated. Lies (L beats) may be in any step, but their truth is fixed at generation, so nothing contradicts it later.
- **Dark beats** only in quests with an alignment requirement; such a quest is hidden entirely from characters who don't meet it (owner).
- **Season and weather beats** (C13, S10) only in higher-reward quests, and only when the server can check them (owner).
- **Turn-in text** follows Propp's return functions: thanks (always), what was found (for A3, and any lie coming out), the stake met ("the wedding can go ahead"), and in chains a lead to the next step.

## Worked examples

English here is only to show how the beats read; real wordings come from the `.ftl` files (Flavour text PR 4).

### 1. The grandmother's ring (Family, Find)

Quest: Find, Item Ring, Place RockCave (NW, Day), Creatures Kobold Few. Cast: Giver Lena (Female), Relative Grandmother (Female, Trait Adventurous, State Dead, TimeAgo Years).

Beats: H1 RelativeWent (Errand Curiosity) + H2 HeirloomLost · C2 CreaturesMovedIn + C1 FirstTryFailed (by Father, Hurt) · A1 Retrieve + S1 Occasion (Wedding) + S8 Reward (Item).

> My grandmother never could leave a hill unclimbed. Years ago she went to the caves a day's walk north-west of here and came home without her ring, the one her own mother gave her.
>
> Kobolds have taken the caves since. My father tried to go back for it last spring and came home with a broken arm.
>
> I'm to be married at the turn of the season, and I'd give anything to wear her ring. Bring it back and her silver locket is yours, and a few coins besides.

### 2. The dog that chased a hare (Lost pet, Escort)

Quest: Escort pet home, Place Ruins (E, Short). Cast: Giver Tam (Male), Pet Dog "Patch" (Trait Curious), Relative Daughter.

Beats: H4 PetStrayed · C11 Guilt (gate left open) · A5 Bring + S6 Child.

> Patch went after a hare three days back and ran clean into the old ruins east of town. I left the gate open; that's on me.
>
> My girl hasn't stopped crying. Find him and bring him home, would you?

### 3. The boundary stone (Neighbour feud, Find; a chain with a twist)

Step 1: Find, Item Deed, Place Farm (S, Near). Cast: Giver Orla (Female, Trait Proud), Rival Bren Hollis (Male, Neighbour, Trait Greedy).
Beats: H6 NeighbourWrong (moved the boundary) · C8 Accused + C5 KeepItQuiet · A1 Retrieve (the old deed, lost when the farm south of town was abandoned) + S4 Honour.

Step 2 (twist, planned with the chain): the deed shows the stone was moved before either family came. T beat "rival not guilty"; A4 Take (the deed to Bren) + S4 Honour turned to peace (Booker's Comedy).

### 4. Wolves on the north road (Threat, Slay then Scout)

Quest: Slay Wolf ×6 at Wilds (N, Short), then Scout the Cave beyond. Cast: Giver the smith Hal (Male), Witness "a carter".
Beats: H9 ThreatSeen (two sheep taken) + H14 BossNamed (a grey leader with a translated epithet) · C6 NobodyBelieves (the leader) · A2 Clear + S3 Safety. Step 2: A7 Look at the cave they came from.

### 5. A father's sword (Grief, Find in a crypt)

Quest: Find, Item Weapon, Place Crypt (W, Day), Creatures Skeleton-like kind when there is one (today's kinds: whatever the crypt holds). Cast: Giver Mira (Female), Relative Father (State Dead, Errand Work: he guarded the old fort), Trait Brave.
Beats: H10 DiedThere + H13 PlaceHistory · C3 GiverCannot (Fearful) · A8 Lay to rest (the sword, for his grave) + S7 Promise.

### 6. The overdue order (Debt or trade, Gather then Deliver)

Quest: Gather Herb ×8 at Wilds (SE, Short), then Deliver Goods to a shop in a neighbouring town (E, Day). Cast: Giver the herbalist Wenna (Female, Old), Recipient Davin Marsh (Male, Partner).
Beats: H8 OrderPending + H7 DebtOwed · C4 TimePressure + C3 GiverCannot (Old) · A6 Collect, then A4 Take + S5 Livelihood.

## Carrying a cast across steps

A chain keeps one cast. A later step names a role from an earlier quest (`quest X, role Grandmother`) instead of rolling one; the server rebuilds that cast from the earlier quest's seed and applies the states its finished steps set. So example 1 can lead to "the kobolds had more than her ring: her old map, too" (Rumour, Explore) with the same Grandmother, still Dead, and the Father still Hurt-then-recovered only if a step said so.

## Gaps and decisions

| Gap | Decision |
|---|---|
| No creature kinds for the dead (skeletons, ghosts) or many pets | use what exists; add pet kinds to the Pet values as creature kinds are added (owner) |
| Seasons and weather in text | only checkable ones, in higher-reward quests (owner) |
| Murder and dark kin situations | allowed as Dark beats behind a bad reputation (owner); needs a player alignment, to be designed with the Quest system |
| Learning reason | added (owner); training a skill can be a reward |
| Exact strategy wording in Doran & Parberry | summarised from the paper's tables; the percentages were checked, the strategy labels are ours |
| Thompson motif numbers | chapter letters only; no numbers needed for tagging |
| How many wordings per beat | Flavour text PR 4 (about 4 to 8 a beat in English to start) |
