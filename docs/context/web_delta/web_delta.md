# web_delta

The `crates/web_delta` crate: delta downloads of the web client, a new build of the wasm compressed against an old build a browser already has, in the `dcz` encoding of Compression Dictionary Transport (RFC 9842). Plain Rust, native only: the server and xtask use it, never the client.

## Files

- `src/lib.rs`: `encode` / `decode` (zstd with the old build as its prefix, behind the 40-byte `dcz` header holding the old build's SHA-256), `sha256`, `hex` / `from_hex`, `find_wasm` (a web build's one `*_bg.wasm`), `DELTAS` (the `deltas` folder of deltas made beforehand) and `delta_name` (`<hash of old build>.dcz`).

## Public API

- `Hash`, `encode`, `decode`, `sha256`, `hex`, `from_hex`, `find_wasm`, `DELTAS`, `delta_name`.

## Depends on

- `zstd-safe` (MIT/Apache-2.0, building the zstd C library, BSD), `sha2` (MIT/Apache-2.0). No other crate of ours.

## Used by

- server (`network/web_files/`): deltas from the builds it served before, and serving them.
- xtask (`web-deltas`): deltas made beforehand, for releases.

## Notes

- zstd level 19 with long distance matching; the window is the largest browsers take (1.25 × the old build, at least 8 MiB, at most 128 MiB), which the 58 MB wasm fits.
- A delta only works from the exact bytes a browser has: make release deltas from the wasm as published, never a rebuild.
