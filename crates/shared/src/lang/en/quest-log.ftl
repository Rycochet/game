### English: the Quest Log (L): its headings, who gave each quest, how far
### along it is, and the dailies done for each giver.

## The planet chooser: the planet the player is on is called this beside its
## runes (other planets show only theirs).

log-here = Here

## The list's groups.

log-active = Active
log-completed = Completed
log-failed = Failed or abandoned
log-dailies = Dailies

## When there is nothing to show.

log-none = No quests here yet.
log-choose = Choose a quest to read it.
log-asking = Asking...

## Who gave a quest, and where.

log-giver = From { $giver } of { $town }

## Where a quest stands, beside its name in full.

log-state-taken = Under way
log-state-ready = Done: hand it in
log-state-done = Handed in
log-state-failed = Failed
log-state-abandoned = Abandoned

## The places a quest sends you to, as buttons showing them on the map.

log-targets = Where to go:

## One line for each giver of dailies.

log-daily = { $giver } of { $town }: { $count ->
    [one] one done
   *[other] { $count } done
}, last { $when }
