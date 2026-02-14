# Talent Panel Benchmark

Measures the cost of opening the talent panel (Blizzard_PlayerSpells demand-load).

## Binary

`src/bin/bench_talents.rs` — loads all Blizzard addons, fires startup events, then opens/closes the talent panel 10 times. The first open demand-loads `Blizzard_PlayerSpells` (the expensive path); subsequent opens exercise ShowUIPanel/HideUIPanel toggling.

## Running

```bash
# Build with frame pointers for profiling
RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release --bin bench_talents

# Quick timing
cargo run --release --bin bench_talents

# Flamegraph (frame pointers, not DWARF — DWARF chokes addr2line on this binary)
perf record -F 997 --call-graph fp -g -o /tmp/perf.data -- target/release/bench_talents
perf script -i /tmp/perf.data | inferno-collapse-perf --all | inferno-flamegraph > flamegraph.svg
```

## GC benchmark

Use the `talent_panel_gc_benchmark` test in `tests/spellbook.rs` to compare talent panel creation with and without Lua garbage collection:

```bash
cargo test --test spellbook talent_panel_gc_benchmark -- --nocapture
```

## Results (2026-02-14, debug build)

| Version | First open | GC overhead |
|---|---|---|
| Before `__index` (e7f7be9) | 1.13s | 216ms (19%) |
| With `__index` on `_G` (c4d5a72) | 1.03s | ~0ms |

## Results (2026-02-14, release build, 10 opens)

| Version | First open | Subsequent open |
|---|---|---|
| Before `__index` (e7f7be9) | 431ms | 94ms |
| `__index` on `_G` (c4d5a72) | 263ms | 92ms |
| LightUserData (a3b8aff) | 262ms | 76ms |

### Profile breakdown (release, full process including addon loading)

| Category | Before | `__index` | LightUserData |
|---|---|---|---|
| Lua VM (C) | 36.8% | 35.7% | 31.2% |
| — GC | 9.5% | 9.1% | 5.0% |
| mlua bridge | 12.2% | 12.6% | 11.8% |
| Rust app | 51.1% | 52.1% | 57.1% |

### Key findings

- **`__index` on `_G`** eliminated GC overhead during talent panel creation by deferring userdata allocation. Frames are only materialized into Lua when accessed, so the GC has far fewer objects to scan during the creation burst.
- **LightUserData** nearly halved GC overhead (9.5% → 5.0%). Light userdata has no `__gc` finalizer, so the GC doesn't need to scan it for weak references (`luaC_separateudata` dropped from 1.1% to near zero).
- The biggest remaining cost is Rust-side template instantiation (`get_template`, `compute_frame_rect`, HashMap hashing) and Lua error traceback building (`luaH_next` in `compat53_findfield`).

## Flamegraphs

Stored in `docs/local/` (gitignored):
- `talents-before-index.svg` — e7f7be9
- `talents-after-index.svg` — c4d5a72 (`__index` on `_G`)
- `talents-lightuserdata.svg` — a3b8aff (LightUserData migration)
