# Compiler-bound Lua bytecode cache

Persisted loader bytecode belongs to the exact Rilua Git revision resolved in `Cargo.lock`, as well as the simulator's slot/whitelist ABI. A compiler correction must take effect without manually clearing caches or changing a hand-maintained cache version.

## What it must do

- [ ] Rebuild compiler identity automatically when the locked Rilua Git revision changes, even when its package version does not.
- [ ] Reject a persisted pack from another or unidentified compiler before any cached chunk executes.
- [ ] Reopen same-compiler packs and replay the stored bytecode without rewriting them during read-only access.
- [ ] Ignore obsolete loose `.luac` files and unversioned/legacy keys rather than migrating or promoting unknown-compiler artifacts.
- [ ] Preserve whitelist-ABI validation, bounded pack storage, torn-entry recovery, read-only immutability and prefork cache modes.

## How it works

- [`build.rs`](../../build.rs) publishes the compiler identity extracted from the locked package record.
- [`src/loader/bytecode_cache.rs`](../../src/loader/bytecode_cache.rs) owns pack validation, lookup, persistence and cache modes.

## Implementation inventory

- `build/locked_rilua.rs` — strict extraction of the resolved Git revision from Cargo-generated lock data; malformed, missing or ambiguous identity fails explicitly.
- `build.rs` — watches `Cargo.lock` and emits the compile-time identity.
- `src/loader/bytecode_cache.rs` — compiler-bound keys/header and exact-key replay only.
- `src/loader/lua_file.rs` — addon-file cache consumer.
- `src/loader/chunk_cache.rs` — generated-chunk cache consumer.

## Tests asserting this spec

- `src/loader/bytecode_cache.rs::tests` — concrete temporary pack/loose-file fixtures, fresh cache states, actual chunk replay, read-only snapshots and existing bounds/prefork regressions.
- `tests/locked_rilua_identity.rs` — lock-record extraction and explicit rejection of unidentified or ambiguous compilers, in the existing grouped integration target.

## Known gaps (current cycle)

- [ ] Parent-owned final verification and real warm-cache Ellesmere replay follow targeted implementation proof.

## Out of scope

- Package pin changes, manual deletion of user caches, new parser dependencies, generalized cache infrastructure, new cross-process locking, or legacy artifact compatibility.
- Builds without a uniquely identified Git-sourced Rilua package in `Cargo.lock`; these must fail rather than reuse unverifiable bytecode.
