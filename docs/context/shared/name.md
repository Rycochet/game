# shared: name

What a character's name may be, for both sides: the client folds what is typed into it, and the server folds again, checks it and keeps names unique (`server` `accounts/names.rs`).

## Files

- `crates/shared/src/name.rs`: `MAX_NAME` (24), `PLAIN_NAME` ("Character", for a name that can't be kept), `fold` (typed text towards a name: lookalike apostrophes, dashes and spaces made plain, accents off by compatibility decomposition, a few letters spelled out (ß, æ, ø, ł, þ...), anything else dropped; no separator first or after another, one may end it while typing), `checked` (folded, trailing separator gone, 1 to 24 long, or why not), `key` (lower case: what makes two names the same), `numbered` (`Brin 2`, cut to fit).

## Public API

All of the above.

## Depends on

- `unicode-normalization` (MIT/Apache-2.0): taking accents off.

## Used by

- Client `character_create` (the name box, the draft), the server's `accounts`, `creature::names` tests (suggested names are valid).

## Notes

- Allowed: A-Z, a-z, 0-9, and single spaces, `'` and `-` only between them. Nothing else can be in a name, so staff marks drawn beside names can't be faked.
