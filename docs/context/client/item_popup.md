# client: item_popup

An item's popup, wherever items are drawn (grids, clothes, bin, ground, shops, `object type="item"`, items in writing, Give Item's preview): its name in its quality's colour, anything about where it is (worn notes), a line per bonus from `shared::bonus` worded through `shared::lang` (`item.ftl`), and its weight. Not what the pointer is on (the drawers say, through `widgets::PointedItem`) nor how a popup behaves (`dom`'s `popup hover`).

## Files

- `crates/client/src/item_popup/mod.rs`: `ItemPopupPlugin`, the module list. Tests (the markup is good).
- `item_popup/open.rs`: `ItemPopups` (open ones, the item rested on), `REST` (0.15 s), `OFFSET`, `FORM` (`item_popup#<item id>/…`), `forget_closed` (despawns on `Closed`), `follow_pointed` (keeps the one about the pointed item open through `dom::KeptOpen`, or opens one after `REST` in place of any not locked; none while something is held).
- `item_popup/show.rs`: `ITEM_POPUP` (`popup hover` with `name`, `notes`, `bonuses`, `weight`), `fill` (place and words by id), `bind_names` (the name's text in its colour, by the top of a stack).
- `item_popup/words.rs`: `bonus_lines` (worn bonuses by `item_bonuses`, food by `food_effect`: how long and what ends it), `signed` ("+40", "−0.6" with a true minus), `number` (one place at most). Tests.
- `item_popup/place.rs`: `keep_on_screen` (moves one past the window's right or bottom edge back in, once laid out).

## Public API

- `ItemPopupPlugin` only.

## Depends on

- `dom` (`popup hover`, `KeptOpen`, `Popup`, `Dom`, `DocumentOf`, `HeldObject`, `BuildDocuments`), `widgets` (`PointedItem`, `ItemsPointed`, `weight_words`, `UiId`), `common` (`Wording`, quality colours), `shared::bonus`, `shared::lang`.

## Used by

- Nothing: it reads `widgets::PointedItem`, which `widgets` (`ItemName`, items in writing) and `inventory` (its cells) fill.

## Notes

- Only what the client's copy knows goes in: `Item::name()` and the bonuses worked out from the copy's kind, quality, form and weight.
- A stack is named and its bonuses given by its top; its weight is the whole stack's (`Item::total_weight`).
- Ids are by the item's id, so one item has at most one popup; pointing at an item with a locked popup only keeps that.
- Order: `follow_pointed` runs in `PostUpdate` after `widgets::ItemsPointed` and before `dom::BuildDocuments`, so a new popup is built and filled the frame it opens.
