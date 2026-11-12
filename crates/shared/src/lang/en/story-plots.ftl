### English: stories' facts, beats, titles and steps (shared::story).
### PLACEHOLDER wordings, enough for tests: Flavour text PR 4 writes the real
### ones. A beat's wordings are its message's attributes (.a, .b...); the story
### picks one by its text seed. Each beat may only use the values it is sure to
### have (docs/context/shared/story.md lists them).

## Relations to the giver.

relation-grandmother = grandmother
relation-grandfather = grandfather
relation-mother = mother
relation-father = father
relation-sister = sister
relation-brother = brother
relation-daughter = daughter
relation-son = son
relation-aunt = aunt
relation-uncle = uncle
relation-cousin = cousin
relation-spouse = spouse
relation-betrothed = betrothed
relation-friend = friend
relation-neighbour = neighbour
relation-apprentice = apprentice
relation-master = master
relation-partner = partner
relation-grandparent = grandparent
relation-parent = parent
relation-sibling = sibling
relation-child = child

## What people are like, as said after "always so".

trait-adventurous = adventurous
trait-curious = curious
trait-stubborn = stubborn
trait-proud = proud
trait-greedy = greedy
trait-kind = kind
trait-fearful = fearful
trait-careless = careless
trait-pious = pious
trait-brave = brave
trait-hot-tempered = hot-tempered
trait-forgetful = forgetful
trait-sickly = sickly
trait-old = old

state-alive = alive
state-missing = missing
state-dead = dead
state-captive = held captive
state-sick = sick
state-hurt = hurt
state-gone = gone for good

passerby-hunter = hunter
passerby-carter = carter
passerby-traveller = traveller
passerby-child = child
passerby-shepherd = shepherd
passerby-pedlar = pedlar

## Why someone went: "went there { $errand }".

errand-treasure = looking for treasure
errand-herbs = to gather herbs
errand-hunting = to hunt
errand-graveside = to visit a grave
errand-trade = to trade
errand-shelter = to shelter from a storm
errand-work = for work
errand-pilgrimage = on a pilgrimage
errand-curiosity = just to see it
errand-rescue = to help someone
errand-eloping = to run away and marry

time-ago-days = a few days ago
time-ago-weeks = some weeks ago
time-ago-last-season = last season
time-ago-years = years ago
time-ago-long-ago = long ago, before I was born

occasion-wedding = the wedding
occasion-birth = the birth
occasion-funeral = the funeral
occasion-festival = the festival
occasion-coming-of-age = the coming of age
occasion-name-day = the name day
occasion-harvest = the harvest
occasion-winter = the first snows

stake-memory = something to remember them by
stake-safety = everyone's safety
stake-livelihood = our living
stake-honour = our good name
stake-promise = a promise
stake-occasion = the day itself
stake-child = the little one
stake-love = love

item-category-ring = ring
item-category-necklace = necklace
item-category-brooch = brooch
item-category-locket = locket
item-category-weapon = sword
item-category-tool = tool
item-category-letter = letter
item-category-book = book
item-category-deed = deed
item-category-box = box
item-category-instrument = fiddle
item-category-toy = toy
item-category-remedy = remedy
item-category-herb = herb
item-category-goods = goods
item-category-coin = coin

count-one = one
count-few = a few
count-many = many

season-spring = spring
season-summer = summer
season-autumn = autumn
season-winter = winter

weather-snow = the snow
weather-rain = the rain
weather-heat = the heat
weather-storms = the storms
weather-floods = the floods

goal-slay = slay
goal-explore = explore
goal-find = find
goal-deliver = deliver
goal-visit = visit
goal-gather = gather
goal-escort = escort

reason-rumour = a rumour
reason-family = family
reason-neighbour-feud = a quarrel with the neighbours
reason-lost-pet = a lost animal
reason-debt-or-trade = trade
reason-threat = a threat to the town
reason-grief = grief
reason-learning = learning

## Creatures, by how many.

creature-beggar = { $count ->
    [one] beggar
   *[other] beggars
}
creature-birdfolk = birdfolk
creature-cattle = { $count ->
    [one] cow
   *[other] cattle
}
creature-centaur = { $count ->
    [one] centaur
   *[other] centaurs
}
creature-chicken = { $count ->
    [one] chicken
   *[other] chickens
}
creature-drider = { $count ->
    [one] drider
   *[other] driders
}
creature-dwarf = { $count ->
    [one] dwarf
   *[other] dwarves
}
creature-elf = { $count ->
    [one] elf
   *[other] elves
}
creature-giant = { $count ->
    [one] giant
   *[other] giants
}
creature-goblin = { $count ->
    [one] goblin
   *[other] goblins
}
creature-halfling = { $count ->
    [one] halfling
   *[other] halflings
}
creature-horse = { $count ->
    [one] horse
   *[other] horses
}
creature-human = { $count ->
    [one] man
   *[other] men
}
creature-kobold = { $count ->
    [one] kobold
   *[other] kobolds
}
creature-lizardfolk = lizardfolk
creature-minotaur = { $count ->
    [one] minotaur
   *[other] minotaurs
}
creature-orc = { $count ->
    [one] orc
   *[other] orcs
}
creature-scorpion-man = { $count ->
    [one] scorpion man
   *[other] scorpion men
}
creature-wolf = { $count ->
    [one] wolf
   *[other] wolves
}

## Rewards.

coin-gold = { $count ->
    [one] a gold coin
   *[other] { $count } gold coins
}
coin-silver = { $count ->
    [one] a silver coin
   *[other] { $count } silver coins
}
coin-copper = { $count ->
    [one] a copper coin
   *[other] { $count } copper coins
}
reward-training = some training
reward-favour = a favour owed

## The place, with "the" when it has no name of its own.

the-place = { $place-named ->
    [yes] { $place }
   *[no] the { $place }
}

## Hooks (paragraph 1).

beat-relative-went =
    .a = My { $relative-relation }, { $relative-given }, went to { the-place } { $place-direction } of here { $errand }, { $time-ago }, and never came back.
    .b = My { $relative-relation } { $relative-name } set out { $errand } for { the-place }, { $place-distance }, { $time-ago }, and was not seen again.
beat-heirloom-lost =
    .a = My { $relative-relation } lost { POSSESSIVE($relative-given) } { $item } at { the-place } { $time-ago }.
beat-driven-off =
    .a = We used to live at { the-place }, until { $creatures } drove us off { $time-ago }.
beat-pet-strayed =
    .a = Our { $pet-kind }, { $pet-name }, ran off towards { the-place } { $place-direction } of here { $time-ago }.
beat-rumour-heard =
    .a = { CAP($witness-name) } swears there is something strange at { the-place } { $place-direction } of here.
beat-neighbour-wrong =
    .a = { $rival-name } next door has wronged us, and won't hear a word about it.
beat-debt-owed =
    .a = { $recipient-name } in { $destination } and I have a debt to settle.
beat-order-pending =
    .a = I have an order to fill and not enough to fill it with.
beat-threat-seen =
    .a = { CAP($creatures) } were seen near { $home } { $time-ago }.
beat-died-there =
    .a = My { $relative-relation } { $relative-given } died at { the-place }, gone there { $errand } { $time-ago }, and never came home.
beat-dying-wish =
    .a = Before { $relative-given } died, I made a promise.
beat-giver-was-there =
    .a = I went to { the-place } myself { $time-ago }, and ran.
beat-place-history =
    .a = Folk say { the-place } { $place-direction } of here was something else once, long ago.
beat-boss-named =
    .a = They follow one called { $boss-name }.
beat-sick-at-home =
    .a = My { $relative-relation } { $relative-given } is sick, and getting worse.
beat-eloped =
    .a = My { $relative-relation } { $relative-given } ran off with someone we never cared for.
beat-study-wanted =
    .a = There is something I need to learn, and I can't learn it from here.
beat-murdered =
    .a = My { $relative-relation } { $relative-given } is dead, and { $rival-name } is the one who did it.

## Complications (paragraph 2).

beat-first-try-failed =
    .a = I tried to go myself and came home hurt.
beat-creatures-moved-in =
    .a = { CAP($creatures) } have taken { the-place } since.
beat-giver-cannot =
    .a = I can't go myself.
beat-time-pressure =
    .a = And { $occasion } is soon.
beat-keep-it-quiet =
    .a = Don't let { $rival-name } hear of this.
beat-nobody-believes =
    .a = Nobody else believes it.
beat-rival-seeks-it =
    .a = { $rival-name } is after it too.
beat-accused =
    .a = And now { $rival-name } blames us.
beat-fate-unknown =
    .a = Nobody knows whether { $relative-given } still lives.
beat-cannot-pay =
    .a = I can't afford what they ask.
beat-guilt =
    .a = I left the gate open. It's my fault.
beat-wrong-place-searched =
    .a = We searched the wrong place first.
beat-weather-bars =
    .a = { CAP($weather) } will make the way hard.
beat-not-guilty =
    .a = It may not have been { $rival-name } after all.
beat-left-on-purpose =
    .a = Perhaps { $relative-given } left because they wanted to.
beat-boss-was-one-of-us =
    .a = They say { $boss-name } was one of us once.
beat-who-to-help =
    .a = { $rival-name } wants the opposite of what I want.
beat-keep-or-return =
    .a = Whoever has the { $item } may want to keep it.
beat-spare-or-slay =
    .a = { $boss-name } may beg to be spared.

## The ask (paragraph 3).

beat-retrieve =
    .a = Bring back the { $item } from { the-place }.
    .b = Would you fetch the { $item } from { the-place } for me?
beat-clear =
    .a = Clear the { $creatures } out of { the-place }.
beat-learn-fate =
    .a = Go to { the-place } and find out what became of { $relative-given }.
beat-take =
    .a = Take this { $item } to { $recipient-name }.
beat-bring =
    .a = See { $relative-given } safely home.
beat-bring-pet =
    .a = Find { $pet-name } and bring them home.
beat-collect =
    .a = Gather { $item } for me.
beat-look =
    .a = Go and look at { the-place }, and tell me what is there.
beat-lay-to-rest =
    .a = Bring back { POSSESSIVE($relative-given) } { $item }, so we can bury it with honour.
beat-steal-from =
    .a = Take the { $item } from { $rival-name }, and don't get caught.
beat-frame =
    .a = Leave this { $item } where { $rival-name } will be blamed for it.
beat-silence =
    .a = Make sure { $witness-name } says nothing more.

## The stake (paragraph 3).

beat-for-occasion =
    .a = It's for { $occasion }.
beat-memory =
    .a = It's all we have left to remember them by.
beat-safety =
    .a = Before the { $creatures } come to town.
beat-honour =
    .a = I'll prove { $rival-name } wrong.
beat-livelihood =
    .a = Or we lose our living.
beat-child-misses =
    .a = The little one cries every night.
beat-promise =
    .a = I promised { $relative-given }.
beat-reward =
    .a = For your trouble: { $reward }.
beat-season-turns =
    .a = It must be done before { $season } is over.
beat-warning =
    .a = Mind the { $creatures }.
beat-revenge =
    .a = { $rival-name } will pay for it.

## The ending, at turn-in.

beat-thanks =
    .a = Thank you.
beat-found =
    .a = So that is what is at { the-place }.
beat-stake-met =
    .a = Now all will be well.
beat-truth-found =
    .a = { CAP($liar-name) } lied: it was { $truth } all along.
beat-lead =
    .a = There is more to this, I think.

## Quests' names.

story-title-rumour =
    .a = Rumours of { the-place }
story-title-family =
    .a = A Family Matter
story-title-neighbour-feud =
    .a = Bad Neighbours
story-title-lost-pet =
    .a = Lost and Found
story-title-debt-or-trade =
    .a = A Matter of Trade
story-title-threat =
    .a = Trouble near { $home }
story-title-grief =
    .a = Laid to Rest
story-title-learning =
    .a = Something to Learn

## Steps' words, plain and counted ($done of $count).

step-slay =
    .plain = Slay { $creatures }
    .counted = Slay { $creatures } { $done }/{ $count }
step-explore =
    .plain = Explore { the-place }
    .counted = Explore { the-place } { $done }/{ $count }
step-find =
    .plain = Find the { $item }
    .counted = Find { $item } { $done }/{ $count }
step-deliver =
    .plain = Deliver the { $item }
    .counted = Deliver { $item } { $done }/{ $count }
step-visit =
    .plain = Visit { the-place }
    .counted = Visit { the-place } { $done }/{ $count }
step-gather =
    .plain = Gather { $item }
    .counted = Gather { $item } { $done }/{ $count }
step-escort =
    .plain = Escort home
    .counted = Escort home { $done }/{ $count }
step-return = Return to { $giver-name }
