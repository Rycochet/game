### English: stories' facts, beats, titles and steps (shared::story).
### A beat's wordings are its message's attributes (.a, .b...); the story picks
### one by its text seed, so adding or changing wordings rewords old quests.
### Each beat may only use the values it is sure to have (story/fits.rs; the
### table in story/wording_tests.rs checks every wording against them). The
### giver speaks, to the player. Drafted with research-tools/llm.py, then
### chosen and checked by hand (Flavour text PR 4).

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
time-ago-last-season = last year
time-ago-years = years ago
time-ago-long-ago = long ago

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

## Items, with their plural ($items) as an attribute.

item-category-ring = ring
    .plural = rings
item-category-necklace = necklace
    .plural = necklaces
item-category-brooch = brooch
    .plural = brooches
item-category-locket = locket
    .plural = lockets
item-category-weapon = sword
    .plural = swords
item-category-tool = tool
    .plural = tools
item-category-letter = letter
    .plural = letters
item-category-book = book
    .plural = books
item-category-deed = deed
    .plural = deeds
item-category-box = box
    .plural = boxes
item-category-instrument = fiddle
    .plural = fiddles
item-category-toy = toy
    .plural = toys
item-category-remedy = remedy
    .plural = remedies
item-category-herb = herb
    .plural = herbs
item-category-goods = goods
    .plural = goods
item-category-coin = coin
    .plural = coins

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

## Helpers wordings call: creatures with their number ("a kobold", "a few
## wolves"), and pronouns for each role ({ relative-he }). Neutral is "they",
## so a verb after it agrees ({ $relative-gender -> [neutral] were *[other] was }).

creatures-some = { $creature-count ->
    [one] { A($creatures) }
   *[other] { $creature-number } { $creatures }
}
giver-he = { $giver-gender ->
    [male] he
    [female] she
   *[neutral] they
}
giver-him = { $giver-gender ->
    [male] him
    [female] her
   *[neutral] them
}
giver-his = { $giver-gender ->
    [male] his
    [female] her
   *[neutral] their
}
giver-himself = { $giver-gender ->
    [male] himself
    [female] herself
   *[neutral] themselves
}
relative-he = { $relative-gender ->
    [male] he
    [female] she
   *[neutral] they
}
relative-him = { $relative-gender ->
    [male] him
    [female] her
   *[neutral] them
}
relative-his = { $relative-gender ->
    [male] his
    [female] her
   *[neutral] their
}
relative-himself = { $relative-gender ->
    [male] himself
    [female] herself
   *[neutral] themselves
}
rival-he = { $rival-gender ->
    [male] he
    [female] she
   *[neutral] they
}
rival-him = { $rival-gender ->
    [male] him
    [female] her
   *[neutral] them
}
rival-his = { $rival-gender ->
    [male] his
    [female] her
   *[neutral] their
}
rival-himself = { $rival-gender ->
    [male] himself
    [female] herself
   *[neutral] themselves
}
recipient-he = { $recipient-gender ->
    [male] he
    [female] she
   *[neutral] they
}
recipient-him = { $recipient-gender ->
    [male] him
    [female] her
   *[neutral] them
}
recipient-his = { $recipient-gender ->
    [male] his
    [female] her
   *[neutral] their
}
recipient-himself = { $recipient-gender ->
    [male] himself
    [female] herself
   *[neutral] themselves
}
witness-he = { $witness-gender ->
    [male] he
    [female] she
   *[neutral] they
}
witness-him = { $witness-gender ->
    [male] him
    [female] her
   *[neutral] them
}
witness-his = { $witness-gender ->
    [male] his
    [female] her
   *[neutral] their
}
witness-himself = { $witness-gender ->
    [male] himself
    [female] herself
   *[neutral] themselves
}
pet-he = { $pet-gender ->
    [male] he
    [female] she
   *[neutral] they
}
pet-him = { $pet-gender ->
    [male] him
    [female] her
   *[neutral] them
}
pet-his = { $pet-gender ->
    [male] his
    [female] her
   *[neutral] their
}
pet-himself = { $pet-gender ->
    [male] himself
    [female] herself
   *[neutral] themselves
}
boss-he = { $boss-gender ->
    [male] he
    [female] she
   *[neutral] they
}
boss-him = { $boss-gender ->
    [male] him
    [female] her
   *[neutral] them
}
boss-his = { $boss-gender ->
    [male] his
    [female] her
   *[neutral] their
}
boss-himself = { $boss-gender ->
    [male] himself
    [female] herself
   *[neutral] themselves
}

## Hooks (paragraph 1).

beat-relative-went =
    .a = My { $relative-relation } went to { the-place } { $errand }, { $time-ago }, and never came home.
    .b = { CAP($time-ago) }, { $relative-given } set off for { the-place } { $errand }. That was the last any of us saw of { relative-him }.
    .c = It was { $time-ago } that my { $relative-relation }, { $relative-name }, went out to { the-place } { $errand }. { CAP(relative-he) } never came back, and nobody in { $home } could tell me why.
    .d = My { $relative-relation } was always { $relative-nature }, so when { relative-he } headed off to { the-place } { $errand } I thought nothing of it. That was { $time-ago }, and { relative-he } { $relative-gender ->
        [neutral] have
       *[other] has
    } not come home.
    .e = I still set a place at the table some nights. { CAP($time-ago) }, my { $relative-relation } went to { the-place } { $errand }, and I have been waiting ever since.

beat-heirloom-lost =
    .a = My { $relative-relation } lost { relative-his } { $item } at { the-place } { $time-ago }, and it has never come home.
    .b = { CAP($time-ago) }, { $relative-given } dropped { relative-his } { $item } somewhere in { the-place }. I still think about it more than I should.
    .c = There's { A($item) } lying out at { the-place }, { $place-direction }. It belonged to my { $relative-relation }, { $relative-name }, and it was lost there { $time-ago }.
    .d = It's only { A($item) }, I know, but it was my { POSSESSIVE($relative-relation) }, and it's been lying out at { the-place } since { $time-ago }.
    .e = Every family has its keepsakes. Ours is { POSSESSIVE($relative-given) } { $item }, and it was lost at { the-place }, { $place-distance }, { $time-ago }.

beat-driven-off =
    .a = My family worked { the-place } for as long as anyone can remember, until { creatures-some } came and drove us out { $time-ago }.
    .b = We used to live at { the-place }, { $place-direction }. Then the { $creatures } came, and we left with nothing but what we could carry.
    .c = I grew up at { the-place }, { $place-distance }. It's the { $creatures } who sleep there now, and my family makes do with a borrowed room here in { $home }.
    .d = Not a morning goes by that I don't think of { the-place }. We had a good living there before the { $creatures } took it from us.
    .e = Our stores, our tools, our beds, all still sitting at { the-place }. The { $creatures } chased us off { $time-ago } and nobody has dared go back since.

beat-pet-strayed =
    .a = Our { $pet-kind }, { $pet-name }, slipped out of the yard { $time-ago } and went off towards { the-place }.
    .b = { CAP($time-ago) }, { $pet-name } bolted, and the last anyone saw, { pet-he } { $pet-gender ->
        [neutral] were
       *[other] was
    } heading for { the-place }, { $place-direction }.
    .c = { $pet-name } has always been { $pet-nature }, but running off to { the-place } { $time-ago } is a new one, even for { pet-him }.
    .d = The children have cried themselves to sleep since { $pet-name } wandered off towards { the-place } { $time-ago }.
    .e = I've called and called, but { $pet-name } hasn't come home. { CAP($time-ago) }, { pet-he } ran off in the direction of { the-place }, and I can't go after { pet-him } myself.

beat-rumour-heard =
    .a = { CAP($witness-name) } came through here swearing { witness-he } saw something strange out at { the-place }, and { witness-he } { $witness-gender ->
        [neutral] don't
       *[other] doesn't
    } scare easily.
    .b = I wouldn't have given it a second thought, but { $witness-name } told me there's something odd going on at { the-place }, { $place-direction }.
    .c = { CAP($witness-name) } stopped at my door, pale as milk, and said { witness-he } { $witness-gender ->
        [neutral] were
       *[other] was
    } never going near { the-place } again.
    .d = There's talk in { $home }, and it started with { $witness-name }. { CAP(witness-he) } { $witness-gender ->
        [neutral] say
       *[other] says
    } things aren't right at { the-place }.
    .e = Folk laughed when { $witness-name } told what { witness-he } saw at { the-place }. I didn't laugh, and I still can't sleep for it.

beat-neighbour-wrong =
    .a = { $rival-name } has done my family a wrong, and { rival-he } won't own up to it no matter how many times I ask.
    .b = I'll not dress it up. { $rival-name } wronged us, plain as day, and { rival-he } { $rival-gender ->
        [neutral] stand
       *[other] stands
    } there and { $rival-gender ->
        [neutral] deny
       *[other] denies
    } it to my face.
    .c = We've lived beside { $rival-given } for years, and now { rival-he } { $rival-gender ->
        [neutral] have
       *[other] has
    } done my family harm and { $rival-gender ->
        [neutral] pretend
       *[other] pretends
    } nothing happened.
    .d = Everyone in { $home } knows what { $rival-name } did to us. Everyone except { $rival-given }, it seems, who swears { rival-he } never did a thing.
    .e = I'm not one to bear a grudge, but { $rival-given } hurt my family, and it galls me that { rival-he } { $rival-gender ->
        [neutral] go
       *[other] goes
    } about { $home } as if { rival-his } hands were clean.

beat-debt-owed =
    .a = I owe { $recipient-name } in { $destination }, and it's been sitting on my conscience longer than I care to admit.
    .b = { CAP($recipient-given) } and I have an old debt between us, and { recipient-he } { $recipient-gender ->
        [neutral] live
       *[other] lives
    } over in { $destination }, { $destination-distance }.
    .c = There's money owed between me and { $recipient-name } of { $destination }, and I'd sooner see it settled before it sours into bad blood.
    .d = Years of trading with { $recipient-given } over in { $destination }, and somewhere along the way the accounts stopped balancing.
    .e = { CAP($destination) } is { $destination-direction }, { $destination-distance }, and in it lives { $recipient-name }, to whom I owe more than I'd like.

beat-order-pending =
    .a = I've an order promised to a customer by week's end, and I can't fill it without the { $item }.
    .b = Folk in { $home } are waiting on work I said I'd finish, and I'm stuck until I get my hands on the { $item }.
    .c = There's an order sitting half done on my bench. All it wants is the { $item }, and I haven't got it.
    .d = I took coin up front for a job, more fool me, and now I need the { $item } before I can make good on it.
    .e = My trade lives or dies on keeping my word, and right now my word depends on the { $item }.

beat-threat-seen =
    .a = { CAP(creatures-some) } { $creature-count ->
        [one] was
       *[other] were
    } seen near { $home } { $time-ago }, and nobody here has slept easy since.
    .b = I'll not pretend otherwise, I'm frightened. { CAP(creatures-some) } { $creature-count ->
        [one] has
       *[other] have
    } been prowling about the edge of { $home }, seen { $time-ago }.
    .c = { CAP($time-ago) }, someone spotted { creatures-some } just outside { $home }. We'd hoped it was a trick of the light, but it wasn't.
    .d = We've always been a quiet sort of place, { $home }. Then { creatures-some } turned up on our doorstep { $time-ago }.
    .e = Folk keep their doors barred now, and I can't blame them, not with the { $creatures } seen so near { $home }.

beat-died-there =
    .a = My { $relative-relation } went to { the-place } { $errand }, and { relative-he } never came back. { CAP(relative-he) } died there, and { relative-he } { $relative-gender ->
        [neutral] lie
       *[other] lies
    } there still.
    .b = I lost my { $relative-relation } at { the-place }, { $place-direction }. { CAP(relative-he) } went there { $errand } and died there, and nobody ever carried { relative-him } home.
    .c = My { $relative-relation } went to { the-place } { $errand } and never left it again. It's { $place-distance }, and still too far to bring { relative-him } home.
    .d = { POSSESSIVE($relative-given) } bones are still out at { the-place }. { CAP(relative-he) } went { $errand }, being { $relative-nature } as { relative-he } always { $relative-gender ->
        [neutral] were
       *[other] was
    }, and it killed { relative-him }.
    .e = It's been hard on me since { $time-ago }, when my { $relative-relation } { $relative-name } died at { the-place }. We never even got to bury { relative-him } in { $home }.

beat-dying-wish =
    .a = Before my { $relative-relation } passed, { relative-he } took my hand and asked one last thing of me, and I gave my word.
    .b = { CAP($relative-given) } asked me for something on { relative-his } deathbed. I promised, and I have not slept well since.
    .c = My { $relative-relation }, { $relative-name }, made me swear to something at the very end. I'd have sworn to anything, just to ease { relative-him }.
    .d = I sat with { $relative-given } through the final night, and before dawn { relative-he } asked a promise of me. I mean to keep it.
    .e = When my { $relative-relation } was dying, { relative-he } called me close and asked me a favour. I told { relative-him } I would see it done, and a promise to the dying is not one you break.

beat-giver-was-there =
    .a = I went to { the-place } myself { $time-ago }, and I ran from it before I'd gone ten steps inside. I'm not proud of that.
    .b = { CAP($time-ago) } I walked out to { the-place }, { $place-direction }, thinking I'd see for myself. I came home at a run and didn't sleep for a week.
    .c = I've been there, you know. { CAP(the-place) }. I got close enough to hear something moving, and then my legs made the choice for me.
    .d = It's only { $place-distance }, and I went once, { $time-ago }. I didn't stay. Whatever is out there, I felt it watching me all the way back to { $home }.
    .e = I don't talk about the day I went out to { the-place }. I'll only say I came back faster than I went, and I've not been back since.

beat-place-history =
    .a = Folk say { the-place } was something else entirely, long ago, though nobody agrees on what.
    .b = My gran used to tell me { the-place } had another life once, before anyone in { $home } can remember.
    .c = { CAP(the-place) } wasn't always as it is now, or so the old ones say over their ale.
    .d = There's talk that { the-place }, { $place-direction }, was a grander thing in its day. I couldn't tell you what, and I doubt anyone could.
    .e = Ask anyone here and they'll swear { the-place } used to be different. Different how, they never quite say.

beat-boss-named =
    .a = They answer to one called { $boss-name }, and folk here have learnt to lower their voices when they say it.
    .b = There's a leader among them, { $boss-name }, and since { boss-he } came the raids have had a purpose to them.
    .c = I've heard the name { $boss-name } whispered at the well, and I wish to the gods I hadn't.
    .d = Whatever drives them on, it's { $boss-name } giving the orders, and { boss-he } { $boss-gender ->
        [neutral] don't
       *[other] doesn't
    } seem to tire of it.
    .e = Every story that reaches { $home } lately ends with the same name, { $boss-name }, and I've had my fill of those stories.

beat-sick-at-home =
    .a = My { $relative-relation } { $relative-given } has taken ill, and { relative-he } { $relative-gender ->
        [neutral] grow
       *[other] grows
    } weaker every morning.
    .b = I've sat up three nights now with my { $relative-relation }, and the fever won't break. { CAP(relative-he) } { $relative-gender ->
        [neutral] don't
       *[other] doesn't
    } even know me some hours.
    .c = { CAP($relative-given) } is sick abed at home, and whatever I try, { relative-he } only { $relative-gender ->
        [neutral] get
       *[other] gets
    } worse.
    .d = It's my { $relative-relation }. { CAP(relative-he) } { $relative-gender ->
        [neutral] were
       *[other] was
    } always { $relative-nature }, but now { relative-he } can barely lift { relative-his } head from the pillow.
    .e = I keep telling myself it's only a chill, but my { $relative-relation } coughs worse each night, and I'm frightened, if I'm honest.

beat-eloped =
    .a = My { $relative-relation }, { $relative-given }, ran off with someone none of us could stand, and { relative-he } never looked back.
    .b = We told { $relative-given } plainly that we would never bless it, so { relative-he } packed a bag and left { $home } with the one we'd turned away.
    .c = I'll not pretend I was kind about it. When my { $relative-relation } chose someone we didn't approve of, I said hard things, and { relative-he } ran off with them the next morning.
    .d = { CAP($relative-given) } was always { $relative-nature }, but I never thought { relative-he } would run away with someone the family had sent packing.
    .e = Our family has its pride, and when { $relative-name } took up with someone beneath it, { relative-he } chose them over us and simply went.

beat-study-wanted =
    .a = There's something I've wanted to learn since I was small, and there's nobody in { $home } who can teach it to me.
    .b = I've asked everyone in { $home } who might know, and not one of them can show me what I want to learn.
    .c = Folk here know their fields and their trades well enough, but what I want to learn isn't taught in { $home }, and never has been.
    .d = I'm not getting any younger, and if I don't find a way to learn this soon, I never will.
    .e = Call me { $giver-given }. I've a head full of questions and nobody in { $home } with the answers.

beat-murdered =
    .a = My { $relative-relation }, { $relative-given }, is dead, and I know who did it. { $rival-name } killed { relative-him }, whatever the rest of { $home } wants to believe.
    .b = I'll not dress it up. { $rival-name } murdered my { $relative-relation }, and { rival-he } still walks past my door every morning as if nothing happened.
    .c = { POSSESSIVE($relative-given) } gone. My { $relative-relation }, { $relative-nature } as { relative-he } { $relative-gender ->
        [neutral] were
       *[other] was
    }, never hurt a soul, and { $rival-given } cut { relative-him } down all the same.
    .d = Folk in { $home } keep telling me to let it rest. How can I, when the one who killed my { $relative-relation } lives three doors down? It was { $rival-name }, I'd swear it on any grave you like.
    .e = { CAP($rival-given) } and my { $relative-relation } quarrelled for years, over nothing much. Then one night { $relative-given } didn't come home, and I've no doubt in my heart whose doing that was.


## Complications (paragraph 2).

beat-first-try-failed =
    .a = I went out to { the-place } myself, and I came limping back with this leg. I'll not manage it twice.
    .b = I tried, you know. Got as far as { the-place } before I took a fall that near broke my arm, and I've been useless since.
    .c = It's { $place-distance }, so I thought I could manage it alone. I was wrong, and I've the bruises to prove it.
    .d = I made it out to { the-place } once already. Barely made it back, mind, and these ribs still ache every time I breathe.
    .e = Folk in { $home } told me not to go, and I went anyway. I came back hurt and none the wiser, so here I am asking you.

beat-creatures-moved-in =
    .a = Since then, { creatures-some } { $creature-count ->
        [one] has
       *[other] have
    } moved into { the-place }, and nobody goes near it now.
    .b = I'd go myself, but { creatures-some } { $creature-count ->
        [one] has
       *[other] have
    } taken the place over since, and I'm no fighter.
    .c = It was bad enough before. Now { the-place } belongs to the { $creatures }, and they don't share.
    .d = Folk who've passed by say { creatures-some } { $creature-count ->
        [one] has
       *[other] have
    } settled in and grown bold about it.
    .e = And as if that weren't enough, the { $creatures } came. { CAP(the-place) } is theirs now, for all I can do about it.

beat-giver-cannot =
    .a = My legs aren't what they were, and I'd only slow down anyone fool enough to take me along.
    .b = I'll be honest with you, I'm afraid. I've never been further than the edge of { $home } and I'm not about to start now.
    .c = Someone has to keep the shop open and the animals fed, and there's nobody here but me to do it.
    .d = If I were twenty years younger I'd have gone already. As it is, I can barely manage the hill to the well.
    .e = I've tried to work up the nerve, truly I have, but every time I reach the gate my hands start shaking and I turn back.

beat-time-pressure =
    .a = And it has to be soon, mind. { CAP($occasion) } is nearly on us, and I won't have it spoiled.
    .b = Time's short, that's the trouble. { CAP($occasion) } will be here before I've stopped fretting about it.
    .c = I'd not rush you if I had the choice, but { $occasion } is only days off now.
    .d = If it isn't sorted before { $occasion }, I don't know what I'll tell folk. It's coming up fast.
    .e = I keep counting the days to { $occasion }, and every morning there are fewer of them.

beat-keep-it-quiet =
    .a = And please, not a word of this to { $rival-name }. { CAP(rival-he) } would only make things worse.
    .b = Keep this between us. If { $rival-given } hears I've asked for help, I'll never hear the end of it.
    .c = I'd rather { $rival-name } didn't find out about any of this. { CAP(rival-he) } { $rival-gender ->
        [neutral] are
       *[other] is
    } { $rival-nature } enough as it is.
    .d = Don't go asking around about this, will you? It'd be round to { $rival-name } by supper, and then I'd be in a proper mess.
    .e = If { $rival-given } should ask you what I wanted, tell { rival-him } I was selling you eggs. Anything but the truth.

beat-nobody-believes =
    .a = Nobody in { $home } believes a word of it, of course. They just nod and wait for me to go away.
    .b = I've told everyone who'd listen, and they all laughed. Folk here think I've gone soft in the head.
    .c = The reeve said I'd been at the cider again. I hadn't touched a drop, I swear it.
    .d = In { $home }, once you've been called a fool, you stay one. Nobody will hear me out, so I'm asking you instead.
    .e = I keep telling them, and they keep shrugging. If something comes of it, they'll all say they knew all along.

beat-rival-seeks-it =
    .a = And I'll tell you straight, { $rival-name } wants it just as badly as I do, and { rival-he } { $rival-gender ->
        [neutral] aren't
       *[other] isn't
    } one to wait politely.
    .b = { CAP($rival-given) } has been asking about it too. I saw { rival-him } poking around only yesterday, pretending not to.
    .c = You should know { $rival-name } is after the very same thing. { CAP(rival-he) } always { $rival-gender ->
        [neutral] were
       *[other] was
    } { $rival-nature }, and that hasn't changed.
    .d = I'd not trouble you if it weren't for { $rival-given }. { CAP(rival-he) } { $rival-gender ->
        [neutral] have
       *[other] has
    } got wind of it, and if { rival-he } { $rival-gender ->
        [neutral] get
       *[other] gets
    } there first, I'll never see it again.
    .e = Keep an eye out for { $rival-name }. { CAP(rival-he) } { $rival-gender ->
        [neutral] want
       *[other] wants
    } it too, and { rival-he } won't think twice about getting in your way.

beat-accused =
    .a = To make matters worse, { $rival-name } has been telling half of { $home } that my family is to blame.
    .b = { CAP($rival-given) } says it was us, of course. { CAP(rival-he) } { $rival-gender ->
        [neutral] point
       *[other] points
    } the finger at my family every time something goes wrong.
    .c = Now { $rival-given } goes about accusing my kin, and people are starting to listen. I can hardly hold my head up at market.
    .d = { CAP($rival-given) } { $rival-gender ->
        [neutral] are
       *[other] is
    } { $rival-nature }, and that makes { rival-him } quick to blame. This time { rival-he } { $rival-gender ->
        [neutral] blame
       *[other] blames
    } us.
    .e = We've done nothing, I swear it, yet { $rival-name } won't hear a word of it and keeps calling my family liars and worse.

beat-fate-unknown =
    .a = Nobody can tell me if { $relative-given } is still alive, and the not knowing is the worst of it.
    .b = Some days I'm sure my { $relative-relation } is dead, and some days I'm sure { relative-he } { $relative-gender ->
        [neutral] are
       *[other] is
    } not. I can't grieve and I can't stop hoping.
    .c = I don't know if I'm looking for { $relative-given } or for a grave. I just need to know which.
    .d = Folk in { $home } have stopped asking after { $relative-given }. I think they've decided { relative-he } { $relative-gender ->
        [neutral] are
       *[other] is
    } gone, but nobody has seen a body.
    .e = Alive or dead, I couldn't tell you. I've lit a candle for my { $relative-relation } every night all the same.

beat-cannot-pay =
    .a = They want more than I have, and I've turned out every pocket and jar in the house twice over.
    .b = I can't pay what they're asking. I'm not too proud to say it, not any more.
    .c = The price went up the moment they saw I needed it, and there's no way I can meet it.
    .d = Even if I sold the roof over my head, I'd still come up short of what they want.
    .e = I'm an honest trader, but honest doesn't fill a purse, and they won't take my word in place of coin.

beat-guilt =
    .a = It's my own fault, really. I left the gate open, and { $pet-name } was through it before I'd even noticed.
    .b = I keep going over it in my head. One latch, that's all I had to check, and I didn't.
    .c = I was in a hurry and never shut the door behind me. If anything's happened to { pet-him }, I'll not forgive myself.
    .d = I'm meant to be the careful one in this house. Then I go and leave the door wide open for { $pet-name } to wander off.
    .e = I haven't slept for blaming myself. I went out to the well and forgot the door, and that's the last I saw of my poor { $pet-kind }.

beat-wrong-place-searched =
    .a = I spent the better part of a week turning over every barn and ditch, and all for nothing. Now I'm near certain it's { the-place }.
    .b = Like a fool, I looked everywhere close to home first. It was only later I thought of { the-place }, { $place-direction }.
    .c = We went the wrong way entirely at the start, and that cost us time we didn't have. It has to be { the-place }.
    .d = I'll not pretend I was clever about it. I searched the wrong ground for days, and only now do I think { the-place } is where to look.
    .e = Folk sent me off in the wrong direction, and I believed them. Now I'm sure it's { the-place }, { $place-distance }.

beat-weather-bars =
    .a = I won't lie to you, { $weather } will make the road to { the-place } hard going.
    .b = With { $weather } as it is, getting to { the-place } won't be easy, so go carefully.
    .c = Mind { $weather }. It's turned the way to { the-place } into a misery.
    .d = It's { $place-distance }, which wouldn't be much, but { $weather } makes every step of it twice the work.
    .e = You'll curse { $weather } before you reach { the-place }, I'm sure of it. Better you know that now than find out on the road.

beat-not-guilty =
    .a = Now I'm not so sure it was { $rival-name } at all, and that sits badly with me.
    .b = I've been thinking it over, and maybe I blamed { $rival-given } too quickly. { CAP(rival-he) } { $rival-gender ->
        [neutral] are
       *[other] is
    } { $rival-nature }, but that doesn't make { rival-him } guilty.
    .c = Folk in { $home } are saying { $rival-given } was nowhere near when it happened. If that's true, I owe { rival-him } an apology.
    .d = I'll admit it. I wanted it to be { $rival-given }, because we've never got on, and wanting isn't the same as knowing.
    .e = Something doesn't fit. { CAP($rival-given) } could never keep quiet about anything, so why would { rival-he } keep quiet about this?

beat-left-on-purpose =
    .a = There's a part of me that wonders if { $relative-given } simply wanted to go, and nothing took { relative-him } at all.
    .b = Folk in { $home } whisper that my { $relative-relation } left of { relative-his } own accord. I don't want to believe it, but I can't say they're wrong.
    .c = { CAP($relative-given) } was always { $relative-nature }, and I've caught myself thinking { relative-he } might have walked off on purpose. If so, I only want to hear it from { relative-his } own mouth.
    .d = Maybe nobody dragged { $relative-given } away. Maybe { relative-he } packed up and went because { relative-he } { $relative-gender ->
        [neutral] were
       *[other] was
    } tired of { $home }, and tired of me.
    .e = If { $relative-name } chose to leave, I'll not force { relative-him } home. I just need to know { relative-he } { $relative-gender ->
        [neutral] are
       *[other] is
    } safe and that it was { relative-his } own choice.

beat-boss-was-one-of-us =
    .a = There's something I didn't tell you before. { CAP($boss-name) } grew up here in { $home }, and I knew { boss-him } when { boss-he } { $boss-gender ->
        [neutral] were
       *[other] was
    } small.
    .b = I'll be honest with you, since you've a right to know. { CAP($boss-name) } was one of ours once, born and raised in { $home }.
    .c = Folk here don't like to talk about it. { CAP($boss-name) } walked these very streets before { boss-he } turned to whatever { boss-he } { $boss-gender ->
        [neutral] are
       *[other] is
    } now.
    .d = You may as well hear it from me. I sat beside { $boss-name } at lessons when we were young, here in { $home }, and I never saw this coming.
    .e = I keep thinking I should have known. { CAP($boss-name) } once helped my family bring in the harvest, right here in { $home }, and now look what { boss-he } { $boss-gender ->
        [neutral] have
       *[other] has
    } become.

beat-who-to-help =
    .a = { $rival-name } will be at you next, asking the very opposite, and I'll not pretend { rival-he } { $rival-gender ->
        [neutral] have
       *[other] has
    } no reason for it.
    .b = I know { $rival-given } wants it the other way, and I'll grant { rival-he } { $rival-gender ->
        [neutral] aren't
       *[other] isn't
    } wrong about everything. Still, I'm asking you first.
    .c = You'll hear a different tale from { $rival-name }, and to be fair, some of it's true. You'll have to weigh us both up yourself.
    .d = { CAP($rival-given) } and I have never agreed on much, and we don't agree on this. I'd be lying if I said { rival-his } side had no sense in it.
    .e = I'll be honest, since you'll find out anyway. { CAP($rival-given) } has a fair point, and so do I, and only one of us can have our way.

beat-keep-or-return =
    .a = Mind you, whoever has the { $item } now may well have come by it honestly, and I'll not have you taking it by force.
    .b = I'll be honest with you, I've lain awake wondering if the { $item } is even mine to ask for any more. If the one holding it has a fair claim, I'd rather know than steal it back.
    .c = Should you find the { $item } in someone else's keeping, hear them out before you do anything. They might have a better right to it than I'd like to admit.
    .d = It galls me, but I know how these things go. Somebody may have bought the { $item }, or been given it, and then it's theirs as much as it was ever mine.
    .e = Bring it back if it's rightfully mine. If it isn't, if it was sold on or handed over honestly, then tell me so plainly and I'll make my peace with it.

beat-spare-or-slay =
    .a = If { $boss-name } begs for { boss-his } life, I'll leave it to you whether { boss-he } { $boss-gender ->
        [neutral] deserve
       *[other] deserves
    } it, though I know what I'd choose.
    .b = They say { $boss-name } talks sweetly when cornered, so if { boss-he } { $boss-gender ->
        [neutral] ask
       *[other] asks
    } for mercy, think hard before you give it.
    .c = Should { $boss-name } plead to be spared, do as your heart tells you. I've no stomach for killing, but I've less for living in fear.
    .d = I'd not blame you for sparing { $boss-name } if { boss-he } { $boss-gender ->
        [neutral] swear
       *[other] swears
    } to leave { $home } be, but promises from that sort are cheap.
    .e = Some here want { $boss-name } dead and some just want { boss-him } gone. If { boss-he } { $boss-gender ->
        [neutral] ask
       *[other] asks
    } to walk away, the choice is yours.


## The ask (paragraph 3).

beat-retrieve =
    .a = Would you go to { the-place } and bring the { $item } back to me?
    .b = I can't make the trip myself, not anymore. If you could fetch the { $item } from { the-place }, I'd be in your debt.
    .c = { CAP(the-place) } is { $place-distance }, { $place-direction }. Bring me back the { $item }, that's all I ask.
    .d = Please, find the { $item } at { the-place } and carry it home to me. I'll not rest easy until it's back in my hands.
    .e = I'm not asking you to fight anyone or fix anything. Just go to { the-place } and fetch the { $item }, and I'll be grateful.

beat-clear =
    .a = Go to { the-place } and clear out whatever has made a home there, would you?
    .b = I'm asking you to deal with { the-place }. Whatever's lurking there, put an end to it.
    .c = Someone has to go to { the-place } and make it safe again, and it can't be me.
    .d = If you've a strong arm and a good blade, clear { the-place } for us. { CAP($home) } won't forget it.
    .e = It's { $place-distance }, { the-place }. Go there and rid it of what's taken hold, and I'll count myself in your debt.

beat-learn-fate =
    .a = Would you go to { the-place } and find out what became of my { $relative-relation }? I can't bear not knowing any longer.
    .b = I'm asking you to go to { the-place }, { $place-direction }, and learn what happened to { $relative-given }. Whatever it is, I'd rather hear it.
    .c = Please, go and look for { $relative-given } at { the-place }. If { relative-he } { $relative-gender ->
        [neutral] are
       *[other] is
    } alive, tell me, and if not, tell me that too.
    .d = I need someone to go to { the-place } and bring me word of my { $relative-relation }. Good word or bad, I'll take it.
    .e = Find out what happened to { $relative-given } at { the-place }, that's all I ask. I've waited long enough by this door.

beat-take =
    .a = Would you take the { $item } to { $recipient-name } in { $destination }? I'd rest easier knowing it got there safe.
    .b = I need someone to carry the { $item } to { $destination }, { $destination-distance }, and put it into { POSSESSIVE($recipient-given) } hands. Will you do it?
    .c = Take the { $item } to { $recipient-given } for me. { CAP($destination) } is { $destination-direction }, and anyone there can point you to { recipient-him }.
    .d = I can't make the trip myself, so I'm asking you. See that { $recipient-name } in { $destination } gets the { $item }, and no one else.
    .e = Please, take my { $item } to { $destination } and hand it to { $recipient-given }. Tell { recipient-him } it comes from { $giver-given }, and { recipient-he }'ll understand.

beat-bring =
    .a = Please, go to { the-place } and walk { $relative-given } home to me, safe and whole.
    .b = I'm asking you to fetch my { $relative-relation } back from { the-place } and stay at { relative-his } side the whole way to { $home }.
    .c = Find { $relative-given } at { the-place }, { $place-distance }, and bring { relative-him } home. I don't care how long it takes, just keep { relative-him } safe.
    .d = Would you see my { $relative-relation } home from { the-place }? { CAP(relative-he) } won't come back on { relative-his } own, and I'd not trust the road alone.
    .e = I can't go myself, so I'm asking you. Bring my { $relative-relation } back from { the-place }, and don't let anything happen to { relative-him } on the way.

beat-bring-pet =
    .a = Please, find { $pet-name } somewhere about { the-place } and bring { pet-him } home to me.
    .b = I can't go myself, so I'm asking you. Go to { the-place }, find my { $pet-kind }, and bring { pet-him } home.
    .c = { CAP(the-place) } is { $place-direction }, { $place-distance }. Look for { $pet-name } there and lead { pet-him } back to { $home }, and I'll not forget it.
    .d = Bring { $pet-name } home, that's all I ask. { CAP(pet-he) } won't have gone far from { the-place }, I'm sure of it.
    .e = Would you look around { the-place } for my { $pet-kind }? { CAP(pet-he) } { $pet-gender ->
        [neutral] are
       *[other] is
    } { $pet-nature }, so go gently, but do bring { pet-him } back.

beat-collect =
    .a = Would you gather some { $items } for me? I'd not ask if I could manage it myself.
    .b = If you could bring me back a few { $items }, I'd be grateful for a long while.
    .c = I need { $items }, as many as you can carry. Will you fetch them for me?
    .d = Bring me some { $items }, would you? There's no one else in { $home } I can ask.
    .e = Could you collect some { $items } for me? Take your time, but please don't forget me.

beat-look =
    .a = Would you go out to { the-place } and have a look for me, then come back and tell me what you find?
    .b = I only need to know what's there. Walk out to { the-place }, { $place-direction }, take a good look, and tell me honestly.
    .c = { CAP(the-place) } is { $place-distance }. Go and see it with your own eyes, and come back and tell me what's there, good or bad.
    .d = You needn't fight anything or fetch anything. Just go to { the-place }, look around, and come and tell me what's what.
    .e = Please, go and see what's become of { the-place }. Not knowing is worse than anything you could tell me.

beat-lay-to-rest =
    .a = Bring { POSSESSIVE($relative-given) } { $item } back to me from { the-place }, so it can go into the ground with { relative-him } as it should.
    .b = I'm asking you to go to { the-place } and fetch the { $item }. My { $relative-relation } deserves to be buried with it, not have it left out there in the dirt.
    .c = If you could find the { $item } at { the-place } and bring it home, we could lay { $relative-given } to rest properly, with the honour { relative-he } { $relative-gender ->
        [neutral] were
       *[other] was
    } owed.
    .d = Go to { the-place }, { $place-direction }, and bring back { POSSESSIVE($relative-given) } { $item }. I'll not bury my { $relative-relation } without it.
    .e = It's only the { $item } I want. Bring it back from { the-place } and we can finally put { $relative-name } to rest the right way.

beat-steal-from =
    .a = Go to { the-place } and take the { $item } from { $rival-name }, and make sure nobody sees you do it.
    .b = I need you to slip into { the-place } and bring me back the { $item }. Quietly, mind. If { $rival-given } catches you, I never sent you.
    .c = { CAP($rival-name) } keeps the { $item } at { the-place }, { $place-direction }. Wait until dark, take it, and be gone before anyone stirs.
    .d = I'm not proud of asking, but would you fetch the { $item } out of { the-place } for me? Just don't let { $rival-given } or anyone else lay eyes on you.
    .e = All I ask is that you go to { the-place } and take back the { $item } while { $rival-name } isn't looking. No noise, no witnesses, and no talk of it after.

beat-frame =
    .a = Take the { $item } to { $destination } and leave it where it will be found, with { POSSESSIVE($rival-name) } name tied to it. Let { rival-him } explain that.
    .b = I want the { $item } left in { $destination }, somewhere folk will stumble on it, and I want them thinking { $rival-given } put it there.
    .c = Carry the { $item } to { $destination } and set it down where nobody could miss it. When they ask whose it is, the answer will be { $rival-name }, not me.
    .d = It's not a kind thing I'm asking. Plant the { $item } in { $destination } so the blame falls on { $rival-given }, and keep my name out of it.
    .e = Get the { $item } to { $destination }, { $destination-distance }, and leave it where { $rival-given } will be blamed. { CAP(rival-he) } { $rival-gender ->
        [neutral] have
       *[other] has
    } had it coming long enough.

beat-silence =
    .a = Go to { the-place } and find { $witness-name }. Make sure { witness-he } never { $witness-gender ->
        [neutral] tell
       *[other] tells
    } anyone else what { witness-he } saw.
    .b = { CAP($witness-name) } can be found at { the-place }, { $place-direction }. I need { witness-him } quiet, and I mean quiet for good.
    .c = I won't say it plainly, so listen well. Go to { the-place } and see to it that { $witness-name } { $witness-gender ->
        [neutral] have
       *[other] has
    } nothing more to say, ever.
    .d = { CAP($witness-name) } has already said too much, and I'll not sleep until { witness-he } { $witness-gender ->
        [neutral] say
       *[other] says
    } no more. Go to { the-place } and end it.
    .e = Find { $witness-name } at { the-place } and settle things so that no one in { $home } ever hears another word from { witness-him }. Don't come back until it's done.


## The stake (paragraph 3).

beat-for-occasion =
    .a = It's meant for { $occasion }, and I won't have it said that I came empty-handed.
    .b = Without it, { $occasion } won't feel right. I know that sounds foolish, but it's true.
    .c = Half of { $home } will be at { $occasion }, and every one of them will notice if it's missing.
    .d = I promised it would be there for { $occasion }. Please don't make me go back on my word.
    .e = We've been planning { $occasion } for months, and that was always part of it.

beat-memory =
    .a = It's all we have left of them now, and I can't bear to lose that too.
    .b = Please understand, it isn't worth much to anyone else, but it's the last thing we have to remember them by.
    .c = When the house is quiet of an evening, it's what we hold on to. There's nothing else left.
    .d = Call me foolish if you like, but every other trace of them is gone, and this is all the memory we have.
    .e = Folk in { $home } tell me to let it go, but how do you let go of the only thing left of someone you loved?

beat-safety =
    .a = Better it's done now, before the { $creatures } find their way into { $home } and somebody gets hurt.
    .b = I'd sooner not wait until there's blood on someone's doorstep.
    .c = If nobody stops the { $creatures } out there, sooner or later they'll come here, and then it'll be one of us they're dragging off.
    .d = We're not fighting folk in { $home }. If the { $creatures } come down our lane, we've got pitchforks and prayers, and that's all.
    .e = Do it for { $home }, if not for me. Nobody here wants to find out what the { $creatures } do once they stop keeping to the wilds.

beat-honour =
    .a = { CAP($rival-name) } has been telling half of { $home } that my family are liars, and I mean to prove { rival-him } wrong.
    .b = It's not about the money. My father's name meant something in { $home }, and I won't have { $rival-given } drag it through the mud.
    .c = People cross the street when they see me now. All I want is for them to know the truth, and for { $rival-given } to eat { rival-his } words.
    .d = { CAP($rival-given) } has always been { $rival-nature }, and folk forgive { rival-him } for it. They won't forgive us unless someone shows them who was really in the right.
    .e = Do this, and I can walk through { $home } with my head up again. That's worth more to me than anything I could pay you.

beat-livelihood =
    .a = If this goes wrong, we lose our living, and everything my family built in { $home } goes with it.
    .b = This trade is all that feeds us. Lose it, and we lose the roof over our heads too.
    .c = My father and his father made their living from this place, and I'll not be the one who lets it go.
    .d = We've scraped by on this work for years, and it's all we have. Please, don't let it end like this.

beat-child-misses =
    .a = My youngest hasn't slept a whole night since. I hear the crying through the wall and I just lie there.
    .b = There's a child in my house who keeps asking when they're coming home. I'd like to give a real answer for once.
    .c = Every night the little one sobs into the pillow, and nothing I say seems to help.
    .d = I can bear it myself, mostly. It's the little one crying in the dark that I can't stand.
    .e = Our smallest won't eat properly and wakes up calling out. A house shouldn't sound like that.

beat-promise =
    .a = I promised { $relative-given } I'd see it done, and { relative-he } can't hold me to it now, so I have to hold myself.
    .b = My { $relative-relation } made me swear to it before the end. I won't sleep easy until I've kept my word.
    .c = It was the last thing I promised { $relative-given }. Folk in { $home } keep their promises to the dead, or they ought to.
    .d = { CAP($relative-given) } asked me for one thing as { relative-he } lay dying, and I said yes without thinking twice. I meant it then and I mean it now.
    .e = I gave my word to my { $relative-relation }, and { $relative-given } never once broke { relative-his } word to me. I'll not be the one who breaks first.

beat-reward =
    .a = It isn't much, but I can give you { $reward } for your trouble.
    .b = I've put by { $reward }. It'll all be yours when the job's done, you have my word on it.
    .c = Folk in { $home } will tell you I pay what I promise, so { $reward } it is, and not a copper less.
    .d = I scraped together { $reward }. Times are hard, but this matters more to me than any of it.
    .e = You'll have { $reward } from me, and my thanks besides, which is worth more than it sounds.

beat-season-turns =
    .a = It has to be done before { $season } is out, or there's no point doing it at all.
    .b = Please, before { $season } turns. After that it'll be too late, and I'll have no one to blame but myself.
    .c = { CAP($season) } won't wait for any of us, and once it's gone, so is my chance.
    .d = I'm not asking you to hurry for the sake of it. If this isn't settled by the end of { $season }, { $home } will feel it come the next.
    .e = There's a little time left in { $season }. Not much, mind, so don't dawdle.

beat-warning =
    .a = Mind yourself out there, the { $creatures } { $creature-count ->
        [one] is
       *[other] are
    } no small thing to cross.
    .b = Don't take the { $creatures } lightly, whatever you've faced before.
    .c = I've heard what the { $creatures } can do, so keep your wits about you and come back in one piece.
    .d = Go carefully, please, there's no shame in turning back if the { $creatures } prove too much.
    .e = I'll say it plain: the { $creatures } won't go easy on you, so be ready for a hard fight.

beat-revenge =
    .a = I want { $rival-name } to pay for what { rival-he } did, and I'll not sleep easy until { rival-he } { $rival-gender ->
        [neutral] do
       *[other] does
    }.
    .b = Call it spite if you like. { CAP($rival-given) } has had it easy too long, and it's time { rival-he } learnt what it costs to cross me.
    .c = { CAP($rival-given) } laughed at me in front of half of { $home }. I want to see { rival-his } face when { rival-he } { $rival-gender ->
        [neutral] realise
       *[other] realises
    } { rival-he } { $rival-gender ->
        [neutral] have
       *[other] has
    } lost.
    .d = Everyone in { $home } knows how { $rival-nature } { $rival-given } is, and nobody ever does a thing about it. Well, I mean to.
    .e = Let { $rival-name } lose something { rival-he } { $rival-gender ->
        [neutral] care
       *[other] cares
    } about, the way I did. That's all I ask, and it's no more than fair.


## The ending, at turn-in.

beat-thanks =
    .a = You came back. I was starting to fear the worst, and here you are. Thank you.
    .b = Thank you, truly. I don't know how I'd have managed without you.
    .c = Well, I'll not pretend I'm good with words, but you have my thanks, and I mean it.
    .d = Oh, thank goodness. I've hardly slept since you left. Bless you for this.
    .e = There aren't many in { $home } who'd have bothered. Thank you for doing what you did.

beat-found =
    .a = So that's what was waiting at { the-place }. I'd half guessed, but hearing it from you is another thing entirely.
    .b = I never thought I'd hear the end of it, not after all this time. Thank you for telling me straight.
    .c = Well. That isn't what I hoped for, but at least now I know, and knowing is better than lying awake wondering.
    .d = All that way to { the-place } and back, and that's what you found? I'll be turning it over in my head for weeks.
    .e = Folk in { $home } won't believe it when I tell them. I hardly believe it myself, and I'm the one who sent you.

beat-stake-met =
    .a = Good. Now things can finally go ahead the way they ought to.
    .b = Well, that's that, then. I can stop fretting and get on with putting it all right.
    .c = You've no idea what this means. At last I can set things straight.
    .d = Right. With that done, nothing's standing in the way any more.
    .e = There. That's the knot undone, and the rest can follow as it should.

beat-truth-found =
    .a = { $liar-is-giver ->
        [yes] I owe you the truth now. { $relative-given } went to { the-place } { $truth }, not as I told you. I couldn't bring myself to say it before.
       *[no] So { $liar-name } had it wrong. { $relative-given } went to { the-place } { $truth } all along.
    }
    .b = { $liar-is-giver ->
        [yes] I wasn't straight with you. { $relative-given } really went { $truth }. I told it the other way because I was ashamed.
       *[no] { CAP($liar-name) } lied to me, then. { $relative-given } went { $truth }, and nobody thought to tell me.
    }
    .c = { $liar-is-giver ->
        [yes] You've found me out. It was { $truth } that { $relative-given } went, and I should have said so from the start.
       *[no] { $relative-given } went { $truth }? Then { $liar-name } told me wrong, and I'll be having words.
    }

beat-lead =
    .a = You've done well. Don't wander too far from { $home }, will you? There's more to this than I've let on.
    .b = That's a weight off. Still, something about it all doesn't sit right with me, and I expect I'll come looking for you.
    .c = Good. Now, I can't say more just yet, but if you're still about in a day or two, find me.
    .d = I'm grateful, I am. Only, I've a feeling this was the easy part, and I'll be asking more of you before it's done.
    .e = Well, that settles one thing. There's another I've not told you about yet, and I'd sooner you heard it from me when the time comes.

## Quests' names, from the giver's reason, using only the home town.

story-title-rumour =
    .a = Strange Talk
    .b = Whispers in { $home }
    .c = Something Out There
story-title-family =
    .a = A Family Matter
    .b = Blood and Kin
    .c = For the Family
story-title-neighbour-feud =
    .a = Bad Neighbours
    .b = Over the Fence
    .c = Bad Blood in { $home }
story-title-lost-pet =
    .a = Lost and Found
    .b = Gone Astray
    .c = The Open Gate
story-title-debt-or-trade =
    .a = A Matter of Trade
    .b = Debts and Dealings
    .c = An Honest Bargain
story-title-threat =
    .a = Trouble near { $home }
    .b = Barred Doors
    .c = Before They Come
story-title-grief =
    .a = Laid to Rest
    .b = What the Dead Ask
    .c = Unfinished Mourning
story-title-learning =
    .a = Something to Learn
    .b = The Wish to Know
    .c = Beyond { $home }

## Steps' words, plain and counted ($done of $count); a boss alone is slain
## by clearing the place.

step-slay =
    .plain = Slay the { $creatures }
    .counted = Slay { $creatures } { $done }/{ $count }
    .place = Clear { the-place }
step-explore =
    .plain = Explore { the-place }
    .counted = Explore { the-place } { $done }/{ $count }
step-find =
    .plain = Find the { $item }
    .counted = Find { $items } { $done }/{ $count }
step-deliver =
    .plain = Deliver the { $item }
    .counted = Deliver { $items } { $done }/{ $count }
step-visit =
    .plain = Visit { the-place }
    .counted = Visit { the-place } { $done }/{ $count }
step-gather =
    .plain = Gather { $items }
    .counted = Gather { $items } { $done }/{ $count }
step-escort =
    .plain = Escort home from { the-place }
    .counted = Escort home { $done }/{ $count }
step-return = Return to { $giver-name }
