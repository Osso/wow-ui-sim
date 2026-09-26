# Default build linker-cache incident

A default Linux dev build failed while linking a cached `wow_ui_sim` archive. Quarantining only that archive and its matching incremental directory allowed the same default incremental build to complete; the cache mismatch cause remains unknown.

## Evidence

`cargo build --bin wow-sim -v` failed with exit `101` after `1.34475828` seconds. The recorded Rust invocation used the default dev profile's `incremental = true`, `clang`, and mold; no environment override was used. The compiler was Rust `1.98.1` with LLVM `22.1.8`.

Mold reported unresolved ordinary Rust symbols and anonymous LLVM symbols from `target/debug/deps/libwow_ui_sim-f3ad90e455e35ad1.rlib`. The archive contained an undefined reference such as `anon.92899af510cbc4e555e276cea25e54eb.52.llvm.18005206435280134742`, while a definition with the same anonymous prefix ended in `llvm.820421656293145055`. This records an internal symbol-suffix mismatch in the archive, not its cause.

Only these paths were moved to `target/linker-quarantine-20260926`:

- `target/debug/deps/libwow_ui_sim-f3ad90e455e35ad1.rlib`
- `target/debug/incremental/wow_ui_sim-0z26aiavexk6u`

Dependencies were left intact. A subsequent `cargo build --bin wow-sim -v --timings`, with no environment override or source change, rebuilt the missing library and completed successfully in `178.1638251` seconds.

## Boundaries

This does not establish disk corruption, concurrent modification, a rustc bug, or a mold bug. Earlier `CARGO_INCREMENTAL=0` evidence did not test the default workflow. The incident did not change `incremental = true`, mold, or source behavior. The passing build is compile evidence only; runtime green verification remains pending an independent verifier agent.

## Sources

- [Cargo manifest](../../Cargo.toml) — dev profile retains `incremental = true`.
- [Cargo configuration](../../.cargo/config.toml) — Linux linker remains clang with mold arguments.
- `/tmp/retail-linker/proof.json` — commands, exits, and durations.
- `/tmp/retail-linker/default-build.stderr` — default linker failure and unresolved symbols.
- `/tmp/retail-linker/archive-nm.log` — unresolved and defined anonymous-symbol suffixes in the archived library.
- `/tmp/retail-linker/quarantine.json` — exact quarantined paths.
- `/tmp/retail-linker/fresh-default-build.stderr` — rebuilt default incremental compile and successful finish.

## See Also

- [[windows-port-build]] — separate platform-specific default-build linker issue.
