### English: what stories are told about. Names arrive as they are; facts
### arrive as keys and are named here by Lang::phrase (<fact>-<key>).

## Whole names. A family name follows the given one in English.

person-full-name = { $given-name } { $family-name }
boss-full-name = { $name } { $epithet }

## Bosses' epithets, as said after a name.

epithet-grey = the Grey
epithet-black = the Black
epithet-white = the White
epithet-red = the Red
epithet-one-eyed = the One-Eyed
epithet-scarred = the Scarred
epithet-old = the Old
epithet-cruel = the Cruel
epithet-hungry = the Hungry
epithet-silent = the Silent
epithet-mad = the Mad
epithet-great = the Great
epithet-swift = the Swift
epithet-cunning = the Cunning
epithet-pale = the Pale
epithet-bloody = the Bloody

## Pets by kind.

pet-dog = dog
pet-horse = horse
pet-chicken = chicken
pet-cattle = cow

## Unnamed places, with no article: the story adds one (rules().indefinite).

place-kind-cave = cave
place-kind-crypt = crypt
place-kind-grave = grave
place-kind-ruin = ruin
place-kind-tower = tower
place-kind-stronghold = stronghold
place-kind-dwelling = house
place-kind-store = storehouse
place-kind-camp = camp
place-kind-pyramid = pyramid
place-kind-settlement = settlement
place-kind-farm = farm
place-kind-herbalist-hut = herbalist's hut
place-kind-oasis = oasis
place-kind-wilds = stretch of wild land

condition-whole = whole
condition-ruined = ruined

place-unnamed = { $condition ->
    [ruined] ruined { $place-kind }
   *[whole] { $place-kind }
}

## Which way from town, and how far.

direction-north = north
direction-north-east = north-east
direction-east = east
direction-south-east = south-east
direction-south = south
direction-south-west = south-west
direction-west = west
direction-north-west = north-west
direction-here = here in town

distance-near = in sight of town
distance-short = a short walk away
distance-day = most of a day away
distance-far = days away
