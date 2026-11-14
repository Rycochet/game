# shared: lang

Words in the player's language: every word sits in a language file (Fluent `.ftl`, built into the program), and Rust only picks the message and gives it values. Not what a story says (`shared::story`, Flavour text PR 3) nor which language a player has chosen (client).

## Files

- `crates/shared/src/lang.rs`: the module's `mod` lines and re-exports.
- `lang/language.rs`: `Language` (English only), its code, own name, `from_locale`, `rules`.
- `lang/args.rs`: `Args` (values by name), `Selector` (a fact's value as a key; `Gender` implements it).
- `lang/render.rs`: `Lang` (one language's messages in a Fluent bundle), `RenderError`; building, `render`, `phrase`, `ids`, `attributes`, `has`.
- `lang/variables.rs`: `Lang::variables` and `missing`, read from the messages' syntax trees.
- `lang/rules.rs`: the `Rules` trait. `lang/english.rs`: English's.
- `lang/functions.rs`: the rules as functions files call: `A`, `POSSESSIVE`, `CAP` (and Fluent's `NUMBER`).
- `lang/files.rs`: the built-in files per language. `lang/en/common.ftl`: genders by name, pronouns. `lang/en/story.ftl`: names' order, and `shared::story`'s values (epithets, pets, places, directions, distances). `lang/en/story-plots.ftl`: stories' facts, creatures, coins, beats (five wordings each, as attributes), titles and steps. `lang/en/talk.ftl`: the talk panel's words: givers' titles and occupations by building kind, hints in the giver's words, refusals, the reward line. `lang/en/quest-log.ftl`: the Quest Log's headings, "Here", who gave a quest, where it stands, the places to go, a line per giver of dailies. `lang/en/item.ftl`: an item's popup: bonuses ("+40 Toughness", restoring, how long, what ends it) and stats' names.

## Public API

- `Language::{English}` (serde by place, new ones at the end; `ALL`, `code()` "en", `name()`, `from_locale("en-GB")`, English when unknown; `rules()`).
- `Lang::new(language)`: parses the built-in files (tests keep them valid). `render(id, &args) -> Result<String, RenderError>`, where `id` is `message` or `message.attribute`; `phrase(&value)` renders `<fact>-<key>`; `ids()` sorted; `attributes(id)` (a message may keep several wordings as attributes); `has(id)`; `variables(id)` (every `$value` it and the messages it refers to use); `missing(id, &args)`; `rules()`. `Send + Sync`, so it can be a Bevy resource.
- `Args::new().name(key, "Mara").text(key, words).count(key, 3).choice(key, &value)`, `has(key)`. Keys are kebab case, as in the files (`$giver-name`).
- `Selector`: `fact()` (`gender`) and `key()` (`male`), both kebab case. Each story fact (relation, trait, time ago, direction, distance band, season, truth...) implements it when added.
- `Rules`: `indefinite` ("a sword", "an axe", "a unicorn", "an hour", "a herb"), `possessive` ("Mara's", "Charles'"), `list` ("a, b and c", no serial comma), `capitalise` (the first letter).
- `RenderError { id, text, problems }`: `text` is what could be rendered, missing parts in braces.

## Depends on

- `shared::gender` (`Gender` as a selector). `fluent-bundle` (concurrent bundle), `fluent-syntax` (the syntax tree), `unic-langid`: all Apache-2.0/MIT, pure Rust, built for wasm.

## Used by

- `shared::story` (its values are selectors; names, places, and whole stories rendered). The client's quest text, and `item_popup` (bonus lines).

## Notes

- Fluent's isolation marks round values are off: Lexend can't draw them.
- Select expressions match keys (`{ $gender -> [male] he [female] she *[neutral] they }`); an unknown key falls to the `*` default silently, so tests should cover every value. A missing value is an error, and the text shows `{$name}` in its place.
- Fluent can't pass a value into a term (named arguments are literals only), so a fact's words come from `phrase` or a select expression, not a term per fact. Messages referred to inside a message see the same values.
- Lists are joined in Rust (`rules().list`), as a Fluent value holds one thing.
- Neutral is "they" in English, and needs the plural verb (`{ $gender -> [neutral] were *[other] was }`).
- A select expression's variants must each be on their own line, even inside a sentence; a one-line `{ $x -> [a] b *[c] d }` doesn't parse.
- Messages can't be given values, so `pronoun-*` only serves `$gender`: stories use a message per role (`relative-he`, in `story-plots.ftl`). A message reference can be a function's argument (`CAP(the-place)`).
- The server never renders: it sends values and the client writes the text, so changing wordings never desyncs anything.
- Tests every language must pass: every file parses with no message written twice, and every message and attribute renders when given every value `variables` lists.
