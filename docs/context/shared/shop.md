# shared: shop

What town shops have in stock, and working it out when one is opened. Only the server will call it, as the seed is secret. It makes no items, since ids are the server's: it says which old stock to keep and what to add. Not used in play yet: the server and a panel follow.

## Files

- `crates/shared/src/shop.rs`: `Shop`, `sells`, `NewStock`, `Restocked`, `catch_up`, `REPLACED_AFTER`, `TOP_UP_AFTER`. Tests.

## Public API

- `sells(BuildingKind)`: what it sells, with weights; empty if it isn't a shop.
- `Shop { kind, quality, trades, of_kind }`: `target`, `minimum` (80% of it), `make_minutes(quality, kind_percent)`, `replaced_after`.
- `catch_up(seed, shop, stock qualities, last_made, last_visit, now) -> Restocked { keep, new, last_made }`.

## Depends on

- `shared::item`, `shared::rng`, `shared::site` (`BuildingKind`).

## Used by

- Nothing yet.

## Notes

- Make time: the quality's base × the kind's `ItemDef::make_percent` × 150% to 50% by shop quality ÷ √(shops of the kind in town). While time is filled the kind isn't known, so it uses the average for what the shop sells.
- Quality odds are `Quality::odds` weighted up with shop quality and trades, stopping at Legendary: artifacts only come from a questline.
- One thing is made at a time. At its count, each one finished pushes one out, by `sell_weight` (Common 100 to Legendary 50), so selling keeps pace with making, rivals included, and stock stands still during a slow make.
- A roll is hashed from the minute making began, so what is part made stays the same however often a shop is looked at.
- Past `replaced_after` (90 days ÷ √rivals) old stock is gone and only that much time is run, so a long absence costs no more.
- Reopening after `TOP_UP_AFTER` (a game day) below `minimum` adds extras up to a random count between it and the target, using none of the shop's time.
- All numbers are first guesses. Design: `decisions.md`, 2026-11-02, item quality.
