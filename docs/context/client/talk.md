# client: talk

Talking at town doors (owner: no one is inside yet, so the door is the person). Clicking a door opens its talk panel from beside it, or walks to the cell in front and opens it on arriving: a panel named for who is behind the door ("Mara Tull, baker"), with the quests they have, and for a shop a Trade tab with what its keeper has in a grid (this was the `trade` module's shop panel, folded in). Not the markers over doors (`quests`), nor the Quest Log.

## Files

- `crates/client/src/talk/mod.rs`: `TalkPlugin`, `AT`, `NEXT_AT`, `COLUMNS`. Unit tests in `tests.rs`.
- `talk/state.rs`: `OpenTalk` (one door's panel: building, document, door, `Tab`, the shop's `StockGrid` and its holder, the giver's town and offers, whether they have answered, the quest shown in full, one just handed in with its `Ending`, a refusal, `dirty`), `OpenTalks` (`of`, `by_key`, `trading_with`, `is_open`), `key_of(building)` (the panel's key in its ids: `Id::stand_in(building)`), `Keepers`, `Heading`, `beside`, `front_of`.
- `talk/markup.rs`: `TALK`, the panel (`talk#<key>/...`): a tab bar (shops only), the Quests page (`list`, `detail` with `title`, `paras`, `steps_head`, `steps`, `reward` and the Accept, Complete, Decline and Back buttons, `refused`), the Trade page's `scroll` and `object-grid` (`stock`), and templates `offer` (a quiet button), `hint`, `para`, `step`.
- `talk/open.rs`: `change_of_mind`, `follow_clicked_door`, `open_on_arrival` (each asks with `ask_at_door`: `QuestAsk::Talk` for the town in `quests::DoorMarkers`, and `OpenTrade` for a shop), `ask_again_with_truesight`.
- `talk/spawn.rs`: `open_panel` (spawns a door's document at a tab if not open, titled with its building's kind until the giver is known), `close_all`.
- `talk/stock.rs`: `read_server` (keepers, `TradeStock` opening a shop's panel at its Trade tab or catching its stock up, a new level closing every panel), `bind_grids`, `draw_grids`, `note_gifts`, `show_gift_aim`.
- `talk/news.rs`: `read_quest_news`: `QuestNews::Talk` (opens the panel at Quests if the giver has anything; the first answer shows Quests if something can be taken or handed in, and a lone one in full), `TurnedIn` (the giver's last words), `Refused`, `Changed`.
- `talk/input.rs`: `read_events`: tabs, a row of the list, Accept and Complete (each followed by asking `Talk` again), Decline and Back.
- `talk/quests_page.rs`: `show_pages` (when dirty or the quest book changes: tab bar and pages, the title "Name, occupation" from the first offer's giver, the refusal, then the list or a detail), `Ids`, `story_of`.
- `talk/detail.rs`: `Detail`, `detail_of` (on offer: story, steps from its asks, return, reward, Accept / Decline; taken: steps with counts and "(done)", Complete once ready; handed in: the ending and reward).
- `talk/words.rs`: `giver_title` (`occupation-<kind>` from `talk.ftl`, none for houses), `hint_text`, `refusal`, `plain`, `or_partly` (what could be written, missing parts in braces, logged).
- `talk/close.rs`: `close_on_escape`, `close_when_left`.

## Public API

- `TalkPlugin`.

## Depends on

- `common` (`CurrentDungeon`, `GameView`, `MapPosition`, `PointerSystems`, `TakesGifts`, `HeldOverGifts`, `GaveItem`, `TookItem`, `HeldStock`, `Truesight`, `Wording`), `quests` (`DoorMarkers`, `QuestBook`, `done_steps`), `inventory` (`Carried`), `connection` (`ToServer`, `FromServer`), `dungeon_map` (`DoorClicked`), `player` (`Player`, `Arrived`, `TakeStep`, `Walking`, `TravelTo`, `PlayerSystems`), `dom`, `widgets`, `shared::protocol` (`QuestAsk`, `QuestNews`, `Offer`, `OfferState`, `Hint`, `QuestRefusal`), `shared::story` (wording), `shared::lang`, `shared::shop`, `shared::item`.

## Used by

- Nothing.

## Notes

- One panel per building on the level (never two for the same door), keyed by the building as a stand-in id, as houses have no keeper. Two side by side can both be open; a second click raises it.
- A panel only opens on an answer worth showing: a giver's `Talk` with offers, or a shop's `TradeStock`. A house with nothing to offer opens nothing, as walking to its door always did. The server checks `DOOR_REACH` itself, and answers `Talk` only beside the door.
- The town asked about is `quests::DoorMarkers::town`, which the server sends on arriving in any town (site or wilderness cutout).
- Text is written from the `Story` the server sends (`shared::story`), in `common::Wording`'s language; hints and refusals from `shared/src/lang/en/talk.ftl`, in the giver's words, never numbers. Alignment-hidden quests never reach the client.
- Rows of the list are one per offer in order, so a press on row `n` is offer `n`; locked offers are hint lines ("!" and what the giver says), not buttons.
- Accept and Complete are followed by `Talk` again, so the list (and the quest now under way) are as the server has them. A turned-in quest's detail needs its log entry (`quests::QuestBook`), which `Changed` brings before `TurnedIn`.
- Selling (owner: giving, for now): all of a grid has `TakesGifts`, so `inventory` treats dropping there as giving (`GiveItem`). Green shows where it will land. Grid (owner's design): 12 wide, as many rows as needed, oldest first, empty rows below as many as the longest side of any kind. Stock is picked up as carried things are (`common::StockCell`), free for now; `TradeStock` puts a refused take right. Stock doesn't change while open.
- Esc closes the panel opened last, ahead of the system menu. The close button (`UiValue::Closed` by `talk#<key>/panel`), walking out of reach, a new level and leaving the map also close panels. `close_when_left` forgets panels whose entity doesn't exist, so it runs before `read_server`.
- A tap arrives as a click and a press, so all of this works by touch.
