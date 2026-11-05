---
name: flamegraph
description: Profile wow-sim with perf and generate flamegraph. Use when investigating performance, profiling startup, or analyzing hot paths.
---

# Flamegraph Profiling

Profile the simulator with `perf` and generate an interactive flamegraph SVG.

## Prerequisites

```bash
# One-time kernel tuning (resets on reboot)
authsudo sysctl -w kernel.perf_event_paranoid=1
authsudo sysctl -w kernel.perf_event_mlock_kb=8192
```

Tools: `perf`, `inferno-collapse-perf`, `inferno-flamegraph` (all pre-installed).

## Steps

### 1. Build release with debug info

```bash
cargo build --release --bin wow-sim
```

The release profile already has `debug = "line-tables-only"` in Cargo.toml.

### 2. Record

```bash
LD_LIBRARY_PATH=target/release \
perf record -g --freq 999 -m 512K \
  -o /tmp/claude/perf.data \
  -- ./target/release/wow-sim --no-addons --no-saved-vars lua-errors
```

Variants:
- Full load with addons: drop `--no-addons --no-saved-vars`
- GUI startup: replace `lua-errors` with `screenshot` or just run the binary
- Custom Lua: add `--exec-lua "code"` before the subcommand

### 3. Generate SVG

```bash
perf script -i /tmp/claude/perf.data \
  | inferno-collapse-perf 2>/dev/null \
  | inferno-flamegraph > /tmp/claude/flamegraph.svg
```

### 4. Analyze collapsed stacks

```bash
perf script -i /tmp/claude/perf.data \
  | inferno-collapse-perf 2>/dev/null > /tmp/claude/collapsed.txt

# Top hotspots
sort -t' ' -k2 -rn /tmp/claude/collapsed.txt | head -40

# Category breakdown
awk '{sum += $NF} END {print "total:", sum}' /tmp/claude/collapsed.txt
grep 'PATTERN' /tmp/claude/collapsed.txt | awk '{sum += $NF} END {print sum}'
```

### Common category patterns

| Category | grep pattern |
|---|---|
| GC | `sweeplist\|luaC_step\|luaC_fullgc\|singlestep\|propagatemark` |
| Lua VM | `luaV_execute` (exclude `luaC_step`) |
| mlua FFI | `stack_value\|push_value\|lua_pushnil\|lua_replace\|lua_xmove\|lua_gettop\|lua_type` |
| Parse/load | `loadbuffer\|f_parser\|luaU_undump\|luaX_\|luaY_` |
| Hashing | `hash_one\|HashMap\|RawTable` |
| Layout | `compute_frame_rect` |
| Allocation | `malloc\|cfree\|alloc_shim\|drop_in_place` |
| Error traceback | `compat53_findfield\|luaL_traceback` |

### Notes

- `-m 512K` is needed because default mmap pages value is too large for perf to map
- `--freq 999` balances sampling resolution vs overhead (~10k samples for a 10s run)
- `[unknown]` frames mean inlined Rust code lost symbolization; build with `RUSTFLAGS="-C force-frame-pointers=yes"` to fix
- `LD_LIBRARY_PATH=target/release` is needed for the dynamically linked `libiced_dynamic.so`
- Always build separately from running — never put a timeout on `cargo build`
