### English: an item's popup: what it gives (its bonuses) and what it weighs.

## A bonus: the amount with its sign ("+40", "−8"), then what it changes.

item-bonus = { $amount } { $stat }
item-bonus-percent = { $amount }% { $stat }

## Health or Mana given back over time, unsigned ("1", "0.5").

item-restores = Restores { $amount } { $stat } a second

## How long an eaten thing's bonus lasts, after its line.

item-lasts = { $line } for { $seconds } s

## What ends it early, after its line.

item-ends = { $ends ->
    [moving] { $line }; stops on moving
    [hurt] { $line }; stops on being hurt
    [both] { $line }; stops on moving or being hurt
   *[nothing] { $line }
}

## What a bonus changes: the attributes, the most Health and Mana, skills.

stat-strength = Strength
stat-agility = Agility
stat-toughness = Toughness
stat-intellect = Intellect
stat-perception = Perception
stat-will = Will
stat-presence = Presence
stat-empathy = Empathy
stat-aura = Aura
stat-most-health = most Health
stat-most-mana = most Mana
stat-health = Health
stat-mana = Mana
stat-skill = { $skill }
