# Postgres migration (plan, not built)

The owner's plan for an optional Postgres backend, as an alternative to RON files, and a tool to move saves between the two. **To do once the data schema has matured:** tables are costly to change, and the saved data is still growing quickly. Before starting, check everything here against the save structs of the day (`storage.rs`, `accounts.rs`, plus whatever has been added since), and the crates' current versions and licences. Why: see `decisions.md`.

## Already built

`SaveBackend` (`crates/server/src/saves.rs`) is the interface a new backend implements, with `FileBackend` the only one so far. The save thread (`save_queue.rs`) already keeps the game from waiting on writes, retries failures and answers reads from what is queued. Loads already separate "can't reach" from "broken", and refuse players rather than start them again. See `server/server.md`.

## Decisions

- **Relational, not blobs:** real tables, so that the data is useful to query, not only safer.
- **Chosen when a server is created:** a server uses one or the other, and moves only through the migration tool.
- **RON stays for files.** The format isn't what keeps the two in parallel: the same Rust save structs are.
- **Game rules never use SQL.** They only use `SaveBackend`, so every feature works on both backends and migrating loses nothing. SQL's extra value is for admin, stats and outside tools.
- **`HOW_DATA` stays,** for certificates, unless SSL is disabled.
- A database doesn't by itself allow several servers on one world: accounts and sites are cached in memory, so servers would overwrite each other.

## Step 2: Postgres backend (one PR)

- **Setting:** `HOW_DATABASE_URL`; unset means files. Never log it, as it holds the password. Add it to `docs/environment-variables.md`.
- **Crates, behind a cargo feature** so the native client doesn't carry them, on for the Dockerfile and `cargo xtask package`: `postgres` (MIT or Apache-2.0, a blocking client suits the save thread), `tokio-postgres-rustls` (MIT, TLS with the `rustls` and `ring` already used; check it works with those versions), `refinery` (MIT, schema migrations). `sqlx` and `diesel` aren't needed: there are few queries to check or map.
- **Tables, as a starting point:**

| Table | Holds | From |
| ----- | ----- | ---- |
| `worlds` | id, seed, format, map version, clock | `WorldSave` |
| `sites` | world, poi chunk or dev details, generator version, forgotten_to | `SiteSave`, `SiteId` |
| `site_levels` | site, level, last visit | `LevelSave` |
| `cell_changes` | site, level, x, y, cell | `LevelSave::changes` |
| `buildings` | site, level, room, trades, last interaction | `BuildingSave` |
| `accounts`, `logins` | account details; login hash to account | `AccountSave`, `LoginSave` |
| `characters` | id, account (empty for single player), name, last played, current world | `CharacterSummary`, `CharacterSave` |
| `character_worlds` | position, current visit and forgotten_to in each world | `CharacterWorldSave` |
| `known_pois` | character, world, chunk, known, visited | `KnownPoi` |
| `character_sites` | character, world, site, version known | `site_versions` |
| `cell_memory` | character, world, site, level, chunk, mask | `CellMasks` |
| `items` | id (primary key, so one item can't be in two places), kind, position, rotation, and either a character or a parent bag | `Inventory`, `Item`, `Placed` |

- **Details:** enums by name, not number, so reordering `Cell` or `PoiKind` can't corrupt saves. `u64` as `bigint` with the same bits (it can look negative in SQL; views or functions can make it readable). A character's memory outlives a forgotten site, as it does in files, so `cell_memory` and `character_sites` can't cascade-delete with `sites`.
- **Writing only what changed:** the interface stays whole structs. The backend, on the save thread, compares each with the last version it saved and writes the rows that differ, in one transaction. No game code changes. Whole-character rewrites at scale were the concern: roughly 1 MB/s for 100 players at 50 KB every 5 s (an estimate).
- **New fields:** `#[serde(default)]` no longer covers them, so each also needs a schema migration and row code. Unpack each struct with no `..` in the row code, so a new field fails to compile until handled.
- **Starting up:**
  - `HOW_DATABASE_URL` set but unreachable: a few retries (for Docker's start order), then stop; never fall back to files.
  - Set, database empty, saves in the data folder: refuse, naming the migration tool, so players don't seem to have lost everything.
  - Unset, on a migrated server (the tool's marker file in the data folder): refuse, rather than run on old files.
  - A Postgres advisory lock per world: a second server on the same one is refused.
- **Tests:** the shared backend tests in `save_queue.rs` run against it too. `cargo xtask test` has no database and Docker isn't on the owner's machine, so real Postgres tests run only when `HOW_DEBUG_TEST_DATABASE_URL` is set. Postgres on Windows or on the test server is needed to run them.
- **Docs:** `docs/saves.md`, `docs/environment-variables.md`, `docs/hosting/` with `docker-compose.yml`, `server/server.md`, `project.md`, `decisions.md`.

## Step 3: Migration tool (one PR)

- A separate binary in the server crate, sharing the backends without being part of the server.
- One `copy(from, to)` for both directions. It adds list operations to `SaveBackend` (all worlds, sites, characters, accounts, logins), used only by the tool.
- **Verifies** by reading everything back from both sides and comparing the structs. The source is left untouched.
- Broken saves are skipped and listed in a report.
- Moving to Postgres leaves the marker file in the data folder (see "Starting up").
- The server must be stopped. Postgres can check with its advisory lock; the file backend needs a lock file, which it doesn't have yet.
- Old files that are no longer saves (characters kept in a world's folder, set aside) aren't copied.
