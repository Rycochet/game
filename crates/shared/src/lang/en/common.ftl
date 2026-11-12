### English: words used by many kinds of text.
### Values are given in kebab case ($gender). Facts arrive as keys for select
### expressions ([male], [female], [neutral]); proper names arrive as they are.

## Genders by name (Lang::phrase).

gender-male = male
gender-female = female
gender-neutral = neutral

## Pronouns for someone of $gender. Neutral is "they": a sentence using it
## needs the plural verb too ({ $gender -> [neutral] were *[other] was }).

pronoun-subject = { $gender ->
    [male] he
    [female] she
   *[neutral] they
}
pronoun-object = { $gender ->
    [male] him
    [female] her
   *[neutral] them
}
pronoun-possessive = { $gender ->
    [male] his
    [female] her
   *[neutral] their
}
pronoun-reflexive = { $gender ->
    [male] himself
    [female] herself
   *[neutral] themselves
}
